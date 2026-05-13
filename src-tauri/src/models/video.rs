use serde::{Deserialize, Serialize};

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
