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
