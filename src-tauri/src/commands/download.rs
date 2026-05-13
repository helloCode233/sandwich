use serde::{Deserialize, Serialize};

/// Progress event payload emitted to the frontend.
/// Maps to the `DownloadProgress` TypeScript interface in src/types/ffmpeg.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct DownloadProgress {
    pub percent: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub stage: DownloadStage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub enum DownloadStage {
    Connecting,
    Downloading,
    Extracting,
    Verifying,
    Complete,
    Error,
}

/// Stub — full implementation in Task 2.
#[tauri::command]
pub async fn start_download(_app: tauri::AppHandle, _target_dir: String) -> Result<(), String> {
    Err("Download not yet implemented".to_string())
}

/// Stub — full implementation in Task 2.
#[tauri::command]
pub async fn cancel_download() -> Result<(), String> {
    Ok(())
}
