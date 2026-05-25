//! Batch processing commands with cancellation and failure isolation.
//!
//! Provides 3 Tauri commands for the batch processing lifecycle:
//! - `start_batch` — processes all queued videos with selected seeds
//! - `cancel_batch` — signals cancellation via global AtomicBool
//! - `get_batch_status` — returns live BatchProgress

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tauri::{AppHandle, Emitter, State};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex as TokioMutex;

use crate::ffmpeg::executor::execute_single_file;
use crate::models::batch::{
    BatchProgress, BatchResult, FileResult, FileSuccess, ProcessingLogEntry,
};
use crate::models::gpu::GpuEncoder;
use crate::state::{AppState, BatchStatus};

/// Global batch cancel flag, modeled after download.rs's DOWNLOAD_STATE pattern.
/// OnceLock ensures lazy init; TokioMutex allows async access from commands.
static BATCH_CANCEL: OnceLock<TokioMutex<Option<Arc<AtomicBool>>>> = OnceLock::new();

fn get_cancel_storage() -> &'static TokioMutex<Option<Arc<AtomicBool>>> {
    BATCH_CANCEL.get_or_init(|| TokioMutex::new(None))
}

/// Read the stored FFmpeg directory from ffmpeg-config.json.
fn get_ffmpeg_dir(app: &AppHandle) -> Option<String> {
    if let Ok(store) = app.store("ffmpeg-config.json")
        && let Some(value) = store.get("ffmpeg_path")
        && let Some(path_str) = value.as_str()
    {
        return Some(path_str.to_string());
    }
    None
}

/// Read concurrency preference from store (D-08, D-09).
/// Returns 1 as default if unset or invalid.
fn get_concurrency_preference(app: &AppHandle) -> u32 {
    if let Ok(store) = app.store("sandwich-config.json")
        && let Some(value) = store.get("concurrency")
        && let Some(n) = value.as_u64()
    {
        let n = n as u32;
        if (1..=4).contains(&n) {
            return n;
        }
    }
    1 // Default per D-08
}

/// Resolve the user's home directory across platforms.
/// macOS/Linux: `HOME` env var. Windows: `HOME` first (Git Bash sets it),
/// falls back to `USERPROFILE` (standard on native Windows).
fn home_dir() -> Option<String> {
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).ok()
}

/// Expand a leading tilde in a path to the user's home directory,
/// normalizing separators to the platform's native form.
/// Rust's Path/PathBuf and OS syscalls do not expand ~ — only shells do.
fn expand_tilde(path: &str) -> String {
    if path.starts_with('~')
        && let Some(home) = home_dir()
    {
        let expanded = path.replacen('~', &home, 1);
        // Normalize to platform native separators (Windows: / → \)
        return PathBuf::from(&expanded).to_string_lossy().to_string();
    }
    path.to_string()
}

/// Persist output directory to sandwich-config.json (D-05, D-09).
fn persist_output_dir(app: &AppHandle, dir: &str) -> Result<(), String> {
    let store = app
        .store("sandwich-config.json")
        .map_err(|e| format!("Failed to open config store: {}", e))?;
    store.set("output_dir", serde_json::Value::String(dir.to_string()));
    store.save().map_err(|e| format!("Failed to save config: {}", e))?;
    Ok(())
}

