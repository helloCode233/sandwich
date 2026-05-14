//! FFmpeg command executor with progress streaming and cancel support.
//!
//! Provides `execute_single_file()` which spawns an FFmpeg process for one
//! queue entry, streams progress events, and supports cancellation.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::LogLevel;
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::ffmpeg::filters::build_filter_args;
use crate::models::batch::PerFileProgress;
use crate::models::seed::Seed;
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
) -> Result<String, String> {
    // Check cancellation before starting
    if cancel_flag.load(Ordering::SeqCst) {
        return Err("Cancelled".to_string());
    }

    // Build output path: {original_stem}_{seed_alias}.{ext}
    let source_path = Path::new(&entry.filepath);
    let output_path = make_output_path(source_path, &seed.alias, Path::new(output_dir))?;

    // Build filter arguments from all operations in the seed
    let mut all_args: Vec<String> = Vec::new();
    for op in &seed.operations {
        let filter_args = build_filter_args(op)?;
        all_args.extend(filter_args);
    }

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
    let mut child = FfmpegCommand::new_with_path(&ffmpeg_bin_str)
        .input(&entry.filepath)
        .args(&all_args)
        .output(&output_path_str_for_cmd)
        .spawn()
        .map_err(|e| format!("FFmpeg spawn failed: {}", e))?;

    // Use child.iter() to drain stderr and parse progress events.
    // Pitfall 1 mitigation: ffmpeg-sidecar's iter() drains stderr continuously,
    // preventing pipe buffer deadlock.
    let output_path_clone = output_path.clone();
    let output_path_str = output_path_clone.to_string_lossy().to_string();
    let app_clone = app.clone();
    let filename = entry.filename.clone();
    let total_duration = entry.metadata.duration_secs;

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
                let _ = app_clone.emit(
                    "batch-log",
                    serde_json::json!({
                        "file": filename,
                        "level": "warning",
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
        Err(format!("FFmpeg exited with code {}", exit_code))
    }
}

/// Parse an FFmpeg time string (e.g., "00:03:29.04" or "123.45") to seconds.
fn parse_time_to_seconds(time_str: &str) -> f64 {
    // Try HH:MM:SS.mmm format first
    if time_str.contains(':') {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() == 3 {
            let h: f64 = parts[0].parse().unwrap_or(0.0);
            let m: f64 = parts[1].parse().unwrap_or(0.0);
            let s: f64 = parts[2].parse().unwrap_or(0.0);
            return h * 3600.0 + m * 60.0 + s;
        }
    }
    // Fallback: plain seconds as float
    time_str.parse().unwrap_or(0.0)
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
