//! GPU encoder detection for hardware-accelerated FFmpeg encoding.
//!
//! Detects available GPU encoders via `ffmpeg -encoders` at startup and
//! provides encoder name injection and CPU fallback logic.

use std::process::Command;

use crate::models::gpu::GpuEncoder;

/// Detect best available GPU encoder for this platform.
/// Returns None if no hardware encoder found (CPU fallback to libx264).
pub fn detect_gpu_encoder(ffmpeg_path: &str) -> Option<GpuEncoder> {
    let ffmpeg_bin = std::path::Path::new(ffmpeg_path).join(if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });

    let output = Command::new(&ffmpeg_bin).args(["-hide_banner", "-encoders"]).output().ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    #[cfg(target_os = "macos")]
    {
        if stdout.contains("h264_videotoolbox") {
            return Some(GpuEncoder::VideoToolbox);
        }
    }
    #[cfg(target_os = "windows")]
    {
        if stdout.contains("h264_nvenc") {
            return Some(GpuEncoder::Nvenc);
        }
        if stdout.contains("h264_amf") {
            return Some(GpuEncoder::Amf);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if stdout.contains("h264_vaapi") {
            return Some(GpuEncoder::Vaapi);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gpu_encoder_no_ffmpeg() {
        let result = detect_gpu_encoder("/nonexistent/path");
        assert!(result.is_none());
    }
}
