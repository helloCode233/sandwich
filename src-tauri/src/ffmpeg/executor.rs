//! FFmpeg command executor with progress streaming and cancel support.
//!
//! Provides `execute_single_file()` which spawns an FFmpeg process for one
//! queue entry, streams progress events, and supports cancellation.
//!
//! Plan 18: Two-pass GPU-first execution.
//! When both GPU encoder and GPU-capable operations are present:
//!   Pass 1 (GPU): hwaccel decode → GPU filters → NVENC encode with compression → intermediate
//!   Pass 2 (CPU): decode intermediate → CPU-only filters → encode → final output

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::LogLevel;
use tauri::{AppHandle, Emitter};

use crate::commands::seed::is_gpu_capable;
use crate::ffmpeg::filters::{FilterKind, MetadataContext, build_filter_args_separated};
use crate::ffmpeg::probe::probe_global_metadata;
use crate::models::batch::PerFileProgress;
use crate::models::gpu::GpuEncoder;
use crate::models::seed::{Operation, OperationType, Seed};
use crate::models::video::VideoEntry;

/// Internal result from building and grouping filter args per operation.
/// Used to route filters to the correct execution pass.
struct OpFilterArgs {
    vf_exprs: Vec<String>,
    af_exprs: Vec<String>,
    other_args: Vec<String>,
}

