use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::version::{ffmpeg_version, ffmpeg_version_with_path};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;

/// Returned to the frontend after FFmpeg detection.
/// Maps to the `FfmpegInfo` TypeScript interface in src/types/ffmpeg.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegInfo {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub outdated: bool,
    pub needs_download: bool,
}

/// Persisted FFmpeg configuration stored in tauri-plugin-store.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct FfmpegConfig {
    pub ffmpeg_path: String,
    pub version: String,
    pub download_time: String, // ISO 8601 timestamp
}

/// D-25: Emitted to frontend when a newer FFmpeg release is found on GitHub.
/// Maps to the `FfmpegUpdateInfo` TypeScript interface consumed by the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
}

/// Core detection logic shared between the Tauri command and setup hook.
/// Checks: (1) cached store path, (2) PATH via ffmpeg-sidecar, (3) version >= 4.0.
pub async fn detect_ffmpeg_internal() -> FfmpegInfo {
    // First, try cached path from store (can't access store without AppHandle here,
    // so we rely on PATH detection; the command variant with store access is below)
    if ffmpeg_is_installed() {
        match ffmpeg_version() {
            Ok(version_str) => {
                let major_version = extract_major_version(&version_str);
                if major_version >= 4 {
                    FfmpegInfo {
                        found: true,
                        path: std::env::var("PATH").ok(),
                        version: Some(version_str),
                        outdated: false,
                        needs_download: false,
                    }
                } else {
                    FfmpegInfo {
                        found: true,
                        path: std::env::var("PATH").ok(),
                        version: Some(version_str),
                        outdated: true,
                        needs_download: true,
                    }
                }
            }
            Err(_) => FfmpegInfo {
                found: false,
                path: None,
                version: None,
                outdated: false,
                needs_download: true,
            },
        }
    } else {
        FfmpegInfo {
            found: false,
            path: None,
            version: None,
            outdated: false,
            needs_download: true,
        }
    }
}

