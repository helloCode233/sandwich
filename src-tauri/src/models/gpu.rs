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
    /// Return the FFmpeg -c:v encoder name
    pub fn encoder_name(&self) -> &str {
        match self {
            Self::VideoToolbox => "h264_videotoolbox",
            Self::Nvenc => "h264_nvenc",
            Self::Amf => "h264_amf",
            Self::Vaapi => "h264_vaapi",
        }
    }
}