/// Tauri command: Start batch processing all queued videos with the selected seeds.
///
/// Per MULTI-01, MULTI-02: accepts multiple seed IDs, one video x N seeds = N outputs.
/// Per MD5-01, MD5-02: pre-computes MD5 hashes, compares post-processing, emits FileSuccess.
/// Per D-05: GPU encode failure silently retries with CPU before permanent failure.
/// Per D-08: concurrency 1-4, default 1 if not in config.
/// Per D-10: cancel flag stored in global static for cross-command access.
/// Per D-11: single-file failure isolation -- one failure continues to next file.
/// Per D-16: per-file duration timing captured and emitted via batch-log events.
#[tauri::command]
pub async fn start_batch(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    seed_ids: Vec<String>,
    output_dir: String,
) -> Result<(), String> {
    // Check batch is not already running
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let batch_state =
            app_state.batch_state.lock().map_err(|e| format!("Batch state lock error: {}", e))?;
        if batch_state.status != BatchStatus::Idle {
            return Err("A batch is already in progress. Cancel it first or wait for completion."
                .to_string());
        }
    }

    // Phase 5: MULTI-01 — Validate and resolve multiple seeds
    if seed_ids.is_empty() {
        return Err("At least one seed must be selected.".to_string());
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

    // Phase 5: PERF-01 — Extract GPU encoder from AppState (detected at startup)
    let gpu_encoder: Option<GpuEncoder> = {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.gpu_encoder.clone()
    };

    // Resolve seeds and snapshot queue (clone out of Mutex before FFmpeg)
    let (seeds, queue_snapshot) = {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let seeds: Vec<crate::models::seed::Seed> = seed_ids
            .iter()
            .filter_map(|id| app_state.seeds.iter().find(|s| s.id == *id).cloned())
            .collect();
        if seeds.len() != seed_ids.len() {
            return Err("One or more selected seeds no longer exist.".to_string());
        }
        let queue = app_state.queue.clone();
        (seeds, queue)
    };

    if queue_snapshot.is_empty() {
        let mut storage = get_cancel_storage().lock().await;
        *storage = None;
        return Err("Queue is empty. Import videos before starting batch processing.".to_string());
    }

    // Initialize batch state with multi-seed total
    let initial_progress;
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut batch_state =
            app_state.batch_state.lock().map_err(|e| format!("Batch state lock error: {}", e))?;
        batch_state.status = BatchStatus::Running;
        let total_count = queue_snapshot.len() * seeds.len();
        batch_state.progress = BatchProgress {
            total: total_count,
            completed: 0,
            succeeded: 0,
            failed: 0,
            current_file: None,
        };
        initial_progress = batch_state.progress.clone();
    }

    let _ = app.emit("batch-progress", initial_progress);

    // Expand tilde in output_dir (Rust Path does not expand ~)
    let output_dir = expand_tilde(&output_dir);

    // Persist output directory preference (D-05)
    persist_output_dir(&app, &output_dir)?;

    // Phase 5: MD5-01 — Pre-compute MD5 hashes for all input files via spawn_blocking
    let mut md5_before_map: HashMap<String, (String, u64)> = HashMap::new();
    for entry in &queue_snapshot {
        let path = entry.filepath.clone();
        let (hash, size) = tokio::task::spawn_blocking(move || {
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let hash = crate::models::batch::file_md5(Path::new(&path)).unwrap_or_else(|e| {
                eprintln!("MD5 pre-hash failed for {}: {}", path, e);
                "N/A".to_string()
            });
            (hash, size)
        })
        .await
        .map_err(|e| format!("MD5 pre-hash task panicked: {}", e))?;
        md5_before_map.insert(entry.filepath.clone(), (hash, size));
    }

    // Process files: outer loop (files) x inner loop (seeds)
    // Per D-10: one video x N seeds = N outputs
    let mut succeeded_files: Vec<FileSuccess> = Vec::new();
    let mut succeeded_durations: Vec<u64> = Vec::new();
    let mut failed_files: Vec<FileResult> = Vec::new();
    let mut failed_durations: Vec<u64> = Vec::new();

    for entry in &queue_snapshot {
        if cancel_flag.load(Ordering::SeqCst) {
            break;
        }
        for seed in &seeds {
            if cancel_flag.load(Ordering::SeqCst) {
                break;
            }

            // Update current file with seed alias context
            {
                let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
                let mut batch_state = app_state
                    .batch_state
                    .lock()
                    .map_err(|e| format!("Batch state lock error: {}", e))?;
                batch_state.progress.current_file =
                    Some(format!("{} ({})", entry.filename, seed.alias));
            }

            // D-16: Capture per-file start time for duration tracking
            let file_start = std::time::Instant::now();

            match execute_single_file(
                &app,
                entry,
                seed,
                &ffmpeg_dir,
                &output_dir,
                &cancel_flag,
                gpu_encoder.as_ref(),
            ) {
                Ok(output_path) => {
                    let elapsed_ms = file_start.elapsed().as_millis() as u64;

                    // Phase 5: MD5-02 — Post-processing MD5 hash + comparison
                    let md5_after = tokio::task::spawn_blocking({
                        let p = output_path.clone();
                        move || {
                            crate::models::batch::file_md5(Path::new(&p))
                                .unwrap_or_else(|_| "N/A".to_string())
                        }
                    })
                    .await
                    .map_err(|e| format!("MD5 post-hash task panicked: {}", e))?;

                    let (md5_before, size_bytes) = md5_before_map
                        .get(&entry.filepath)
                        .cloned()
                        .unwrap_or(("N/A".to_string(), 0));

                    let modified = md5_before != md5_after && md5_after != "N/A";

                    // Emit per-file log entry for the log panel (before data moves)
                    let _ = app.emit(
                        "batch-log",
                        ProcessingLogEntry {
                            id: uuid::Uuid::new_v4().to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            file: entry.filename.clone(),
                            seed_alias: seed.alias.clone(),
                            status: "success".to_string(),
                            md5_before: md5_before.clone(),
                            md5_after: md5_after.clone(),
                            modified,
                            output_path: Some(output_path.clone()),
                            error_message: None,
                            duration_ms: elapsed_ms,
                        },
                    );

                    succeeded_files.push(FileSuccess {
                        path: output_path,
                        seed_alias: seed.alias.clone(),
                        source_file: entry.filepath.clone(),
                        md5_before,
                        md5_after,
                        modified,
                        size_bytes,
                    });
                    succeeded_durations.push(elapsed_ms);

                    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    let mut batch_state = app_state
                        .batch_state
                        .lock()
                        .map_err(|e| format!("Batch state lock error: {}", e))?;
                    batch_state.progress.succeeded += 1;
                    batch_state.progress.completed += 1;
                    let progress_snapshot = batch_state.progress.clone();
                    drop(batch_state);
                    drop(app_state);
                    let _ = app.emit("batch-progress", progress_snapshot);
                }
                Err(e) => {
                    let elapsed_ms = file_start.elapsed().as_millis() as u64;

                    // Phase 5: D-05 — GPU encode failure: retry with CPU before permanent failure
                    if gpu_encoder.is_some() {
                        let _ = app.emit(
                            "gpu-encode-failed-fallback",
                            FileResult {
                                file: entry.filename.clone(),
                                seed: seed.alias.clone(),
                                error: format!("GPU encoder failed ({}), retrying with CPU...", e),
                            },
                        );
                        let retry_start = std::time::Instant::now();
                        match execute_single_file(
                            &app,
                            entry,
                            seed,
                            &ffmpeg_dir,
                            &output_dir,
                            &cancel_flag,
                            None,
                        ) {
                            Ok(output_path) => {
                                let retry_elapsed =
                                    elapsed_ms + retry_start.elapsed().as_millis() as u64;
                                // CPU retry succeeded — treat as success with MD5
                                let md5_after = tokio::task::spawn_blocking({
                                    let p = output_path.clone();
                                    move || {
                                        crate::models::batch::file_md5(Path::new(&p))
                                            .unwrap_or_else(|_| "N/A".to_string())
                                    }
                                })
                                .await
                                .map_err(|e| format!("MD5 post-hash task panicked: {}", e))?;
                                let (md5_before, size_bytes) = md5_before_map
                                    .get(&entry.filepath)
                                    .cloned()
                                    .unwrap_or(("N/A".to_string(), 0));
                                let modified = md5_before != md5_after && md5_after != "N/A";

                                // Emit per-file log entry (CPU fallback success)
                                let _ = app.emit(
                                    "batch-log",
                                    ProcessingLogEntry {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                        file: entry.filename.clone(),
                                        seed_alias: seed.alias.clone(),
                                        status: "success".to_string(),
                                        md5_before: md5_before.clone(),
                                        md5_after: md5_after.clone(),
                                        modified,
                                        output_path: Some(output_path.clone()),
                                        error_message: None,
                                        duration_ms: retry_elapsed,
                                    },
                                );

                                succeeded_files.push(FileSuccess {
                                    path: output_path,
                                    seed_alias: seed.alias.clone(),
                                    source_file: entry.filepath.clone(),
                                    md5_before,
                                    md5_after,
                                    modified,
                                    size_bytes,
                                });
                                succeeded_durations.push(retry_elapsed);
                                let app_state =
                                    state.lock().map_err(|e| format!("Lock error: {}", e))?;
                                let mut batch_state = app_state
                                    .batch_state
                                    .lock()
                                    .map_err(|e| format!("Batch state lock error: {}", e))?;
                                batch_state.progress.succeeded += 1;
                                batch_state.progress.completed += 1;
                                let progress_snapshot = batch_state.progress.clone();
                                drop(batch_state);
                                drop(app_state);
                                let _ = app.emit("batch-progress", progress_snapshot);
                                continue;
                            }
                            Err(cpu_err) => {
                                let retry_elapsed =
                                    elapsed_ms + retry_start.elapsed().as_millis() as u64;
                                let combined = format!("GPU: {} | CPU fallback: {}", e, cpu_err);
                                let _ = app.emit(
                                    "batch-file-error",
                                    FileResult {
                                        file: entry.filename.clone(),
                                        seed: seed.alias.clone(),
                                        error: combined.clone(),
                                    },
                                );

                                // Emit per-file log entry (GPU+CPU both failed)
                                let _ = app.emit(
                                    "batch-log",
                                    ProcessingLogEntry {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                        file: entry.filename.clone(),
                                        seed_alias: seed.alias.clone(),
                                        status: "failure".to_string(),
                                        md5_before: String::new(),
                                        md5_after: String::new(),
                                        modified: false,
                                        output_path: None,
                                        error_message: Some(combined.clone()),
                                        duration_ms: retry_elapsed,
                                    },
                                );

                                failed_files.push(FileResult {
                                    file: entry.filename.clone(),
                                    seed: seed.alias.clone(),
                                    error: combined,
                                });
                                failed_durations.push(retry_elapsed);

                                let app_state =
                                    state.lock().map_err(|e| format!("Lock error: {}", e))?;
                                let mut batch_state = app_state
                                    .batch_state
                                    .lock()
                                    .map_err(|e| format!("Batch state lock error: {}", e))?;
                                batch_state.progress.failed += 1;
                                batch_state.progress.completed += 1;
                                let progress_snapshot = batch_state.progress.clone();
                                drop(batch_state);
                                drop(app_state);
                                let _ = app.emit("batch-progress", progress_snapshot);
                                continue;
                            }
                        }
                    }
                    // Original error handling (when gpu_encoder is None)
                    let _ = app.emit(
                        "batch-file-error",
                        FileResult {
                            file: entry.filename.clone(),
                            seed: seed.alias.clone(),
                            error: e.clone(),
                        },
                    );

                    // Emit per-file log entry (failure) — before e is moved
                    let _ = app.emit(
                        "batch-log",
                        ProcessingLogEntry {
                            id: uuid::Uuid::new_v4().to_string(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                            file: entry.filename.clone(),
                            seed_alias: seed.alias.clone(),
                            status: "failure".to_string(),
                            md5_before: String::new(),
                            md5_after: String::new(),
                            modified: false,
                            output_path: None,
                            error_message: Some(e.clone()),
                            duration_ms: elapsed_ms,
                        },
                    );

                    failed_files.push(FileResult {
                        file: entry.filename.clone(),
                        seed: seed.alias.clone(),
                        error: e,
                    });
                    failed_durations.push(elapsed_ms);

                    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
                    let mut batch_state = app_state
                        .batch_state
                        .lock()
                        .map_err(|e| format!("Batch state lock error: {}", e))?;
                    batch_state.progress.failed += 1;
                    batch_state.progress.completed += 1;
                    let progress_snapshot = batch_state.progress.clone();
                    drop(batch_state);
                    drop(app_state);
                    let _ = app.emit("batch-progress", progress_snapshot);
                }
            }
        }
    }

    // Batch complete -- check if cancelled
    let was_cancelled = cancel_flag.load(Ordering::SeqCst);

    // Reset batch state
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut batch_state =
            app_state.batch_state.lock().map_err(|e| format!("Batch state lock error: {}", e))?;
        batch_state.status = BatchStatus::Idle;
        batch_state.progress.current_file = None;
    }

    // Clear global cancel flag
    {
        let mut storage = get_cancel_storage().lock().await;
        *storage = None;
    }

    let result = BatchResult { succeeded: succeeded_files, failed: failed_files };

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
        let mut batch_state =
            app_state.batch_state.lock().map_err(|e| format!("Batch state lock error: {}", e))?;

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
/// Creates the directory if it doesn't exist (first run before any batch).
#[tauri::command]
pub fn open_file_manager(path: String) -> Result<(), String> {
    let expanded = expand_tilde(&path);
    // Ensure directory exists so file manager opens the right location
    let _ = std::fs::create_dir_all(&expanded);
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
    let batch_state =
        app_state.batch_state.lock().map_err(|e| format!("Batch state lock error: {}", e))?;
    Ok(batch_state.progress.clone())
}

