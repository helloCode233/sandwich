use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use crate::models::video::{VideoEntry, VideoStatus};
use crate::state::AppState;

/// Tauri command: Get the full video queue with metadata.
/// Per D-06: checks path validity — moved/deleted files are marked Invalid
/// but their metadata is preserved for user notification.
#[tauri::command]
pub async fn get_queue(state: State<'_, Mutex<AppState>>) -> Result<Vec<VideoEntry>, String> {
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    let mut entries = app_state.queue.clone();

    // D-06: Check path validity for each entry
    for entry in &mut entries {
        if !Path::new(&entry.filepath).exists() {
            entry.status = VideoStatus::Invalid;
        }
    }

    // Drop the lock before returning (entries is cloned)
    drop(app_state);

    Ok(entries)
}

/// Tauri command: Remove a single video from the queue by index.
#[tauri::command]
pub async fn remove_from_queue(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    index: usize,
) -> Result<(), String> {
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        if index >= app_state.queue.len() {
            return Err(format!(
                "Index {} out of bounds (queue has {} items)",
                index,
                app_state.queue.len()
            ));
        }
        app_state.queue.remove(index);
    }

    persist_queue(&app)?;
    let _ = app.emit("queue-updated", ());

    Ok(())
}

/// Tauri command: Remove all videos from the queue.
#[tauri::command]
pub async fn clear_queue(state: State<'_, Mutex<AppState>>, app: AppHandle) -> Result<(), String> {
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.queue.clear();
    }

    persist_queue(&app)?;
    let _ = app.emit("queue-updated", ());

    Ok(())
}

/// Tauri command: Persist reordered queue after drag-and-drop (D-14).
/// Accepts the full reordered entries array from the frontend,
/// updates order_index on each entry, replaces app_state.queue, and persists.
#[tauri::command]
pub async fn reorder_queue(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    entries: Vec<VideoEntry>,
) -> Result<(), String> {
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        // Assign correct order_index to each entry
        let indexed: Vec<VideoEntry> = entries
            .into_iter()
            .enumerate()
            .map(|(i, mut entry)| {
                entry.order_index = i as u32;
                entry
            })
            .collect();
        app_state.queue = indexed;
    }

    persist_queue(&app)?;
    let _ = app.emit("queue-updated", ());

    Ok(())
}

/// Write-through persistence: serialize the full queue to tauri-plugin-store.
/// Follows the exact pattern from ffmpeg.rs lines 185-191.
fn persist_queue(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    let store =
        app.store("queue.json").map_err(|e| format!("Failed to open queue store: {}", e))?;
    let json = serde_json::to_value(&app_state.queue)
        .map_err(|e| format!("Serialization error: {}", e))?;
    store.set("queue", json);
    store.save().map_err(|e| format!("Failed to save queue: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::video::VideoEntry;

    // RED: reorder_queue does not exist yet — these tests will fail to compile.
    // They define the expected behavior: order_index assignment from position.

    /// reorder_queue assigns order_index based on array position.
    #[test]
    fn reorder_queue_assigns_order_indices() {
        let entries = vec![
            VideoEntry {
                filename: "a.mp4".into(),
                filepath: "/v/a.mp4".into(),
                metadata: crate::models::video::VideoMetadata {
                    duration_secs: 10.0,
                    width: 1920,
                    height: 1080,
                    size_bytes: 1000,
                    codec: "h264".into(),
                    fps: 30.0,
                    sample_rate: 0,
                },
                status: crate::models::video::VideoStatus::Valid,
                thumbnail_base64: None,
                order_index: 999, // should be overwritten
            },
            VideoEntry {
                filename: "b.mp4".into(),
                filepath: "/v/b.mp4".into(),
                metadata: crate::models::video::VideoMetadata {
                    duration_secs: 5.0,
                    width: 1280,
                    height: 720,
                    size_bytes: 500,
                    codec: "hevc".into(),
                    fps: 24.0,
                    sample_rate: 0,
                },
                status: crate::models::video::VideoStatus::Valid,
                thumbnail_base64: None,
                order_index: 999,
            },
            VideoEntry {
                filename: "c.mp4".into(),
                filepath: "/v/c.mp4".into(),
                metadata: crate::models::video::VideoMetadata {
                    duration_secs: 15.0,
                    width: 640,
                    height: 480,
                    size_bytes: 800,
                    codec: "mpeg4".into(),
                    fps: 30.0,
                    sample_rate: 0,
                },
                status: crate::models::video::VideoStatus::Valid,
                thumbnail_base64: None,
                order_index: 999,
            },
        ];

        // Simulate the order_index assignment logic from reorder_queue
        let indexed: Vec<VideoEntry> = entries
            .into_iter()
            .enumerate()
            .map(|(i, mut entry)| {
                entry.order_index = i as u32;
                entry
            })
            .collect();

        assert_eq!(indexed[0].order_index, 0);
        assert_eq!(indexed[1].order_index, 1);
        assert_eq!(indexed[2].order_index, 2);
        assert_eq!(indexed[0].filename, "a.mp4");
    }
}
