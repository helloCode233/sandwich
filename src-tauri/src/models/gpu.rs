use serde::{Deserialize, Serialize};

/// Detected GPU encoder, or None if only CPU is available.
/// Serialized to frontend for status display (not for user selection per D-06).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuEncoder {
    /// macOS: h264_videotoolbox or hevc_videotoolbox
    VideoToolbox,
    /// Windows NVIDIA: h264_nvenc or hevc_nvenc, with detected capabilities
    Nvenc(NvencCaps),
    /// Windows AMD: h264_amf or hevc_amf
    Amf,
    /// Linux: h264_vaapi or hevc_vaapi
    Vaapi,
}

/// NVENC encoder capabilities detected at startup via `ffmpeg -h encoder=h264_nvenc`.
///
/// Turing (20-series, NVENC 6th gen) and newer support spatial AQ, temporal AQ,
/// p1-p7 presets, and b-frames. These produce significantly better quality than
/// the baseline named presets available on Pascal (10-series, NVENC 4th gen).
///
/// Detection is automatic — no user configuration needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvencCaps {
    /// NVENC SDK ≥9.0 (driver 456+, Maxwell 2nd gen+) — spatial adaptive quantization
    pub has_spatial_aq: bool,
    /// NVENC SDK ≥11.0 (driver 520+, Turing+) — p1-p7 preset names
    pub has_presets_p: bool,
    /// NVENC SDK ≥9.0 (driver 456+) — b-frames for better compression
    pub has_bf: bool,
}

impl NvencCaps {
    /// Baseline: NVENC SDK 7.0 (driver 418, Kepler) — only named presets + VBR CQ.
    #[cfg(target_os = "windows")]
    pub fn baseline() -> Self {
        Self { has_spatial_aq: false, has_presets_p: false, has_bf: false }
    }
}

impl GpuEncoder {
    /// Return encoder-specific FFmpeg arguments including codec, preset, and quality.
    ///
    /// NVENC driver compatibility tiers:
    ///   - Baseline (10-series / Pascal, driver 418+): `-preset medium -rc vbr -cq 23`
    ///   - Enhanced (20-series+ / Turing+, driver 520+): `-preset p4 -spatial-aq 1
    ///     -temporal-aq 1 -bf 3` — better detail via adaptive quantization,
    ///     more efficient H.264 via b-frames.
    ///
    /// If any NVENC param is unsupported, batch.rs D-05 auto-retries with CPU.
    pub fn encoder_args(&self) -> Vec<String> {
        match self {
            Self::Nvenc(caps) => {
                let mut args = vec!["-c:v".to_string(), "h264_nvenc".to_string()];
                // Preset: p4 for Turing+, medium for Pascal
                if caps.has_presets_p {
                    args.push("-preset".to_string());
                    args.push("p4".to_string());
                } else {
                    args.push("-preset".to_string());
                    args.push("medium".to_string());
                }
                args.push("-rc".to_string());
                args.push("vbr".to_string());
                args.push("-cq".to_string());
                args.push("23".to_string());
                // Turing+ optimizations
                if caps.has_spatial_aq {
                    args.push("-spatial-aq".to_string());
                    args.push("1".to_string());
                    args.push("-temporal-aq".to_string());
                    args.push("1".to_string());
                }
                if caps.has_bf {
                    args.push("-bf".to_string());
                    args.push("3".to_string());
                }
                args
            }
            Self::Amf => vec![
                "-c:v".to_string(),
                "h264_amf".to_string(),
                "-quality".to_string(),
                "balanced".to_string(),
                "-rc".to_string(),
                "cqp".to_string(),
                "-qp_i".to_string(),
                "23".to_string(),
                "-qp_p".to_string(),
                "23".to_string(),
            ],
            Self::VideoToolbox => vec![
                "-c:v".to_string(),
                "h264_videotoolbox".to_string(),
                "-realtime".to_string(),
                "0".to_string(),
            ],
            Self::Vaapi => vec![
                "-c:v".to_string(),
                "h264_vaapi".to_string(),
                "-compression_level".to_string(),
                "1".to_string(),
            ],
        }
    }
}
