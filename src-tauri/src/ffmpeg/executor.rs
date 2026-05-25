//! FFmpeg command executor with progress streaming and cancel support.
//!
//! Provides `execute_single_file()` which spawns an FFmpeg process for one
//! queue entry, streams progress events, and supports cancellation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::LogLevel;
use tauri::{AppHandle, Emitter};

use crate::ffmpeg::filters::{FilterKind, MetadataContext, build_filter_args_separated};
use crate::ffmpeg::probe::probe_global_metadata;
use crate::models::batch::PerFileProgress;
use crate::models::gpu::GpuEncoder;
use crate::models::seed::{Operation, OperationType, Seed};
use crate::models::video::VideoEntry;

/// Execute FFmpeg processing for a single video entry using the given seed.
///
/// # Arguments
/// * `app` - Tauri AppHandle for event emission
/// * `entry` - The video queue entry to process
/// * `seed` - The seed recipe to apply
/// * `ffmpeg_path` - Directory containing the ffmpeg binary (from Phase 1 store)
/// * `output_dir` - Directory to write the output file
/// * `cancel_flag` - Shared AtomicBool; checked before and during FFmpeg execution
/// * `gpu_encoder` - Optional GPU encoder detected at startup; None means CPU (libx264)
///
/// # Returns
/// * `Ok(output_path)` on success — the path to the completed output file
/// * `Err(message)` on failure or cancellation
///
/// Per D-10: if cancelled, kills the FFmpeg process and returns Err("Cancelled").
/// The caller (batch.rs) handles D-11 failure isolation — this function just
/// returns the result.
pub fn execute_single_file(
    app: &AppHandle,
    entry: &VideoEntry,
    seed: &Seed,
    ffmpeg_path: &str,
    output_dir: &str,
    cancel_flag: &AtomicBool,
    gpu_encoder: Option<&GpuEncoder>,
) -> Result<String, String> {
    // Check cancellation before starting
    if cancel_flag.load(Ordering::SeqCst) {
        return Err("Cancelled".to_string());
    }

    // Build output path: {original_stem}_{seed_alias}.{ext}
    let source_path = Path::new(&entry.filepath);
    let output_path = make_output_path(source_path, &seed.alias, Path::new(output_dir))?;

    // Build and merge filter arguments from all operations in the seed.
    // Pitfall: multiple video/audio filter ops each produced their own -vf/-af flag.
    // FFmpeg only honors the last one; earlier expressions become orphaned args.
    // Fix: collect video filter expressions into one comma-joined chain, same for audio.
    let mut vf_exprs: Vec<String> = Vec::new();
    let mut af_exprs: Vec<String> = Vec::new();
    let mut other_args: Vec<String> = Vec::new();

    // Phase 7: MetadataSelectiveErase needs current file metadata from ffprobe (D-12).
    // Probe the file for global metadata tags if any operation is MetadataSelectiveErase.
    // Pass the context into build_filter_args_separated so the filter builder can
    // determine which fields to keep vs erase.
    let metadata_ctx: Option<MetadataContext> = if seed
        .operations
        .iter()
        .any(|op| matches!(op.op_type, OperationType::MetadataSelectiveErase))
    {
        match probe_global_metadata(&entry.filepath) {
            Ok(fields) => Some(MetadataContext { fields }),
            Err(e) => {
                // Log the error but continue — fallback to full metadata erase
                let _ = app.emit(
                    "ffmpeg-debug-log",
                    serde_json::json!({
                        "file": entry.filename,
                        "level": "warning",
                        "message": format!("Cannot probe metadata for selective erase: {}", e),
                    }),
                );
                None
            }
        }
    } else {
        None
    };

    let total_video_frames = (entry.metadata.duration_secs * entry.metadata.fps as f64) as u32;

    // Crop's scale-back formula (iw*inv_w:ih*inv_h) breaks when FFmpeg
    // truncates crop dimensions — e.g. 927.12 crops to 926, then
    // 926*1.035=958 instead of 960. Explicit target dimensions fix this.
    let orig_w = entry.metadata.width;
    let orig_h = entry.metadata.height;
    let audio_sample_rate = entry.metadata.sample_rate;

    for op in &seed.operations {
        // TrimEdges needs totalFrames to set end_frame. totalFrames is
        // video-specific (depends on input duration/fps), so the executor
        // injects it at runtime rather than storing it in the seed.
        // Without this, totalFrames defaults to 0 → end_frame=0 → empty output.
        //
        // Crop needs orig_w/orig_h for reliable scale-back to original
        // dimensions (FFmpeg's crop+scale rounding causes 2px height loss).
        //
        // AudioPitch needs the actual audio sample rate from ffprobe.
        // The seed stores a generic default (48000) that may not match
        // the input file — e.g. input is 44100 Hz → asetrate miscomputes
        // the pitch shift, causing 12% audio duration shrinkage.
        let op_ref: &Operation;
        let mut op_with_frames;
        let needs_injection = matches!(op.op_type, OperationType::TrimEdges)
            || matches!(op.op_type, OperationType::Crop)
            || matches!(op.op_type, OperationType::AudioPitch);
        if needs_injection {
            op_with_frames = op.clone();
            if matches!(op.op_type, OperationType::TrimEdges)
                && !op_with_frames.params["totalFrames"].is_number()
            {
                op_with_frames.params["totalFrames"] = serde_json::json!(total_video_frames);
            }
            if matches!(op.op_type, OperationType::Crop) {
                if !op_with_frames.params["origW"].is_number() {
                    op_with_frames.params["origW"] = serde_json::json!(orig_w);
                }
                if !op_with_frames.params["origH"].is_number() {
                    op_with_frames.params["origH"] = serde_json::json!(orig_h);
                }
            }
            if matches!(op.op_type, OperationType::AudioPitch)
                && audio_sample_rate > 0
                && op_with_frames.params["originalRate"].as_u64().unwrap_or(0)
                    != audio_sample_rate as u64
            {
                op_with_frames.params["originalRate"] = serde_json::json!(audio_sample_rate);
            }
            op_ref = &op_with_frames;
        } else {
            op_ref = op;
        }

        let results = build_filter_args_separated(op_ref, metadata_ctx.as_ref())?;
        for (kind, _args) in results {
            match kind {
                FilterKind::VideoFilter(expr) => vf_exprs.push(expr),
                FilterKind::AudioFilter(expr) => af_exprs.push(expr),
                FilterKind::Other(args) => other_args.extend(args),
            }
        }
    }

    // Assemble final args: merged -vf, merged -af, then other args
    let mut all_args: Vec<String> = Vec::new();
    if !vf_exprs.is_empty() {
        // Append pad filter to force even output dimensions.
        // libx264 (yuv420p) requires even width and height — odd dimensions
        // from accumulated crop/scale floating-point rounding cause exit code 187.
        // pad=iw+mod(iw,2):ih+mod(ih,2) adds 0-1 px padding to make both even.
        // Using mod() avoids potential ceil() issues in FFmpeg's expression evaluator.
        let vf_chain = format!("{},pad=iw+mod(iw\\,2):ih+mod(ih\\,2)", vf_exprs.join(","));
        all_args.push("-vf".to_string());
        all_args.push(vf_chain);
    }
    if !af_exprs.is_empty() {
        all_args.push("-af".to_string());
        all_args.push(af_exprs.join(","));
    }
    // If video or audio filters are present, -c copy (remux) is incompatible —
    // filters require re-encoding. Skip -c copy pairs from other_args.
    let has_filtering = !vf_exprs.is_empty() || !af_exprs.is_empty();
    let mut i = 0;
    while i < other_args.len() {
        if has_filtering
            && other_args[i] == "-c"
            && i + 1 < other_args.len()
            && other_args[i + 1] == "copy"
        {
            i += 2; // skip "-c copy"
        } else {
            all_args.push(other_args[i].clone());
            i += 1;
        }
    }

    // Phase 7: FrameDrop uses 'select' filter which drops frames (D-17).
    // Without -vsync vfr, ffmpeg's default -vsync cfr inserts duplicate frames
    // to maintain constant frame rate, undoing the frame drop.
    // Inject -vsync vfr when any operation in the seed is FrameDrop.
    let has_frame_drop =
        seed.operations.iter().any(|op| matches!(op.op_type, OperationType::FrameDrop));

    // Inject GPU encoder or CPU fallback (Phase 5: PERF-01, D-04, D-05)
    let mut encoder_args: Vec<String> = if let Some(enc) = gpu_encoder {
        enc.encoder_args()
    } else {
        vec!["-c:v".to_string(), "libx264".to_string(), "-preset".to_string(), "medium".to_string()]
    };
    // Phase 7: -vsync vfr must come BEFORE encoder args to take effect
    if has_frame_drop {
        let mut vsync_args = vec!["-vsync".to_string(), "vfr".to_string()];
        vsync_args.extend(encoder_args);
        encoder_args = vsync_args;
    }
    encoder_args.extend(all_args);
    let all_args = encoder_args; // shadow with injected encoder + vsync args

    // Determine ffmpeg binary path
    let ffmpeg_bin = Path::new(ffmpeg_path).join(if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });
    let ffmpeg_bin_str = ffmpeg_bin.to_string_lossy().to_string();

    // Spawn and iterate.
    // Per ffmpeg-sidecar 2.5.x API: FfmpegCommand::new_with_path() accepts path,
    // .input() sets input file, .args() appends CLI arguments, .output() sets output.
    // Note: .output() requires AsRef<str>, so convert PathBuf to string.
    let output_path_str_for_cmd = output_path.to_string_lossy().to_string();

    // Diagnostic: emit the full FFmpeg command line for debugging
    let cmd_diag = format!(
        "{} -i {} {} {}",
        ffmpeg_bin_str,
        entry.filepath,
        all_args.join(" "),
        output_path_str_for_cmd
    );
    let _ = app.emit(
        "ffmpeg-debug-log",
        serde_json::json!({
            "file": entry.filename,
            "level": "info",
            "message": format!("FFmpeg cmd: {}", cmd_diag),
        }),
    );
    let mut child = FfmpegCommand::new_with_path(&ffmpeg_bin_str)
        .input(&entry.filepath)
        .args(&all_args)
        .output(&output_path_str_for_cmd)
        .spawn()
        .map_err(|e| format!("FFmpeg spawn failed: {}", e))?;

    // Use child.iter() to drain stderr and parse progress events.
    // Pitfall 1 mitigation: ffmpeg-sidecar's iter() drains stderr continuously,
    // preventing pipe buffer deadlock.
    // D-08: child.iter() streams stderr continuously without buffering entire output.
    // This prevents memory exhaustion on large/long video files. Verified: ffmpeg-sidecar
    // 2.5.x iter() drains the pipe in real-time, emitting FfmpegEvent::Progress per frame.
    let output_path_clone = output_path.clone();
    let output_path_str = output_path_clone.to_string_lossy().to_string();
    let app_clone = app.clone();
    let filename = entry.filename.clone();
    let total_duration = entry.metadata.duration_secs;

    let mut ffmpeg_log: Vec<String> = Vec::new();

    for event in child.iter().map_err(|e| format!("FFmpeg iteration error: {}", e))? {
        // Pitfall 5: Always use SeqCst for cancel flag visibility on ARM
        if cancel_flag.load(Ordering::SeqCst) {
            // D-10: Kill FFmpeg process on cancel
            let _ = child.kill();
            let _ = std::fs::remove_file(&output_path_clone);
            return Err("Cancelled".to_string());
        }

        // Parse progress from FfmpegEvent
        match event {
            ffmpeg_sidecar::event::FfmpegEvent::Progress(progress) => {
                let seconds = parse_time_to_seconds(&progress.time);
                let percent = if total_duration > 0.0 {
                    (seconds / total_duration * 100.0).clamp(0.0, 100.0)
                } else {
                    0.0
                };
                let remaining = if progress.speed > 0.01 {
                    (total_duration - seconds) / progress.speed as f64
                } else {
                    0.0
                };
                let total_frames = (total_duration * entry.metadata.fps as f64) as u32;

                let _ = app_clone.emit(
                    "batch-file-progress",
                    PerFileProgress {
                        file: filename.clone(),
                        seed_alias: seed.alias.clone(),
                        percent,
                        current_frame: progress.frame,
                        total_frames,
                        fps: progress.fps,
                        remaining_seconds: remaining.max(0.0),
                    },
                );
            }
            ffmpeg_sidecar::event::FfmpegEvent::Log(LogLevel::Warning, msg)
            | ffmpeg_sidecar::event::FfmpegEvent::Log(LogLevel::Error, msg) => {
                ffmpeg_log.push(msg.clone());
                let _ = app_clone.emit(
                    "ffmpeg-debug-log",
                    serde_json::json!({
                        "file": filename,
                        "level": "warning",
                        "message": msg,
                    }),
                );
            }
            ffmpeg_sidecar::event::FfmpegEvent::Log(_, msg) => {
                // Capture all other log levels (Info, Debug, etc.) too
                ffmpeg_log.push(msg.clone());
                let _ = app_clone.emit(
                    "ffmpeg-debug-log",
                    serde_json::json!({
                        "file": filename,
                        "level": "info",
                        "message": msg,
                    }),
                );
            }
            _ => {} // Ignore other events
        }
    }

    // Wait for process completion
    let status = child.wait().map_err(|e| format!("FFmpeg wait error: {}", e))?;

    if status.success() {
        Ok(output_path_str)
    } else {
        let exit_code = status.code().unwrap_or(-1);
        let log_tail: String = if ffmpeg_log.is_empty() {
            String::new()
        } else {
            let start = ffmpeg_log.len().saturating_sub(10);
            format!("\nFFmpeg last log lines:\n{}", ffmpeg_log[start..].join("\n"))
        };
        Err(format!(
            "FFmpeg exited with code {}. Cmd: {} -i {} {} {}{}",
            exit_code,
            ffmpeg_bin_str,
            entry.filepath,
            all_args.join(" "),
            output_path_str,
            log_tail
        ))
    }
}

