use serde::{Deserialize, Serialize};

/// Configuration for a batch processing run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchConfig {
    /// ID of the seed to apply.
    pub seed_id: String,
    /// Output directory path.
    pub output_dir: String,
    /// Concurrency level (1-4 per D-08).
    pub concurrency: u32,
}

/// Live progress state emitted to the frontend during batch processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    /// Total number of files in the batch.
    pub total: usize,
    /// Number of files processed so far (succeeded + failed).
    pub completed: usize,
    /// Number of files processed successfully.
    pub succeeded: usize,
    /// Number of files that failed.
    pub failed: usize,
    /// Name of the file currently being processed, if any.
    pub current_file: Option<String>,
}

/// Final result returned when a batch completes or is cancelled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    /// Output file paths for successfully processed files.
    pub succeeded: Vec<String>,
    /// Error details for failed files.
    pub failed: Vec<FileResult>,
}

/// Per-file error result for failure isolation (D-11).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResult {
    /// Display filename that failed.
    pub file: String,
    /// Seed alias used.
    pub seed: String,
    /// Human-readable error message.
    pub error: String,
}

/// Per-file frame-level progress emitted during FFmpeg execution.
/// Emitted via "batch-file-progress" event from executor.rs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerFileProgress {
    /// Display filename being processed.
    pub file: String,
    /// Percentage complete for this file (0.0 - 100.0).
    pub percent: f64,
    /// Current frame number being encoded (from ffmpeg-sidecar FfmpegProgress.frame).
    pub current_frame: u32,
    /// Total frames in this file (computed from VideoEntry.metadata.duration_secs * fps).
    pub total_frames: u32,
    /// Frames per second encoding speed (from ffmpeg-sidecar FfmpegProgress.fps).
    pub fps: f32,
    /// Estimated remaining seconds for this file (computed from (total_duration - current_time) / speed).
    pub remaining_seconds: f64,
}