impl OpFilterArgs {
    fn is_empty(&self) -> bool {
        self.vf_exprs.is_empty() && self.af_exprs.is_empty() && self.other_args.is_empty()
    }
}

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
///
/// Plan 18: when GPU encoder is available and the seed contains GPU-capable
/// operations, execution uses two FFmpeg passes:
///   1. GPU pass: GPU ops with hwaccel + NVENC compression → temp file
///   2. CPU pass: CPU ops on temp file → final output
/// If no GPU ops or no GPU encoder, falls back to single-pass.
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

    // Phase 7: MetadataSelectiveErase needs current file metadata from ffprobe (D-12).
    let metadata_ctx: Option<MetadataContext> = if seed
        .operations
        .iter()
        .any(|op| matches!(op.op_type, OperationType::MetadataSelectiveErase))
    {
        match probe_global_metadata(&entry.filepath, Some(ffmpeg_path)) {
            Ok(fields) => Some(MetadataContext { fields }),
            Err(e) => {
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

    // Plan 18: Determine if two-pass execution should be used.
    // Two-pass is enabled when a GPU encoder is available AND the seed
    // contains at least one GPU-capable operation.
    let hwaccel_active = gpu_encoder.map_or(false, |e| matches!(e, GpuEncoder::Nvenc(_)));
    let has_gpu_ops =
        gpu_encoder.is_some() && seed.operations.iter().any(|op| is_gpu_capable(op.op_type));

    let total_video_frames = (entry.metadata.duration_secs * entry.metadata.fps as f64) as u32;
    let orig_w = entry.metadata.width;
    let orig_h = entry.metadata.height;
    let audio_sample_rate = entry.metadata.sample_rate;

    // Build filter args and group by GPU/CPU
    let mut gpu_args =
        OpFilterArgs { vf_exprs: Vec::new(), af_exprs: Vec::new(), other_args: Vec::new() };
    let mut cpu_args =
        OpFilterArgs { vf_exprs: Vec::new(), af_exprs: Vec::new(), other_args: Vec::new() };

    for op in &seed.operations {
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

        let results = build_filter_args_separated(
            op_ref,
            metadata_ctx.as_ref(),
            gpu_encoder,
            hwaccel_active,
        )?;

        // Plan 18: route to GPU or CPU group based on operation type
        let is_gpu_op = has_gpu_ops && is_gpu_capable(op.op_type);
        let target = if is_gpu_op { &mut gpu_args } else { &mut cpu_args };

        for (kind, _args) in results {
            match kind {
                FilterKind::VideoFilter(expr) => target.vf_exprs.push(expr),
                FilterKind::AudioFilter(expr) => target.af_exprs.push(expr),
                FilterKind::Other(args) => target.other_args.extend(args),
            }
        }
    }

    // Determine ffmpeg binary path
    let ffmpeg_bin = Path::new(ffmpeg_path).join(if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });
    let ffmpeg_bin_str = ffmpeg_bin.to_string_lossy().to_string();

    // Shared metadata for progress reporting
    let filename = entry.filename.clone();
    let total_duration = entry.metadata.duration_secs;
    let fps = entry.metadata.fps as f64;

    // ── Plan 18: Two-pass execution ──────────────────────────────────
    if has_gpu_ops && !gpu_args.is_empty() {
        // Compute compression-adjusted encoder args for GPU pass.
        // GPU pass uses -cq 28 (higher CRF = more compression) for smaller
        // intermediate files since quality only needs to survive one re-encode.
        let compress_gpu = |args: &[String]| -> Vec<String> {
            let mut compressed = Vec::with_capacity(args.len());
            let mut i = 0;
            while i < args.len() {
                if args[i] == "-cq" && i + 1 < args.len() {
                    compressed.push("-cq".to_string());
                    compressed.push("28".to_string()); // higher CRF = more compression
                    i += 2;
                } else {
                    compressed.push(args[i].clone());
                    i += 1;
                }
            }
            compressed
        };

        let gpu_enc = gpu_encoder.unwrap(); // safe: has_gpu_ops requires gpu_encoder.is_some()
        let gpu_enc_args = compress_gpu(&gpu_enc.encoder_args());

        // Create temp directory for intermediate file
        let temp_dir = std::env::temp_dir().join("sandwich_two_pass");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Cannot create temp dir for two-pass: {}", e))?;

        let gpu_output = temp_dir.join(format!(
            "{}_{}_gpu.mp4",
            source_path.file_stem().map(|s| s.to_string_lossy()).unwrap_or_else(|| "output".into()),
            seed.alias
        ));
        let gpu_output_str = gpu_output.to_string_lossy().to_string();

        // Assemble GPU pass args (video filters only + pad, no audio)
        let gpu_all_args = assemble_pass_args(
            &gpu_args.vf_exprs,
            &gpu_args.af_exprs,
            &gpu_args.other_args,
            seed,
            &gpu_enc_args,
            true, // hwaccel for GPU pass
        );

        // Emit debug log for GPU pass
        let gpu_cmd_diag = format!(
            "-hwaccel cuda -hwaccel_output_format cuda {}-i {} {} {}",
            ffmpeg_bin_str,
            entry.filepath,
            gpu_all_args.join(" "),
            gpu_output_str
        );
        let _ = app.emit(
            "ffmpeg-debug-log",
            serde_json::json!({
                "file": filename,
                "level": "info",
                "message": format!("[GPU Pass] FFmpeg cmd: {}", gpu_cmd_diag),
            }),
        );

        // ── Execute GPU pass ──
        run_ffmpeg_pass(
            app,
            &entry.filepath,
            &gpu_output_str,
            &ffmpeg_bin_str,
            &gpu_all_args,
            &filename,
            &seed.alias,
            total_duration,
            fps,
            cancel_flag,
            true, // hwaccel_active for GPU pass
        )?;

        // Check cancellation between passes
        if cancel_flag.load(Ordering::SeqCst) {
            let _ = std::fs::remove_file(&gpu_output);
            return Err("Cancelled".to_string());
        }

        // ── Execute CPU pass on GPU intermediate output ──
        let output_path_str = output_path.to_string_lossy().to_string();

        let cpu_enc_args: Vec<String> = if let Some(enc) = gpu_encoder {
            enc.encoder_args()
        } else {
            vec![
                "-c:v".to_string(),
                "libx264".to_string(),
                "-preset".to_string(),
                "medium".to_string(),
            ]
        };

        // For CPU pass, audio filters are also applied
        let cpu_all_args = assemble_pass_args(
            &cpu_args.vf_exprs,
            &cpu_args.af_exprs,
            &cpu_args.other_args,
            seed,
            &cpu_enc_args,
            false, // no hwaccel for CPU pass (input is already decoded intermediate)
        );

        let cpu_cmd_diag = format!(
            "{}-i {} {} {}",
            ffmpeg_bin_str,
            gpu_output_str,
            cpu_all_args.join(" "),
            output_path_str
        );
        let _ = app.emit(
            "ffmpeg-debug-log",
            serde_json::json!({
                "file": filename,
                "level": "info",
                "message": format!("[CPU Pass] FFmpeg cmd: {}", cpu_cmd_diag),
            }),
        );

        let cpu_result = run_ffmpeg_pass(
            app,
            &gpu_output_str,
            &output_path_str,
            &ffmpeg_bin_str,
            &cpu_all_args,
            &filename,
            &seed.alias,
            total_duration,
            fps,
            cancel_flag,
            false, // no hwaccel for CPU pass
        );

        // Cleanup intermediate temp file regardless of CPU pass result
        let _ = std::fs::remove_file(&gpu_output);

        cpu_result?;

        return Ok(output_path_str);
    }

    // ── Fallback: Single-pass execution (no GPU ops or no GPU encoder) ──
    // Merge GPU+CPU args back into flat lists since two-pass wasn't used
    let mut vf_exprs = gpu_args.vf_exprs;
    vf_exprs.extend(cpu_args.vf_exprs);
    let mut af_exprs = gpu_args.af_exprs;
    af_exprs.extend(cpu_args.af_exprs);
    let mut other_args = gpu_args.other_args;
    other_args.extend(cpu_args.other_args);

    let enc_args: Vec<String> = if let Some(enc) = gpu_encoder {
        enc.encoder_args()
    } else {
        vec!["-c:v".to_string(), "libx264".to_string(), "-preset".to_string(), "medium".to_string()]
    };

    let all_args =
        assemble_pass_args(&vf_exprs, &af_exprs, &other_args, seed, &enc_args, hwaccel_active);

    let output_path_str = output_path.to_string_lossy().to_string();

    // Diagnostic for single pass
    let hwaccel_str =
        if hwaccel_active { "-hwaccel cuda -hwaccel_output_format cuda " } else { "" };
    let cmd_diag = format!(
        "{}{}-i {} {} {}",
        hwaccel_str,
        ffmpeg_bin_str,
        entry.filepath,
        all_args.join(" "),
        output_path_str
    );
    let _ = app.emit(
        "ffmpeg-debug-log",
        serde_json::json!({
            "file": filename,
            "level": "info",
            "message": format!("FFmpeg cmd: {}", cmd_diag),
        }),
    );

    run_ffmpeg_pass(
        app,
        &entry.filepath,
        &output_path_str,
        &ffmpeg_bin_str,
        &all_args,
        &filename,
        &seed.alias,
        total_duration,
        fps,
        cancel_flag,
        hwaccel_active,
    )?;

    Ok(output_path_str)
}

/// Assemble FFmpeg arguments for a single pass (GPU or CPU).
///
/// Combines video filters (with pad for even dimensions), audio filters,
/// other args (stripping -c copy if filters present), FrameDrop vsync,
/// encoder args, and hwaccel-awareness (hwdownload prefix for GPU decode).
fn assemble_pass_args(
    vf_exprs: &[String],
    af_exprs: &[String],
    other_args: &[String],
    seed: &Seed,
    encoder_args: &[String],
    hwaccel_active: bool,
) -> Vec<String> {
    let raw_vf: Vec<String> = vf_exprs.to_vec();

    // When hwaccel is active, frames start on GPU. GPU-native filters
    // (containing _cuda) must run BEFORE hwdownload; CPU filters after.
    let mut vf: Vec<String> = if hwaccel_active && !raw_vf.is_empty() {
        let (gpu_vf, cpu_vf): (Vec<_>, Vec<_>) = raw_vf
            .into_iter()
            .partition(|expr| expr.contains("_cuda") || expr.contains("hwupload"));
        let mut ordered = gpu_vf;
        if !cpu_vf.is_empty() {
            ordered.push("hwdownload,format=nv12".to_string());
            ordered.extend(cpu_vf);
        }
        ordered
    } else {
        raw_vf
    };

    let mut all_args: Vec<String> = Vec::new();

    if !vf.is_empty() {
        let vf_chain = format!("{},pad=iw+mod(iw\\,2):ih+mod(ih\\,2)", vf.join(","));
        all_args.push("-vf".to_string());
        all_args.push(vf_chain);
    }
    if !af_exprs.is_empty() {
        all_args.push("-af".to_string());
        all_args.push(af_exprs.join(","));
    }

    // If video or audio filters are present, -c copy (remux) is incompatible
    let has_filtering = !vf.is_empty() || !af_exprs.is_empty();
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

    // FrameDrop requires -vsync vfr
    let has_frame_drop =
        seed.operations.iter().any(|op| matches!(op.op_type, OperationType::FrameDrop));
    let mut final_args: Vec<String> = Vec::new();
    if has_frame_drop {
        final_args.push("-vsync".to_string());
        final_args.push("vfr".to_string());
    }
    final_args.extend_from_slice(encoder_args);
    final_args.extend(all_args);

    final_args
}

/// Run a single FFmpeg pass: spawn, iterate progress, wait for completion.
///
/// Returns `Ok(())` on success or `Err(message)` on failure/cancellation.
fn run_ffmpeg_pass(
    app: &AppHandle,
    input_path: &str,
    output_path_str: &str,
    ffmpeg_bin_str: &str,
    ffmpeg_args: &[String],
    filename: &str,
    seed_alias: &str,
    total_duration: f64,
    fps: f64,
    cancel_flag: &AtomicBool,
    hwaccel_active: bool,
) -> Result<(), String> {
    // Build spawn arguments with optional hwaccel prefix before -i
    let mut spawn_args: Vec<String> = Vec::with_capacity(ffmpeg_args.len() + 6);
    if hwaccel_active {
        spawn_args.push("-hwaccel".to_string());
        spawn_args.push("cuda".to_string());
        spawn_args.push("-hwaccel_output_format".to_string());
        spawn_args.push("cuda".to_string());
    }
    spawn_args.push("-i".to_string());
    spawn_args.push(input_path.to_string());
    spawn_args.extend_from_slice(ffmpeg_args);

    let mut ffmpeg_cmd = FfmpegCommand::new_with_path(ffmpeg_bin_str);
    let mut child = ffmpeg_cmd
        .args(&spawn_args)
        .output(output_path_str)
        .spawn()
        .map_err(|e| format!("FFmpeg spawn failed: {}", e))?;

    let output_path_owned = PathBuf::from(output_path_str);
    let app_clone = app.clone();
    let filename_owned = filename.to_string();
    let seed_alias_owned = seed_alias.to_string();

    let mut ffmpeg_log: Vec<String> = Vec::new();

    for event in child.iter().map_err(|e| format!("FFmpeg iteration error: {}", e))? {
        if cancel_flag.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = std::fs::remove_file(&output_path_owned);
            return Err("Cancelled".to_string());
        }

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
                let total_frames = (total_duration * fps) as u32;

                let _ = app_clone.emit(
                    "batch-file-progress",
                    PerFileProgress {
                        file: filename_owned.clone(),
                        seed_alias: seed_alias_owned.clone(),
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
                        "file": filename_owned,
                        "level": "warning",
                        "message": msg,
                    }),
                );
            }
            ffmpeg_sidecar::event::FfmpegEvent::Log(_, msg) => {
                ffmpeg_log.push(msg.clone());
                let _ = app_clone.emit(
                    "ffmpeg-debug-log",
                    serde_json::json!({
                        "file": filename_owned,
                        "level": "info",
                        "message": msg,
                    }),
                );
            }
            _ => {}
        }
    }

    let status = child.wait().map_err(|e| format!("FFmpeg wait error: {}", e))?;

    if status.success() {
        Ok(())
    } else {
        let exit_code = status.code().unwrap_or(-1);
        let log_tail: String = if ffmpeg_log.is_empty() {
            String::new()
        } else {
            let start = ffmpeg_log.len().saturating_sub(10);
            format!("\nFFmpeg last log lines:\n{}", ffmpeg_log[start..].join("\n"))
        };
        let hw_str = if hwaccel_active { "-hwaccel cuda -hwaccel_output_format cuda " } else { "" };
        Err(format!(
            "FFmpeg exited with code {}. Cmd: {}{}-i {} {} {}{}",
            exit_code,
            hw_str,
            ffmpeg_bin_str,
            input_path,
            ffmpeg_args.join(" "),
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
