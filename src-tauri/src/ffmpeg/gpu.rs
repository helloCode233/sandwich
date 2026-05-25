//! GPU encoder detection for hardware-accelerated FFmpeg encoding.
//!
//! Detects available GPU encoders via `ffmpeg -encoders` at startup and
//! provides encoder name injection and CPU fallback logic.
//! For NVENC, also probes capability features (spatial AQ, p-presets, b-frames)
//! to select optimal encoding parameters for the detected GPU generation.

use std::process::Command;

use crate::models::gpu::GpuEncoder;
#[cfg(target_os = "windows")]
use crate::models::gpu::NvencCaps;

/// Prevent spawned processes from creating a visible console window on Windows.
fn no_console_window(cmd: &mut Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = cmd;
    }
}

/// Detect best available GPU encoder for this platform.
/// Returns None if no hardware encoder found (CPU fallback to libx264).
/// For NVENC, also probes and attaches capability info for optimal parameter selection.
pub fn detect_gpu_encoder(ffmpeg_dir: &str) -> Option<GpuEncoder> {
    let ffmpeg_bin = std::path::Path::new(ffmpeg_dir).join(if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });

    let mut cmd = Command::new(&ffmpeg_bin);
    no_console_window(&mut cmd);
    let output = cmd.args(["-hide_banner", "-encoders"]).output().ok()?;

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
            let caps = detect_nvenc_caps(ffmpeg_dir);
            return Some(GpuEncoder::Nvenc(caps));
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

/// Run `ffmpeg -h encoder=h264_nvenc` and parse supported capabilities.
///
/// Detection logic:
///   - `spatial_aq` in output → NVENC SDK ≥9.0 (driver 456+, Maxwell 2nd gen / 20-series+)
///   - `preset.*p1` in output → NVENC SDK ≥11.0 (driver 520+, Turing+)
///   - `bf` in output → b-frames supported
#[cfg(target_os = "windows")]
fn detect_nvenc_caps(ffmpeg_dir: &str) -> NvencCaps {
    let ffmpeg_bin = std::path::Path::new(ffmpeg_dir).join(if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });

    let mut cmd = Command::new(&ffmpeg_bin);
    no_console_window(&mut cmd);
    let output = match cmd.args(["-h", "encoder=h264_nvenc"]).output() {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return NvencCaps::baseline(),
    };

    NvencCaps {
        has_spatial_aq: output.contains("spatial_aq"),
        has_presets_p: output.contains("preset") && output.contains("p1"),
        has_bf: output.contains("bf"),
    }
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
