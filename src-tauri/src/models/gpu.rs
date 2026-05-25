use serde::{Deserialize, Serialize};

/// Detected GPU encoder, or None if only CPU is available.
/// Serialized to frontend for status display (not for user selection per D-06).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuEncoder {
    /// macOS: h264_videotoolbox or hevc_videotoolbox
    VideoToolbox,
    /// Windows NVIDIA: h264_nvenc or hevc_nvenc
    Nvenc,
    /// Windows AMD: h264_amf or hevc_amf
    Amf,
    /// Linux: h264_vaapi or hevc_vaapi
    Vaapi,
}

impl GpuEncoder {
    /// Return encoder-specific FFmpeg arguments including codec, preset, and quality.
    ///
    /// NVENC: `-rc vbr -cq 23` gives quality-based encoding equivalent to CRF 23,
    /// avoiding the default CBR 2Mbps which produces terrible quality.
    ///
    /// NVENC driver compatibility:
    ///   - `-preset medium` (named): driver 418+ (2019), all Kepler+ GPUs
    ///   - `-preset p1-p7` (numeric): driver 520+ (2022) — NOT used, too narrow
    ///   - `-rc vbr -cq`: driver 418+, widely supported
    ///   - `-spatial-aq`/`-temporal-aq`: driver 456+ (2020), Maxwell+ — NOT used
    ///     to keep compatibility with older driver versions common on consumer machines.
    ///   If any NVENC param is unsupported, batch.rs D-05 auto-retries with CPU.
    ///
    /// AMF: `-quality balanced -rc cqp -qp_i/p 23` matches NVENC quality level.
    ///
    /// VideoToolbox: `-realtime 0` disables real-time mode for better compression.
    ///
    /// VAAPI: `-compression_level 1` enables better quality at the cost of speed.
    pub fn encoder_args(&self) -> Vec<String> {
        match self {
            Self::Nvenc => vec![
                "-c:v".to_string(),
                "h264_nvenc".to_string(),
                "-preset".to_string(),
                "medium".to_string(),
                "-rc".to_string(),
                "vbr".to_string(),
                "-cq".to_string(),
                "23".to_string(),
            ],
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
