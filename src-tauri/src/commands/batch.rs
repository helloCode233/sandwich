//! Batch processing commands with cancellation and failure isolation.
//!
//! Provides 3 Tauri commands for the batch processing lifecycle:
//! - `start_batch` — processes all queued videos with a seed
//! - `cancel_batch` — signals cancellation via global AtomicBool
//! - `get_batch_status` — returns live BatchProgress

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tauri::{AppHandle, Emitter, State};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex as TokioMutex;

use crate::ffmpeg::executor::execute_single_file;
use crate::models::batch::{BatchProgress, BatchResult, FileResult};
use crate::state::{AppState, BatchStatus};

/// Global batch cancel flag, modeled after download.rs's DOWNLOAD_STATE pattern.
/// OnceLock ensures lazy init; TokioMutex allows async access from commands.
static BATCH_CANCEL: OnceLock<TokioMutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

fn get_cancel_storage() -> &'static TokioMutex<Option<Arc<AtomicBool>>> {
    BATCH_CANCEL.get_or_init(|| TokioMutex::new(None))
}

/// Read the stored FFmpeg directory from ffmpeg-config.json.
fn get_ffmpeg_dir(app: &AppHandle) -> Option<String> {
    if let Ok(store) = app.store("ffmpeg-config.json") {
        if let Some(value) = store.get("ffmpeg_path") {
            if let Some(path_str) = value.as_str() {
                return Some(path_str.to_string());
            }
        }
    }
    None
}

/// Read concurrency preference from store (D-08, D-09).
/// Returns 1 as default if unset or invalid.
fn get_concurrency_preference(app: &AppHandle) -> u32 {
    if let Ok(store) = app.store("sandwich-config.json") {
        if let Some(value) = store.get("concurrency") {
            if let Some(n) = value.as_u64() {
                let n = n as u32;
                if (1..=4).contains(&n) {
                    return n;
                }
            }
        }
    }
    1 // Default per D-08
}

/// Expand a leading tilde in a path to the user's home directory.
/// Rust's Path/PathBuf and OS syscalls do not expand ~ — only shells do.
fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Ok(home) = std::env::var("HOME") {
            return path.replacen('~', &home, 1);
        }
    }
    path.to_string()
}

/// Persist output directory to sandwich-config.json (D-05, D-09).
fn persist_output_dir(app: &AppHandle, dir: &str) -> Result<(), String> {
    let store = app
        .store("sandwich-config.json")
        .map_err(|e| format!("Failed to open config store: {}", e))?;
    store.set("output_dir", serde_json::Value::String(dir.to_string()));
    store
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(())
}