/// Parse an FFmpeg time string (e.g., "00:03:29.04", "01:30.50", or "123.45") to seconds.
fn parse_time_to_seconds(time_str: &str) -> f64 {
    if time_str.contains(':') {
        let parts: Vec<&str> = time_str.split(':').collect();
        match parts.len() {
            3 => {
                // HH:MM:SS.mm
                let h: f64 = parts[0].parse().unwrap_or(0.0);
                let m: f64 = parts[1].parse().unwrap_or(0.0);
                let s: f64 = parts[2].parse().unwrap_or(0.0);
                h * 3600.0 + m * 60.0 + s
            }
            2 => {
                // MM:SS.mm (videos under 1 hour)
                let m: f64 = parts[0].parse().unwrap_or(0.0);
                let s: f64 = parts[1].parse().unwrap_or(0.0);
                m * 60.0 + s
            }
            _ => time_str.parse().unwrap_or(0.0),
        }
    } else {
        // No colons: plain seconds as float
        time_str.parse().unwrap_or(0.0)
    }
}

/// Build the output file path with collision-safe naming.
/// Per D-16: {original_stem}_{seed_alias}.{ext}
/// If file exists, appends -1, -2, etc. before extension.
fn make_output_path(
    source_path: &Path,
    seed_alias: &str,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)
        .map_err(|e| format!("Cannot create output directory: {}", e))?;

    let stem = source_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| std::borrow::Cow::Borrowed("output"));
    let ext = source_path
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "mp4".to_string());

    let base_name = format!("{}_{}", stem, seed_alias);
    let mut candidate = output_dir.join(format!("{}.{}", base_name, ext));

    // D-16: Collision detection with numeric suffix
    let mut suffix = 1;
    while candidate.exists() {
        candidate = output_dir.join(format!("{}-{}.{}", base_name, suffix, ext));
        suffix += 1;
    }

    Ok(candidate)
}
