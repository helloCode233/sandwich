//! FFprobe wrapper for video metadata extraction.
//!
//! Extracts duration, resolution, codec, FPS, and file size from any video file
//! via ffprobe JSON output. Validates at least one video stream exists (D-14).

use std::path::PathBuf;
use std::process::Command;

use ffmpeg_sidecar::ffprobe::ffprobe_path;
use serde::Deserialize;

use crate::models::video::VideoMetadata;

/// Prevent spawned processes from creating a visible console window on Windows.
fn no_console_window(_cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

/// Run ffprobe on a video file and extract structured metadata.
///
/// ffmpeg_dir_opt: optional directory containing ffprobe binary.
/// If None, uses ffprobe from PATH via ffmpeg-sidecar.
/// Per D-14: validates at least one video stream exists; returns Err if not.
pub fn extract_metadata(
    filepath: &str,
    ffmpeg_dir_opt: Option<&str>,
) -> Result<VideoMetadata, String> {
    // Locate ffprobe binary
    let ffprobe_bin = match ffmpeg_dir_opt {
        Some(dir) => PathBuf::from(dir).join(if cfg!(target_os = "windows") {
            "ffprobe.exe"
        } else {
            "ffprobe"
        }),
        None => ffprobe_path(),
    };

    // Run ffprobe with JSON output
    let mut cmd = Command::new(&ffprobe_bin);
    no_console_window(&mut cmd);
    let output = cmd
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", filepath])
        .output()
        .map_err(|e| format!("ffprobe execution failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // D-14: Invalid file — reject with specific error
        return Err(format!("File is not a valid video: {}", stderr.trim()));
    }

    // Parse JSON output
    let probe: RawProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe JSON output: {}", e))?;

    // D-14: Verify at least one video stream
    let video_stream = probe
        .streams
        .iter()
        .find(|s| s.codec_type == "video")
        .ok_or_else(|| "No video stream found in file".to_string())?;

    // Extract fields
    let duration: f64 = probe.format.duration.parse().unwrap_or(0.0);
    let size: u64 = probe.format.size.parse().unwrap_or(0);
    let width = video_stream.width.unwrap_or(0);
    let height = video_stream.height.unwrap_or(0);
    let codec = video_stream.codec_name.clone().unwrap_or_else(|| "unknown".to_string());

    // Parse FPS from r_frame_rate (e.g., "30000/1001" or "30/1")
    let fps = video_stream
        .r_frame_rate
        .as_ref()
        .and_then(|r| {
            let parts: Vec<&str> = r.split('/').collect();
            if parts.len() == 2 {
                let num: f32 = parts[0].parse().ok()?;
                let den: f32 = parts[1].parse().ok()?;
                if den > 0.0 { Some(num / den) } else { None }
            } else {
                r.parse::<f32>().ok()
            }
        })
        .unwrap_or(0.0);

    // Extract audio sample rate from first audio stream (for AudioPitch).
    let sample_rate = probe
        .streams
        .iter()
        .find(|s| s.codec_type == "audio")
        .and_then(|s| s.sample_rate)
        .unwrap_or(0);

    Ok(VideoMetadata {
        duration_secs: duration,
        width,
        height,
        size_bytes: size,
        codec,
        fps,
        sample_rate,
    })
}

/// Run ffprobe to extract all global metadata tags from a video file.
/// Returns HashMap<key, value> of all metadata fields in the format.
/// Used by MetadataSelectiveErase to know which fields exist before targeted erasure.
/// This is separate from extract_metadata() because we only need it when
/// a MetadataSelectiveErase operation is present in the seed.
///
/// ffmpeg_dir_opt: optional directory containing ffprobe binary.
/// If None, uses ffprobe from PATH via ffmpeg-sidecar.
pub fn probe_global_metadata(
    filepath: &str,
    ffmpeg_dir_opt: Option<&str>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let ffprobe_bin = match ffmpeg_dir_opt {
        Some(dir) => PathBuf::from(dir).join(if cfg!(target_os = "windows") {
            "ffprobe.exe"
        } else {
            "ffprobe"
        }),
        None => ffprobe_path(),
    };
    let mut cmd = std::process::Command::new(&ffprobe_bin);
    no_console_window(&mut cmd);
    let output = cmd
        .args(["-v", "quiet", "-print_format", "json", "-show_format", filepath])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;

    if !output.status.success() {
        return Err("Cannot probe file metadata".to_string());
    }

    let probe: RawProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe JSON: {}", e))?;

    Ok(probe.format.tags.unwrap_or_default())
}

// --- Raw ffprobe JSON structures (private, used only for parsing) ---

#[derive(Debug, Deserialize)]
struct RawProbeOutput {
    format: RawFormat,
    streams: Vec<RawStream>,
}

#[derive(Debug, Deserialize)]
struct RawFormat {
    #[serde(default)]
    duration: String,
    #[serde(default)]
    size: String,
    #[serde(default)]
    tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct RawStream {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(rename = "r_frame_rate", default)]
    r_frame_rate: Option<String>,
    #[serde(rename = "sample_rate", default)]
    sample_rate: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metadata_rejects_nonexistent_file() {
        let result = extract_metadata("/nonexistent/video.mp4", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_metadata_rejects_text_file() {
        // Create a temp text file and verify it's rejected
        let tmp = std::env::temp_dir().join("sandwich_test_notavideo.txt");
        std::fs::write(&tmp, "hello world").unwrap();
        let result = extract_metadata(&tmp.to_string_lossy(), None);
        // Should fail — not a valid video
        assert!(result.is_err());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_fps_parse_30_over_1() {
        // Parse "30/1" -> 30.0
        let r = "30/1";
        let parts: Vec<&str> = r.split('/').collect();
        let num: f32 = parts[0].parse().unwrap();
        let den: f32 = parts[1].parse().unwrap();
        assert!((num / den - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_fps_parse_30000_over_1001() {
        // Parse "30000/1001" -> ~29.97
        let r = "30000/1001";
        let parts: Vec<&str> = r.split('/').collect();
        let num: f32 = parts[0].parse().unwrap();
        let den: f32 = parts[1].parse().unwrap();
        assert!((num / den - 29.97).abs() < 0.1);
    }
}