/// Tauri command: Start batch processing all queued videos with the given seed.
///
/// Per D-08: concurrency 1-4, default 1 if not in config.
/// Per D-10: cancel flag stored in global static for cross-command access.
/// Per D-11: single-file failure isolation -- one failure continues to next file.
/// Per OUTPUT-02: output naming via executor.rs (collision-safe per D-16).
#[tauri::command]
pub async fn start_batch(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    seed_id: String,
    output_dir: String,
) -> Result<(), String> {
    // Check batch is not already running
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let batch_state = app_state
            .batch_state
            .lock()
            .map_err(|e| format!("Batch state lock error: {}", e))?;
        if batch_state.status != BatchStatus::Idle {
            return Err(
                "A batch is already in progress. Cancel it first or wait for completion."
                    .to_string(),
            );
        }
    }

    // Get the stored FFmpeg directory
    let ffmpeg_dir = get_ffmpeg_dir(&app)
        .ok_or_else(|| "FFmpeg is not configured. Please set up FFmpeg first.".to_string())?;

    // Get concurrency preference (D-08, D-09)
    let _concurrency = get_concurrency_preference(&app);

    // D-10: Initialize global cancel flag -- fresh for each batch
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut storage = get_cancel_storage().lock().await;
        *storage = Some(cancel_flag.clone());
    }

    // Resolve seed and snapshot queue (clone out of Mutex before FFmpeg)
    let (seed, queue_snapshot) = {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let seed = app_state
            .seeds
            .iter()
            .find(|s| s.id == seed_id)
            .cloned()
            .ok_or_else(|| format!("Seed not found: {}", seed_id))?;
        let queue = app_state.queue.clone();
        (seed, queue)
    };

    if queue_snapshot.is_empty() {
        // Clear cancel flag before returning error
        let mut storage = get_cancel_storage().lock().await;
        *storage = None;
        return Err("Queue is empty. Import videos before starting batch processing.".to_string());
    }

    // Initialize batch state (Pitfall 3: drop locks before FFmpeg spawn)
    let initial_progress;
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut batch_state = app_state
            .batch_state
            .lock()
            .map_err(|e| format!("Batch state lock error: {}", e))?;
        batch_state.status = BatchStatus::Running;
        batch_state.progress = BatchProgress {
            total: queue_snapshot.len(),
            completed: 0,
            succeeded: 0,
            failed: 0,
            current_file: None,
        };
        initial_progress = batch_state.progress.clone();
    }

    // Emit initial progress so frontend shows "0/n" immediately (not "0/1")
    let _ = app.emit("batch-progress", initial_progress);

    // Expand tilde in output_dir (Rust Path does not expand ~)
    let output_dir = expand_tilde(&output_dir);

    // Persist output directory preference (D-05)
    persist_output_dir(&app, &output_dir)?;

    // Process files sequentially (D-11: single-file failure isolation)
    let mut succeeded_files: Vec<String> = Vec::new();
    let mut failed_files: Vec<FileResult> = Vec::new();

    for entry in &queue_snapshot {
        // D-10: Check cancellation before each file (Pitfall 5: SeqCst)
        if cancel_flag.load(Ordering::SeqCst) {
            break;
        }

        // Update current file in progress (Pitfall 3: drop locks before FFmpeg)
        {
            let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
            let mut batch_state = app_state
                .batch_state
                .lock()
                .map_err(|e| format!("Batch state lock error: {}", e))?;
            batch_state.progress.current_file = Some(entry.filename.clone());
        }

        // Execute FFmpeg -- cancel_flag passed as &AtomicBool (Arc deref, no lock needed)
        // D-11: single-file failure isolation -- Result handling continues loop
        match execute_single_file(&app, entry, &seed, &ffmpeg_dir, &output_dir, &cancel_flag) {
            Ok(output_path) => {
                succeeded_files.push(output_path);
                let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
                let mut batch_state = app_state
                    .batch_state
                    .lock()
                    .map_err(|e| format!("Batch state lock error: {}", e))?;
                batch_state.progress.succeeded += 1;
                batch_state.progress.completed += 1;
            }
            Err(e) => {
                // D-11: Single-file failure -- log and continue
                let _ = app.emit(
                    "batch-file-error",
                    FileResult {
                        file: entry.filename.clone(),
                        seed: seed.alias.clone(),
                        error: e.clone(),
                    },
                );
                failed_files.push(FileResult {
                    file: entry.filename.clone(),
                    seed: seed.alias.clone(),
                    error: e,
                });
                let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
                let mut batch_state = app_state
                    .batch_state
                    .lock()
                    .map_err(|e| format!("Batch state lock error: {}", e))?;
                batch_state.progress.failed += 1;
                batch_state.progress.completed += 1;
            }
        }

        // Emit progress to frontend
        {
            let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
            let batch_state = app_state
                .batch_state
                .lock()
                .map_err(|e| format!("Batch state lock error: {}", e))?;
            let _ = app.emit("batch-progress", batch_state.progress.clone());
        }
    }

    // Batch complete -- check if cancelled
    let was_cancelled = cancel_flag.load(Ordering::SeqCst);

    // Reset batch state
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut batch_state = app_state
            .batch_state
            .lock()
            .map_err(|e| format!("Batch state lock error: {}", e))?;
        batch_state.status = BatchStatus::Idle;
        batch_state.progress.current_file = None;
    }

    // Clear global cancel flag
    {
        let mut storage = get_cancel_storage().lock().await;
        *storage = None;
    }

    let result = BatchResult {
        succeeded: succeeded_files,
        failed: failed_files,
    };

    if was_cancelled {
        let _ = app.emit("batch-cancelled", result);
    } else {
        let _ = app.emit("batch-complete", result);
    }

    Ok(())
}

/// Tauri command: Cancel an in-progress batch.
///
/// Per D-10: sets the global cancel flag. The processing loop checks this
/// flag between files; the executor checks it mid-FFmpeg iteration.
/// Completed files are preserved; in-progress files are cleaned up by executor.
#[tauri::command]
pub async fn cancel_batch(state: State<'_, Mutex<AppState>>, app: AppHandle) -> Result<(), String> {
    // Verify batch is running
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut batch_state = app_state
            .batch_state
            .lock()
            .map_err(|e| format!("Batch state lock error: {}", e))?;

        if batch_state.status != BatchStatus::Running {
            return Err("No batch is currently running.".to_string());
        }

        batch_state.status = BatchStatus::Cancelling;
    }

    // D-10: Set the global cancel flag -- batch loop and executor read this
    {
        let storage = get_cancel_storage().lock().await;
        if let Some(ref flag) = *storage {
            flag.store(true, Ordering::SeqCst);
        }
    }

    let _ = app.emit("batch-cancelling", ());

    Ok(())
}

/// Tauri command: Open a directory in the system file manager.
#[tauri::command]
pub fn open_file_manager(path: String) -> Result<(), String> {
    let expanded = if path.starts_with('~') {
        std::env::var("HOME")
            .map(|home| path.replacen('~', &home, 1))
            .unwrap_or(path)
    } else {
        path
    };
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&expanded)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&expanded)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&expanded)
            .spawn()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
    }
    Ok(())
}

/// Tauri command: Get current batch processing status.
/// Returns the live BatchProgress even during processing.
#[tauri::command]
pub async fn get_batch_status(state: State<'_, Mutex<AppState>>) -> Result<BatchProgress, String> {
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let batch_state = app_state
        .batch_state
        .lock()
        .map_err(|e| format!("Batch state lock error: {}", e))?;
    Ok(batch_state.progress.clone())
}
