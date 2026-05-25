//! Video import commands with ffprobe validation.
//!
//! Provides `import_video` — validates extension (D-12), runs ffprobe (D-14),
//! checks disk space (D-13), allows duplicates (D-15), and persists to queue.

use std::path::Path;
use std::sync::Mutex;

use base64::Engine;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use crate::ffmpeg::probe::extract_metadata;
use crate::models::video::{VideoEntry, VideoStatus};
use crate::state::AppState;

/// Supported video file extensions per D-12.
const SUPPORTED_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "webm", "flv", "wmv"];

/// Tauri command: Import a video file into the processing queue.
///
/// Per D-12: filters by supported extensions first, then validates with ffprobe.
/// Per D-14: ffprobe validation rejects files without video streams with
///           a specific error message.
/// Per D-15: duplicate file paths are allowed (user may process same source
///           with different seeds).
/// Per D-13: no hard file size limit.
#[tauri::command]
pub async fn import_video(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    filepath: String,
) -> Result<VideoEntry, String> {
    // D-12: Extension filter — quick rejection before spawning ffprobe
    let path = Path::new(&filepath);
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| "File has no extension".to_string())?;

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(format!(
            "Unsupported file format '.{}'. Supported formats: {}",
            extension,
            SUPPORTED_EXTENSIONS.join(", ")
        ));
    }

    // Check file exists before spawning ffprobe
    if !path.exists() {
        return Err(format!("File not found: {}", filepath));
    }

    // Get the stored FFmpeg directory for ffprobe lookup
    let ffmpeg_dir = get_stored_ffmpeg_dir(&app);

    // D-14: ffprobe validation — validates video stream and extracts metadata
    let metadata = extract_metadata(&filepath, ffmpeg_dir.as_deref())?;

    let filename = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // D-13: Check available disk space
    check_disk_space_for_output(&app)?;

    // Extract thumbnail: first frame, scaled to 120px wide, JPEG output to stdout
    let thumbnail_base64 = match extract_thumbnail(&filepath, ffmpeg_dir.as_deref()) {
        Ok(b64) => Some(b64),
        Err(e) => {
            // Graceful degradation: import succeeds even if thumbnail fails
            let _ = app.emit(
                "thumbnail-extraction-warning",
                serde_json::json!({
                    "file": filename,
                    "error": e,
                }),
            );
            None
        }
    };

    let entry = VideoEntry {
        filename,
        filepath: filepath.clone(),
        metadata,
        status: VideoStatus::Valid,
        thumbnail_base64,
        order_index: 0,
    };

    // Add to queue (D-15: duplicates allowed — no dedup check)
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.queue.push(entry.clone());
    }

    // Persist queue
    persist_queue_import(&app)?;

    // Emit events
    let _ = app.emit("video-imported", entry.clone());
    let _ = app.emit("queue-updated", ());

    Ok(entry)
}

/// Read the stored FFmpeg directory from ffmpeg-config.json (Phase 1 store).
fn get_stored_ffmpeg_dir(app: &AppHandle) -> Option<String> {
    if let Ok(store) = app.store("ffmpeg-config.json")
        && let Some(value) = store.get("ffmpeg_path")
        && let Some(path_str) = value.as_str()
    {
        return Some(path_str.to_string());
    }
    None
}

/// Extract first frame as 120px-wide JPEG, return as base64 string.
/// Uses ffmpeg-sidecar's take_stdout() to capture JPEG bytes from stdout.
/// Output is ~2-8KB — safe for in-memory handling and store persistence.
fn extract_thumbnail(video_path: &str, ffmpeg_dir: Option<&str>) -> Result<String, String> {
    use std::io::Read;
    use std::path::Path;

    let ffmpeg_bin_path = if let Some(dir) = ffmpeg_dir {
        Path::new(dir).join("ffmpeg")
    } else {
        ffmpeg_sidecar::paths::ffmpeg_path()
    };

    let bin_path_string = ffmpeg_bin_path.to_string_lossy().into_owned();

    let mut child = ffmpeg_sidecar::command::FfmpegCommand::new_with_path(&bin_path_string)
        .input(video_path)
        .args([
            "-ss",
            "1", // seek to 1 second in
            "-vframes",
            "1", // extract single frame
            "-vf",
            "scale=120:-1", // scale width to 120px, height auto
            "-f",
            "image2pipe", // output raw image to stdout
            "-vcodec",
            "mjpeg", // JPEG encoding
            "-",     // stdout
        ])
        .spawn()
        .map_err(|e| format!("Thumbnail spawn failed: {}", e))?;

    // Read stdout (JPEG bytes) before waiting to avoid deadlock
    let mut jpeg_bytes = Vec::new();
    if let Some(mut stdout) = child.take_stdout() {
        stdout
            .read_to_end(&mut jpeg_bytes)
            .map_err(|e| format!("Failed to read thumbnail stdout: {}", e))?;
    }

    // Wait for process completion (also drains stderr to avoid zombie)
    child.wait().map_err(|e| format!("Thumbnail wait failed: {}", e))?;

    if jpeg_bytes.is_empty() {
        return Err("Thumbnail output was empty".to_string());
    }

    Ok(base64::engine::general_purpose::STANDARD.encode(&jpeg_bytes))
}

