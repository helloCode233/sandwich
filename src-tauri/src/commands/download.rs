use ffmpeg_sidecar::download::unpack_ffmpeg;
use reqwest::header::{HeaderValue, RANGE};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::commands::ffmpeg::verify_ffmpeg;

/// Progress event payload emitted to the frontend.
/// Maps to the `DownloadProgress` TypeScript interface in src/types/ffmpeg.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub percent: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub stage: DownloadStage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStage {
    Connecting,
    Downloading,
    Extracting,
    Verifying,
    Complete,
    Error,
}

/// Global download state for cancellation and status tracking.
/// Uses OnceLock for lazy initialization, Mutex for safe concurrent access.
#[allow(dead_code)]
struct GlobalDownloadState {
    cancel_flag: AtomicBool,
    temp_file_path: Option<PathBuf>,
    downloaded_bytes: u64,
    retry_count: u32,
    error_message: Option<String>,
}

static DOWNLOAD_STATE: OnceLock<Mutex<GlobalDownloadState>> = OnceLock::new();

fn get_download_state() -> &'static Mutex<GlobalDownloadState> {
    DOWNLOAD_STATE.get_or_init(|| {
        Mutex::new(GlobalDownloadState {
            cancel_flag: AtomicBool::new(false),
            temp_file_path: None,
            downloaded_bytes: 0,
            retry_count: 0,
            error_message: None,
        })
    })
}

