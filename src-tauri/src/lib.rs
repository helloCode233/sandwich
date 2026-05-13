mod commands;
mod ffmpeg;
mod models;
mod state;

use commands::download::{cancel_download, start_download};
use commands::ffmpeg::{
    check_latest_version, detect_ffmpeg, detect_ffmpeg_internal, get_ffmpeg_status, verify_ffmpeg,
};
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            detect_ffmpeg,
            get_ffmpeg_status,
            start_download,
            cancel_download,
            verify_ffmpeg,
        ])
        .setup(|app| {
            // On startup, detect FFmpeg and emit initial status to frontend
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let info = detect_ffmpeg_internal().await;
                let _ = handle.emit("ffmpeg-status", info);
            });

            // D-25: Non-blocking check for newer FFmpeg release on GitHub.
            // Runs independently of detection; failures are silent (network errors ignored).
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
