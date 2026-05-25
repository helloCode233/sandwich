use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify VideoEntry deserializes when new Phase 6 fields are missing (serde default).
    #[test]
    fn video_entry_deserialize_missing_new_fields() {
        let json = r#"{
            "filename": "test.mp4",
            "filepath": "/videos/test.mp4",
            "metadata": {
                "durationSecs": 10.0,
                "width": 1920,
                "height": 1080,
                "sizeBytes": 5000000,
                "codec": "h264",
                "fps": 30.0
            },
            "status": "valid"
        }"#;
        let entry: VideoEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.thumbnail_base64, None);
        assert_eq!(entry.order_index, 0);
    }

    /// Verify VideoEntry with thumbnail and order_index round-trips through serde_json.
    #[test]
    fn video_entry_thumbnail_order_round_trip() {
        let entry = VideoEntry {
            filename: "clip.mp4".into(),
            filepath: "/videos/clip.mp4".into(),
            metadata: VideoMetadata {
                duration_secs: 5.0,
                width: 1280,
                height: 720,
                size_bytes: 2000000,
                codec: "hevc".into(),
                fps: 24.0,
                sample_rate: 0,
            },
            status: VideoStatus::Valid,
            thumbnail_base64: Some("abc123base64".into()),
            order_index: 5,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: VideoEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.thumbnail_base64, Some("abc123base64".into()));
        assert_eq!(parsed.order_index, 5);
    }
}

/// A video entry in the processing queue.
/// Maps to a TypeScript `VideoEntry` interface in the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEntry {
    /// Display filename (stem only, no path).
    pub filename: String,
    /// Absolute path to the video file on disk.
    pub filepath: String,
    /// Extracted metadata from ffprobe.
    pub metadata: VideoMetadata,
    /// Validity status per D-06.
    pub status: VideoStatus,
    /// Base64-encoded JPEG thumbnail (first frame, 120px wide). None if extraction failed.
    /// D-15: extracted during import, stored for display.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail_base64: Option<String>,
    /// Display order index for drag-and-drop reordering (D-14).
    /// 0-based; persisted to maintain user's preferred queue order.
    #[serde(default)]
    pub order_index: u32,
}

/// Video metadata extracted via ffprobe.
/// Fields map directly to QUEUE-01 display requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoMetadata {
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Video width in pixels.
    pub width: u32,
    /// Video height in pixels.
    pub height: u32,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Video codec name (e.g., "h264", "hevc").
    pub codec: String,
    /// Frames per second.
    pub fps: f32,
    /// Audio sample rate in Hz. 0 if no audio stream.
    /// Used by AudioPitch to set the correct asetrate target.
    #[serde(default)]
    pub sample_rate: u32,
}

/// Validity status for a queued video entry.
/// D-06: moved/deleted files are marked Invalid and preserve metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoStatus {
    /// File exists on disk and ffprobe confirms valid video stream.
    Valid,
    /// File is missing or ffprobe reports no video stream.
    Invalid,
}