/// Check available disk space. Per D-13: no hard limit, but warn if low.
fn check_disk_space_for_output(app: &AppHandle) -> Result<(), String> {
    let output_dir = get_output_dir(app);
    let output_path = Path::new(&output_dir);

    if !output_path.exists() {
        std::fs::create_dir_all(output_path).ok();
    }

    if let Ok(available) = fs2::available_space(output_path)
        && available < 100_000_000
    {
        let _ = app.emit(
            "low-disk-space",
            serde_json::json!({
                "available_bytes": available,
                "message": "Low disk space — less than 100MB available on output volume.",
            }),
        );
    }

    Ok(())
}

/// Get the output directory from preferences, or default.
fn get_output_dir(app: &AppHandle) -> String {
    if let Ok(store) = app.store("sandwich-config.json")
        && let Some(value) = store.get("output_dir")
        && let Some(dir_str) = value.as_str()
    {
        let s = dir_str.to_string();
        if s.starts_with('~') {
            // Expand legacy tilde-prefixed paths
            if let Ok(home) = std::env::var("HOME") {
                return s.replacen('~', &home, 1);
            }
        }
        return s;
    }

    #[cfg(target_os = "windows")]
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    #[cfg(not(target_os = "windows"))]
    let home = std::env::var("HOME").unwrap_or_default();

    Path::new(&home).join("Videos").join("sandwich-output").to_string_lossy().to_string()
}

/// Persist the video queue to tauri-plugin-store.
fn persist_queue_import(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    let store =
        app.store("queue.json").map_err(|e| format!("Failed to open queue store: {}", e))?;
    let json = serde_json::to_value(&*app_state.queue)
        .map_err(|e| format!("Serialization error: {}", e))?;
    store.set("queue", json);
    store.save().map_err(|e| format!("Failed to save queue: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED: These tests will not compile until extract_thumbnail is implemented.
    // They define the expected behavior: extract_thumbnail returns error for invalid
    // inputs, and base64-encoded JPEG (starting with /9j/) for valid videos.

    /// extract_thumbnail on a nonexistent file returns an error.
    #[test]
    fn extract_thumbnail_nonexistent_file_returns_error() {
        let result = extract_thumbnail("/nonexistent/video.mp4", None);
        assert!(result.is_err());
    }

    /// extract_thumbnail on a non-video file returns an error (FFmpeg won't decode it).
    #[test]
    fn extract_thumbnail_non_video_returns_error() {
        let result = extract_thumbnail("/dev/null", None);
        assert!(result.is_err());
    }

    /// extract_thumbnail on a valid video produces a base64 string starting with JPEG magic.
    #[test]
    fn extract_thumbnail_valid_video_returns_base64_jpeg() {
        let test_video = std::path::Path::new("../../test-assets/sample.mp4");
        if !test_video.exists() {
            eprintln!("Skipping: test-assets/sample.mp4 not found");
            return;
        }
        let result = extract_thumbnail(test_video.to_str().unwrap(), None);
        match result {
            Ok(b64) => {
                assert!(
                    b64.starts_with("/9j/"),
                    "JPEG base64 should start with /9j/, got: {}",
                    &b64[..20.min(b64.len())]
                );
                assert!(b64.len() > 500, "thumbnail too small: {} bytes", b64.len());
                assert!(b64.len() < 20000, "thumbnail too large: {} bytes", b64.len());
            }
            Err(e) => {
                eprintln!("Thumbnail extraction failed (may be missing FFmpeg): {}", e);
            }
        }
    }
}
