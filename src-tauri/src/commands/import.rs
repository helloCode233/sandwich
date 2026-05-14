//! Video import commands with ffprobe validation.
//!
//! Provides `import_video` — validates extension (D-12), runs ffprobe (D-14),
//! checks disk space (D-13), allows duplicates (D-15), and persists to queue.

use std::path::Path;
use std::sync::Mutex;

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

    let entry = VideoEntry {
        filename,
        filepath: filepath.clone(),
        metadata,
        status: VideoStatus::Valid,
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
    if let Ok(store) = app.store("ffmpeg-config.json") {
        if let Some(value) = store.get("ffmpeg_path") {
            if let Some(path_str) = value.as_str() {
                return Some(path_str.to_string());
            }
        }
    }
    None
}

/// Check available disk space. Per D-13: no hard limit, but warn if low.
fn check_disk_space_for_output(app: &AppHandle) -> Result<(), String> {
    let output_dir = get_output_dir(app);
    let output_path = Path::new(&output_dir);

    if !output_path.exists() {
        std::fs::create_dir_all(output_path).ok();
    }

    if let Ok(available) = fs2::available_space(output_path) {
        if available < 100_000_000 {
            let _ = app.emit(
                "low-disk-space",
                serde_json::json!({
                    "available_bytes": available,
                    "message": "Low disk space — less than 100MB available on output volume.",
                }),
            );
        }
    }

    Ok(())
}

/// Get the output directory from preferences, or default.
fn get_output_dir(app: &AppHandle) -> String {
    if let Ok(store) = app.store("sandwich-config.json") {
        if let Some(value) = store.get("output_dir") {
            if let Some(dir_str) = value.as_str() {
                let s = dir_str.to_string();
                if s.starts_with('~') {
                    // Expand legacy tilde-prefixed paths
                    if let Ok(home) = std::env::var("HOME") {
                        return s.replacen('~', &home, 1);
                    }
                }
                return s;
            }
        }
    }

    #[cfg(target_os = "windows")]
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    #[cfg(not(target_os = "windows"))]
    let home = std::env::var("HOME").unwrap_or_default();

    Path::new(&home)
        .join("Videos")
        .join("sandwich-output")
        .to_string_lossy()
        .to_string()
}

/// Persist the video queue to tauri-plugin-store.
fn persist_queue_import(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    let store = app
        .store("queue.json")
        .map_err(|e| format!("Failed to open queue store: {}", e))?;
    let json = serde_json::to_value(&*app_state.queue)
        .map_err(|e| format!("Serialization error: {}", e))?;
    store.set("queue", json);
    store
        .save()
        .map_err(|e| format!("Failed to save queue: {}", e))?;

    Ok(())
}
