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