/// Extract major version number from ffmpeg version string (e.g., "6.1.1" -> 6).
fn extract_major_version(version: &str) -> u32 {
    version
        .split_whitespace()
        .find(|s| s.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .and_then(|s| s.split('.').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Tauri command: Detect FFmpeg in PATH (and check cached store path).
/// Corresponds to frontend `invoke('detect_ffmpeg')`.
/// Per D-19: PATH first, then prompt download.
/// Per D-22: Version < 4.0 is treated as outdated and prompts download.
#[tauri::command]
pub async fn detect_ffmpeg(app: AppHandle) -> Result<FfmpegInfo, String> {
    // Check store for previously downloaded FFmpeg path
    if let Ok(store) = app.store("ffmpeg-config.json")
        && let Some(cached_path) = store.get("ffmpeg_path")
        && let Some(path_str) = cached_path.as_str()
    {
        let cached_path = PathBuf::from(path_str);
        if cached_path.join("ffmpeg").exists() || cached_path.join("ffmpeg.exe").exists() {
            match ffmpeg_version_with_path(path_str) {
                Ok(version_str) => {
                    let major = extract_major_version(&version_str);
                    return Ok(FfmpegInfo {
                        found: true,
                        path: Some(path_str.to_string()),
                        version: Some(version_str),
                        outdated: major < 4,
                        needs_download: major < 4,
                    });
                }
                Err(_) => {
                    // Cached binary is broken, fall through to PATH check
                }
            }
        }
    }

    // Fallback to PATH detection
    Ok(detect_ffmpeg_internal().await)
}

/// Tauri command: Get current FFmpeg status (reads from store).
/// Used by frontend to check if FFmpeg is configured without re-running detection.
#[tauri::command]
pub async fn get_ffmpeg_status(app: AppHandle) -> Result<FfmpegInfo, String> {
    if let Ok(store) = app.store("ffmpeg-config.json")
        && let Some(cached_path) = store.get("ffmpeg_path")
        && let Some(path_str) = cached_path.as_str()
    {
        let cached_path = PathBuf::from(path_str);
        let ffmpeg_bin = if cfg!(target_os = "windows") {
            cached_path.join("ffmpeg.exe")
        } else {
            cached_path.join("ffmpeg")
        };
        if ffmpeg_bin.exists() {
            match ffmpeg_version_with_path(path_str) {
                Ok(version_str) => {
                    let major = extract_major_version(&version_str);
                    return Ok(FfmpegInfo {
                        found: true,
                        path: Some(path_str.to_string()),
                        version: Some(version_str),
                        outdated: major < 4,
                        needs_download: major < 4,
                    });
                }
                Err(_e) => {
                    return Ok(FfmpegInfo {
                        found: false,
                        path: Some(path_str.to_string()),
                        version: None,
                        outdated: false,
                        needs_download: true,
                    });
                }
            }
        }
    }
    Ok(detect_ffmpeg_internal().await)
}

/// Tauri command: Verify a downloaded FFmpeg binary and persist its path.
/// Called after download + extraction completes (from download.rs).
/// Per D-23: auto verifies with `ffmpeg -version`.
/// Per D-24: persists ffmpeg_path, version, download_time to store.
/// Per D-28: on macOS, quarantine is removed BEFORE this call (handled in download.rs).
#[tauri::command]
pub async fn verify_ffmpeg(app: AppHandle, path: String) -> Result<FfmpegInfo, String> {
    let version_str = ffmpeg_version_with_path(&path)
        .map_err(|e| format!("FFmpeg verification failed at {}: {}", path, e))?;

    let major = extract_major_version(&version_str);
    let now = chrono::Utc::now().to_rfc3339();

    // Persist to store
    let store =
        app.store("ffmpeg-config.json").map_err(|e| format!("Failed to open store: {}", e))?;
    store.set("ffmpeg_path", serde_json::Value::String(path.clone()));
    store.set("version", serde_json::Value::String(version_str.clone()));
    store.set("download_time", serde_json::Value::String(now));
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;

    // Emit event that FFmpeg is ready
    let info = FfmpegInfo {
        found: true,
        path: Some(path),
        version: Some(version_str),
        outdated: major < 4,
        needs_download: major < 4,
    };
    let _ = app.emit("ffmpeg-ready", info.clone());

    Ok(info)
}

/// D-25: Check GitHub for the latest FFmpeg-Builds release and compare with installed version.
///
/// Fetches the latest release tag from BtbN/FFmpeg-Builds GitHub API.
/// If the installed version is older than the latest release, returns `FfmpegUpdateInfo`
/// which is emitted to the frontend as a non-blocking notification.
///
/// The version comparison extracts the FFmpeg version string from the release tag
/// (e.g., "ffmpeg-7.1.1" -> "7.1.1") and compares it with the locally installed version.
///
/// Returns `Ok(None)` if: no FFmpeg installed, up-to-date, network error (silent failure).
/// Per D-25: non-blocking — errors are silently ignored, the app continues regardless.
pub async fn check_latest_version() -> Result<Option<FfmpegUpdateInfo>, String> {
    // Only check if FFmpeg is already installed (no point suggesting update if none exists)
    let current_version = match ffmpeg_version() {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };

    // Extract the version number from the local ffmpeg version string
    // ffmpeg-sidecar returns something like "ffmpeg version 7.1.1 Copyright ..."
    let current_ver = extract_major_version(&current_version);

    // Fetch latest release from GitHub API (BtbN/FFmpeg-Builds)
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("sandwich-app/0.1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get("https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("GitHub API request failed: {}", e))?;

    if !response.status().is_success() {
        return Ok(None); // Silent failure for non-200 (rate-limited, etc.)
    }

    #[derive(Deserialize)]
    struct GitHubRelease {
        tag_name: String,
        html_url: String,
    }

    let release: GitHubRelease =
        response.json().await.map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    // Extract version from tag (e.g., "ffmpeg-7.1.1" -> "7.1.1")
    let latest_version_str = release.tag_name.trim_start_matches("ffmpeg-").trim_start_matches("v");
    let latest_ver = extract_major_version(latest_version_str);

    if latest_ver > current_ver {
        Ok(Some(FfmpegUpdateInfo {
            current_version,
            latest_version: latest_version_str.to_string(),
            download_url: release.html_url,
        }))
    } else {
        Ok(None) // Up to date
    }
}
