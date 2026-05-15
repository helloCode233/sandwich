mod commands;
mod ffmpeg;
mod models;
mod state;

use crate::ffmpeg::gpu::detect_gpu_encoder;
use commands::batch::{cancel_batch, get_batch_status, open_file_manager, start_batch};
use commands::download::{cancel_download, start_download};
use commands::ffmpeg::{
    check_latest_version, detect_ffmpeg, detect_ffmpeg_internal, get_default_ffmpeg_dir,
    get_ffmpeg_status, verify_ffmpeg,
};
use commands::import::import_video;
use commands::queue::{clear_queue, get_queue, remove_from_queue};
use commands::seed::{copy_seed, delete_seed, generate_seed, list_seeds, rename_seed};
use tauri::Emitter;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Phase 1 commands (unchanged)
            detect_ffmpeg,
            get_ffmpeg_status,
            start_download,
            cancel_download,
            verify_ffmpeg,
            get_default_ffmpeg_dir,
            // Phase 2: Seed commands
            generate_seed,
            rename_seed,
            delete_seed,
            copy_seed,
            list_seeds,
            // Phase 2: Import command
            import_video,
            // Phase 2: Queue commands
            get_queue,
            remove_from_queue,
            clear_queue,
            // Phase 2: Batch commands
            start_batch,
            cancel_batch,
            get_batch_status,
            open_file_manager,
        ])
        .setup(|app| {
            // --- Phase 2: Initialize managed state ---
            use std::sync::Mutex;
            app.manage(Mutex::new(state::AppState::default()));

            // Load persisted seeds and queue from store into managed state
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tauri_plugin_store::StoreExt;

                // Load persisted seeds from seeds.json
                if let Ok(store) = handle.store("seeds.json") {
                    if let Some(value) = store.get("seeds") {
                        if let Ok(seeds) = serde_json::from_value::<Vec<models::seed::Seed>>(value)
                        {
                            let app_state = handle.state::<Mutex<state::AppState>>();
                            if let Ok(mut state) = app_state.lock() {
                                state.seeds = seeds;
                            }
                        }
                    }
                }

                // Load persisted queue from queue.json
                if let Ok(store) = handle.store("queue.json") {
                    if let Some(value) = store.get("queue") {
                        if let Ok(queue) =
                            serde_json::from_value::<Vec<models::video::VideoEntry>>(value)
                        {
                            let app_state = handle.state::<Mutex<state::AppState>>();
                            if let Ok(mut state) = app_state.lock() {
                                state.queue = queue;
                            }
                        }
                    }
                }

                // Emit initial state counts to frontend
                let app_state = handle.state::<Mutex<state::AppState>>();
                if let Ok(state) = app_state.lock() {
                    let _ = handle.emit("seeds-loaded", state.seeds.len());
                    let _ = handle.emit("queue-loaded", state.queue.len());
                }
            });

            // --- Existing Phase 1: FFmpeg detection on startup (unchanged) ---
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let info = detect_ffmpeg_internal().await;
                let _ = handle.emit("ffmpeg-status", info);
            });

            // --- Phase 5: GPU encoder detection (PERF-01, D-04, D-05) ---
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let ffmpeg_path = ffmpeg_sidecar::paths::ffmpeg_path();
                let ffmpeg_dir = ffmpeg_path
                    .parent()
                    .map(|p: &std::path::Path| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let gpu_enc = detect_gpu_encoder(&ffmpeg_dir);
                if let Some(ref enc) = gpu_enc {
                    let _ = handle.emit("gpu-encoder-detected", enc);
                } else {
                    let _ = handle.emit("gpu-encoder-not-detected", ());
                }
                let app_state = handle.state::<std::sync::Mutex<state::AppState>>();
                if let Ok(mut state) = app_state.lock() {
                    state.gpu_encoder = gpu_enc;
                }
            });

            // D-25: Non-blocking check for newer FFmpeg release (unchanged)
            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(Some(update_info)) = check_latest_version().await {
                    let _ = update_handle.emit("ffmpeg-update-available", update_info);
                }
                // Network errors are silently ignored (non-blocking per D-25)
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