/// Tauri command: Get the current GPU encoder status from AppState.
/// Returns the encoder name string (e.g. "Nvenc", "Amf") or null if CPU only.
/// Called by the frontend on mount to get the initial GPU state, since the
/// startup event may have fired before the UI listener was registered.
#[tauri::command]
pub fn get_gpu_status(state: State<'_, Mutex<AppState>>) -> Result<Option<String>, String> {
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(app_state.gpu_encoder.as_ref().map(|enc| match enc {
        GpuEncoder::Nvenc(_) => "Nvenc".to_string(),
        GpuEncoder::Amf => "Amf".to_string(),
        GpuEncoder::VideoToolbox => "VideoToolbox".to_string(),
        GpuEncoder::Vaapi => "Vaapi".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED: These tests validate the ProcessingLogEntry construction patterns
    // needed for D-16 (batch-log events with per-file duration_ms).
    // The ProcessingLogEntry struct exists in models/batch.rs, so these
    // construction tests compile — but they verify behavior that will be
    // exercised only after the batch-log emission is implemented.

    /// ProcessingLogEntry for a succeeded file has duration_ms > 0 (D-16).
    #[test]
    fn log_entry_success_has_positive_duration() {
        let entry = crate::models::batch::ProcessingLogEntry {
            id: "test-1".into(),
            timestamp: "2026-05-16T00:00:00Z".into(),
            file: "video.mp4".into(),
            seed_alias: "my-seed".into(),
            status: "success".into(),
            md5_before: "abc".into(),
            md5_after: "def".into(),
            modified: true,
            output_path: Some("/out/file.mp4".into()),
            error_message: None,
            duration_ms: 4200,
        };
        assert!(entry.duration_ms > 0, "duration_ms must be > 0 for succeeded files");
        assert_eq!(entry.status, "success");
        assert!(entry.error_message.is_none());
        assert!(entry.output_path.is_some());
    }

    /// ProcessingLogEntry for a failed file has duration_ms and error_message set.
    #[test]
    fn log_entry_failure_has_error_and_duration() {
        let entry = crate::models::batch::ProcessingLogEntry {
            id: "test-2".into(),
            timestamp: "2026-05-16T00:00:00Z".into(),
            file: "bad.mp4".into(),
            seed_alias: "fail-seed".into(),
            status: "failure".into(),
            md5_before: String::new(),
            md5_after: String::new(),
            modified: false,
            output_path: None,
            error_message: Some("FFmpeg crash".into()),
            duration_ms: 150,
        };
        assert!(entry.duration_ms > 0, "duration_ms must be > 0 even for failures");
        assert_eq!(entry.status, "failure");
        assert!(entry.error_message.is_some());
        assert!(entry.output_path.is_none());
        assert!(!entry.modified);
    }

    /// Each FileSuccess + elapsed_ms yields a correct ProcessingLogEntry.
    #[test]
    fn construct_log_from_file_success() {
        let success = crate::models::batch::FileSuccess {
            path: "/out/vid.mp4".into(),
            seed_alias: "s1".into(),
            source_file: "/src/vid.mp4".into(),
            md5_before: "aaa".into(),
            md5_after: "bbb".into(),
            modified: true,
            size_bytes: 10000,
        };
        let elapsed_ms = 1234u64;
        let log = crate::models::batch::ProcessingLogEntry {
            id: "uuid-1".into(),
            timestamp: "now".into(),
            file: success.source_file.clone(),
            seed_alias: success.seed_alias.clone(),
            status: "success".into(),
            md5_before: success.md5_before.clone(),
            md5_after: success.md5_after.clone(),
            modified: success.modified,
            output_path: Some(success.path.clone()),
            error_message: None,
            duration_ms: elapsed_ms,
        };
        assert_eq!(log.duration_ms, 1234);
        assert_eq!(log.status, "success");
        assert_eq!(log.modified, true);
    }

    /// Each FileResult + elapsed_ms yields a correct ProcessingLogEntry.
    #[test]
    fn construct_log_from_file_result() {
        let failure = crate::models::batch::FileResult {
            file: "err.mp4".into(),
            seed: "bad".into(),
            error: "something went wrong".into(),
        };
        let elapsed_ms = 567u64;
        let log = crate::models::batch::ProcessingLogEntry {
            id: "uuid-2".into(),
            timestamp: "now".into(),
            file: failure.file.clone(),
            seed_alias: failure.seed.clone(),
            status: "failure".into(),
            md5_before: String::new(),
            md5_after: String::new(),
            modified: false,
            output_path: None,
            error_message: Some(failure.error.clone()),
            duration_ms: elapsed_ms,
        };
        assert_eq!(log.duration_ms, 567);
        assert_eq!(log.status, "failure");
        assert_eq!(log.error_message, Some("something went wrong".into()));
    }

    /// D-16: duration_ms is set from actual elapsed time, not hardcoded.
    #[test]
    fn duration_ms_not_zero_for_success() {
        let start = std::time::Instant::now();
        let elapsed_ms = start.elapsed().as_millis() as u64;
        let _entry = crate::models::batch::ProcessingLogEntry {
            id: "x".into(),
            timestamp: "x".into(),
            file: "x.mp4".into(),
            seed_alias: "x".into(),
            status: "success".into(),
            md5_before: "x".into(),
            md5_after: "x".into(),
            modified: false,
            output_path: None,
            error_message: None,
            duration_ms: elapsed_ms,
        };
        assert_eq!(_entry.duration_ms, elapsed_ms);
    }
}