/// Tauri command: Start downloading FFmpeg + FFprobe to the user-selected directory.
///
/// Downloads:
/// 1. Archives from platform-specific URL groups (with mirror fallback between groups)
/// 2. On macOS x86_64, downloads BOTH ffmpeg and ffprobe from evermeet.cx (D-16: both required)
/// 3. Emits 'ffmpeg-download-progress' events during download and extraction
/// 4. Extracts using ffmpeg-sidecar's `unpack_ffmpeg()`
/// 5. On macOS: runs `xattr -dr com.apple.quarantine` on extracted binaries
/// 6. Calls `verify_ffmpeg` to validate and persist
///
/// Per D-16: macOS x86_64 downloads BOTH ffmpeg.zip AND ffprobe.zip into same target dir.
/// Per D-17: progress shows percent, downloaded/total size, and speed.
/// Per D-20: 3 retry attempts, then error with manual download instructions.
/// Per D-21: GitHub Releases primary (Linux/Windows), osxexperts primary (macOS aarch64),
///           evermeet primary (macOS x86_64 with paired downloads), DIFFERENT-domain mirrors.
/// Per D-26: if temp file exists from prior attempt, resume via Range header.
/// Per D-27: cancelable via `cancel_download` command.
/// Per D-28: macOS quarantine removed via xattr.
#[tauri::command]
pub async fn start_download(app: AppHandle, target_dir: String) -> Result<(), String> {
    // Reset download state
    {
        let state = get_download_state().lock().await;
        state.cancel_flag.store(false, Ordering::SeqCst);
    }

    let target_path = PathBuf::from(&target_dir);
    std::fs::create_dir_all(&target_path)
        .map_err(|e| format!("Cannot create directory {}: {}", target_dir, e))?;

    // Platform-specific URL groups (each group is an alternative; URLs within a group are ALL required)
    let url_groups = select_download_urls();
    let temp_dir = std::env::temp_dir().join("sandwich-ffmpeg-download");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Cannot create temp directory: {}", e))?;

    // Emit initial Connecting progress so frontend has real data immediately
    let _ = app.emit(
        "ffmpeg-download-progress",
        DownloadProgress {
            percent: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0,
            stage: DownloadStage::Connecting,
        },
    );

    // Try each URL group in the chain with up to 3 retries per group
    let max_retries = 3;
    let mut last_error = String::new();

    for (group_idx, url_group) in url_groups.iter().enumerate() {
        let is_last_group = group_idx == url_groups.len() - 1;

        for attempt in 1..=max_retries {
            // Check cancellation before each attempt
            {
                let state = get_download_state().lock().await;
                if state.cancel_flag.load(Ordering::SeqCst) {
                    cleanup_temp(&temp_dir, &state.temp_file_path).await;
                    return Err("Download cancelled by user".to_string());
                }
            }

            // Download ALL URLs in this group (D-16: on macOS x86_64 both are required)
            let mut all_succeeded = true;
            for (url_idx, url) in url_group.iter().enumerate() {
                let app_clone = app.clone();
                let target_clone = target_path.clone();
                let temp_clone = temp_dir.clone();

                match download_single(&app_clone, url, &target_clone, &temp_clone, url_idx, attempt)
                    .await
                {
                    Ok(_) => {
                        // This URL in the group succeeded; continue to next URL
                    }
                    Err(e) => {
                        last_error = format!(
                            "{} [group {} url {}/{}]",
                            e,
                            group_idx + 1,
                            url_idx + 1,
                            url_group.len()
                        );
                        all_succeeded = false;
                        break; // Group failed, try next attempt or next group
                    }
                }
            }

            if all_succeeded && !url_group.is_empty() {
                // Emit extracting progress
                let _ = app.emit(
                    "ffmpeg-download-progress",
                    DownloadProgress {
                        percent: 95.0,
                        downloaded_bytes: 0,
                        total_bytes: 0,
                        speed_bytes_per_sec: 0,
                        stage: DownloadStage::Extracting,
                    },
                );

                // macOS: remove quarantine attribute (D-28)
                #[cfg(target_os = "macos")]
                {
                    let ffmpeg_bin = target_path.join("ffmpeg");
                    let ffprobe_bin = target_path.join("ffprobe");
                    for bin in [&ffmpeg_bin, &ffprobe_bin] {
                        if bin.exists() {
                            let output = std::process::Command::new("/usr/bin/xattr")
                                .args(["-dr", "com.apple.quarantine"])
                                .arg(bin)
                                .output();
                            if let Err(e) = output {
                                eprintln!("Warning: xattr failed on {:?}: {}", bin, e);
                            }
                        }
                    }
                }

                // Emit verifying progress
                let _ = app.emit(
                    "ffmpeg-download-progress",
                    DownloadProgress {
                        percent: 98.0,
                        downloaded_bytes: 0,
                        total_bytes: 0,
                        speed_bytes_per_sec: 0,
                        stage: DownloadStage::Verifying,
                    },
                );

                // Verify and persist — verify_ffmpeg checks for ffmpeg binary inside.
                let verify_path = target_path.to_string_lossy().to_string();

                match verify_ffmpeg(app.clone(), verify_path).await {
                    Ok(_info) => {
                        // Cleanup
                        cleanup_temp(&temp_dir, &None).await;
                        let _ = app.emit(
                            "ffmpeg-download-progress",
                            DownloadProgress {
                                percent: 100.0,
                                downloaded_bytes: 0,
                                total_bytes: 0,
                                speed_bytes_per_sec: 0,
                                stage: DownloadStage::Complete,
                            },
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        last_error = format!("Verification failed: {}", e);
                        // Fall through to retry/next group
                    }
                }
            }

            if attempt < max_retries {
                // Brief delay before retry
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        if is_last_group {
            break;
        }
        // Try next mirror group
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    // All groups exhausted, all retries failed
    let manual_url = get_manual_download_url();
    let error_msg = format!(
        "Download failed after trying {} source groups and {} retries each.\n\
         Last error: {}\n\n\
         Please download FFmpeg manually from:\n\
         {}\n\n\
         Extract ffmpeg and ffprobe binaries to: {}",
        url_groups.len(),
        max_retries,
        last_error,
        manual_url,
        target_dir,
    );

    let _ = app.emit(
        "ffmpeg-download-progress",
        DownloadProgress {
            percent: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0,
            stage: DownloadStage::Error,
        },
    );

    Err(error_msg)
}

/// Download a single URL attempt. Returns the extracted directory path.
async fn download_single(
    app: &AppHandle,
    url: &str,
    target_dir: &std::path::Path,
    temp_dir: &std::path::Path,
    url_idx: usize,
    attempt: u32,
) -> Result<String, String> {
    let archive_name = url.rsplit('/').next().unwrap_or("ffmpeg-archive").to_string();
    let archive_path = temp_dir.join(&archive_name);

    // Check for partial download (resume support, D-26)
    let mut downloaded_offset: u64 = 0;
    if archive_path.exists()
        && let Ok(metadata) = std::fs::metadata(&archive_path)
    {
        downloaded_offset = metadata.len();
    }

    // Build reqwest client
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5 min connect timeout
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let mut request = client.get(url);

    // If we have a partial file, set Range header for resume (D-26)
    if downloaded_offset > 0 {
        let range_value = format!("bytes={}-", downloaded_offset);
        request = request.header(RANGE, HeaderValue::from_str(&range_value).unwrap());
    }

    let response = request.send().await.map_err(|e| {
        format!("Network error [source {} attempt {}]: {}", url_idx + 1, attempt, e)
    })?;

    let status = response.status();
    // Accept 200 (full download) or 206 (partial content for resume)
    if status != reqwest::StatusCode::OK && status != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(format!(
            "Server returned {} for {} (source {} attempt {})",
            status,
            url,
            url_idx + 1,
            attempt
        ));
    }

    // Determine total size
    let total_bytes = if status == reqwest::StatusCode::PARTIAL_CONTENT {
        // Range response: Content-Range header has total size
        response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split('/').next_back())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    } else {
        response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    };

    // Check available disk space (simple heuristic)
    if total_bytes > 0
        && let Ok(available) = fs2::available_space(temp_dir)
        && available < total_bytes + 100_000_000
    {
        // Need 100MB extra for extraction
        return Err(format!(
            "Insufficient disk space. Required: {} MB, Available: {} MB",
            (total_bytes + 100_000_000) / 1_000_000,
            available / 1_000_000
        ));
    }

    // Stream the download with progress tracking
    let mut downloaded = downloaded_offset;
    let mut last_progress_emit = std::time::Instant::now();
    let start_time = std::time::Instant::now();

    use tokio::io::AsyncWriteExt;

    let file = if downloaded_offset > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(&archive_path)
            .await
            .map_err(|e| format!("Cannot open file for resume: {}", e))?
    } else {
        tokio::fs::File::create(&archive_path)
            .await
            .map_err(|e| format!("Cannot create temp file: {}", e))?
    };

    let mut file_writer = tokio::io::BufWriter::new(file);
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;

    while let Some(chunk_result) = stream.next().await {
        // Check cancellation
        {
            let state = get_download_state().lock().await;
            if state.cancel_flag.load(Ordering::SeqCst) {
                return Err("Download cancelled".to_string());
            }
        }

        let chunk = chunk_result.map_err(|e| format!("Download stream error: {}", e))?;

        file_writer.write_all(&chunk).await.map_err(|e| format!("Write error: {}", e))?;

        downloaded += chunk.len() as u64;

        // Emit progress at most every 100ms to avoid flooding events
        let now = std::time::Instant::now();
        if now.duration_since(last_progress_emit).as_millis() >= 100 {
            let elapsed = now.duration_since(start_time).as_secs_f64();
            let speed = if elapsed > 0.0 {
                ((downloaded - downloaded_offset) as f64 / elapsed) as u64
            } else {
                0
            };
            let percent = if total_bytes > 0 {
                // 0-90% for download, 90-100% for extract/verify
                downloaded as f64 / total_bytes as f64 * 90.0
            } else {
                // No Content-Length: show indeterminate progress (increment slowly up to 50%)
                (50.0 * (1.0 - (-elapsed / 60.0).exp())).min(50.0)
            };

            let _ = app.emit(
                "ffmpeg-download-progress",
                DownloadProgress {
                    percent,
                    downloaded_bytes: downloaded,
                    total_bytes,
                    speed_bytes_per_sec: speed,
                    stage: DownloadStage::Downloading,
                },
            );
            last_progress_emit = now;
        }
    }

    file_writer.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    drop(file_writer);

    // Track temp file for resume on next startup
    {
        let mut state = get_download_state().lock().await;
        state.temp_file_path = Some(archive_path.clone());
        state.downloaded_bytes = downloaded;
    }

    // Extract using ffmpeg-sidecar's unpack_ffmpeg (handles tar.xz, zip, permissions)
    let _ = app.emit(
        "ffmpeg-download-progress",
        DownloadProgress {
            percent: 92.0,
            downloaded_bytes: downloaded,
            total_bytes,
            speed_bytes_per_sec: 0,
            stage: DownloadStage::Extracting,
        },
    );

    unpack_ffmpeg(&archive_path, target_dir).map_err(|e| {
        format!(
            "Extraction failed: {}. The archive may be corrupted. Try deleting {} and retrying.",
            e,
            archive_path.display()
        )
    })?;

    // Delete the archive after successful extraction
    let _ = std::fs::remove_file(&archive_path);

    {
        let mut state = get_download_state().lock().await;
        state.temp_file_path = None;
    }

    Ok(target_dir.to_string_lossy().to_string())
}

/// Tauri command: Cancel an active download.
/// Per D-27: cleans up temp files, returns to initial state.
#[tauri::command]
pub async fn cancel_download() -> Result<(), String> {
    let state = get_download_state().lock().await;
    state.cancel_flag.store(true, Ordering::SeqCst);

    // Clean temp files
    if let Some(ref temp_path) = state.temp_file_path {
        let _ = std::fs::remove_file(temp_path);
    }
    let temp_dir = std::env::temp_dir().join("sandwich-ffmpeg-download");
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(())
}

/// Return the platform-specific download URL groups (primary + mirror chain).
///
/// Returns `Vec<Vec<String>>` where each inner Vec is a "download group":
/// - Groups are tried as alternatives (first group that fully succeeds wins)
/// - Within a group, ALL URLs must be downloaded successfully (D-16 on macOS x86_64)
///
/// Per D-21: GitHub Releases primary (Linux/Windows) with jsDelivr mirror.
///           macOS uses osxexperts.net (aarch64) or evermeet.cx (x86_64).
///           Mirrors use DIFFERENT domains (not duplicates of the primary).
///
/// Per D-16: macOS x86_64 evermeet.cx provides separate ffmpeg.zip and ffprobe.zip.
///           Both are in the same group (both required); extracted to same target dir.
///
/// Per RESEARCH.md Pitfall 1: BtbN provides NO macOS builds.
/// - Linux/Windows: BtbN GitHub Releases primary, jsDelivr CDN mirror
/// - macOS aarch64: osxexperts.net primary, evermeet.cx mirror (DIFFERENT domain per D-21)
/// - macOS x86_64: evermeet.cx paired [ffmpeg.zip, ffprobe.zip] primary, osxexperts Intel build mirror
fn select_download_urls() -> Vec<Vec<String>> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return vec![
            // Primary: evermeet.cx paired downloads (D-16: BOTH ffmpeg and ffprobe required)
            vec![
                "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip".to_string(),
                "https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip".to_string(),
            ],
            // Mirror: osxexperts.net ARM build (single archive, DIFFERENT domain)
            vec!["https://www.osxexperts.net/ffmpeg80arm.zip".to_string()],
        ];
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return vec![
            // Primary: evermeet.cx paired downloads (D-16: BOTH ffmpeg and ffprobe required)
            vec![
                "https://evermeet.cx/ffmpeg/ffmpeg-7.1.1.zip".to_string(),
                "https://evermeet.cx/ffmpeg/ffprobe-7.1.1.zip".to_string(),
            ],
            // Mirror: osxexperts.net Intel build (single archive with both binaries, DIFFERENT domain)
            vec!["https://www.osxexperts.net/ffmpeg80intel.zip".to_string()],
        ];
    }

    #[cfg(target_os = "linux")]
    {
        let arch = if cfg!(target_arch = "aarch64") { "linuxarm64" } else { "linux64" };
        return vec![
            // GitHub Releases (BtbN) -- provides both ffmpeg and ffprobe in one archive
            vec![format!(
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-{}-gpl-shared.tar.xz",
                arch
            )],
            // jsDelivr CDN mirror (CN-friendly, DIFFERENT domain)
            vec![format!(
                "https://cdn.jsdelivr.net/gh/BtbN/FFmpeg-Builds@latest/ffmpeg-master-latest-{}-gpl-shared.tar.xz",
                arch
            )],
        ];
    }

    #[cfg(target_os = "windows")]
    {
        return vec![
            // GitHub Releases (BtbN) -- Windows static build (no DLLs, self-contained .exe)
            // Per D-21: shared build avoided — ffmpeg-sidecar unpack_ffmpeg only moves
            // .exe files, leaving .dll files in the temp dir to be deleted. Static build
            // eliminates this problem by linking all codecs into the binary.
            vec![
                "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
                    .to_string(),
            ],
            // jsDelivr CDN mirror (CN-friendly, DIFFERENT domain)
            vec![
                "https://cdn.jsdelivr.net/gh/BtbN/FFmpeg-Builds@latest/ffmpeg-master-latest-win64-gpl.zip"
                    .to_string(),
            ],
        ];
    }

    // Fallback for non-standard target platforms
    #[allow(unreachable_code)]
    {
        vec![]
    }
}

/// Return the best URL for manual FFmpeg download instructions (shown after all retries fail).
fn get_manual_download_url() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "https://ffmpeg.org/download.html (select macOS build)"
    }
    #[cfg(target_os = "linux")]
    {
        "https://github.com/BtbN/FFmpeg-Builds/releases"
    }
    #[cfg(target_os = "windows")]
    {
        "https://github.com/BtbN/FFmpeg-Builds/releases"
    }
}

/// Clean up temp files. Called on cancel or after successful download.
async fn cleanup_temp(temp_dir: &PathBuf, specific_file: &Option<PathBuf>) {
    if let Some(file) = specific_file {
        let _ = std::fs::remove_file(file);
    }
    let _ = std::fs::remove_dir_all(temp_dir);
}
