use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify ProcessingLogEntry has all 11 fields serializing with camelCase.
    #[test]
    fn processing_log_entry_camel_case_serialization() {
        let entry = ProcessingLogEntry {
            id: "log-1".into(),
            timestamp: "2026-05-16T00:00:00Z".into(),
            file: "video.mp4".into(),
            seed_alias: "test-seed".into(),
            status: "success".into(),
            md5_before: "abc123".into(),
            md5_after: "def456".into(),
            modified: true,
            output_path: Some("/out/video_processed.mp4".into()),
            error_message: None,
            duration_ms: 4200,
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"id\":\"log-1\""));
        assert!(json.contains("\"timestamp\":\"2026-05-16T00:00:00Z\""));
        assert!(json.contains("\"file\":\"video.mp4\""));
        assert!(json.contains("\"seedAlias\":\"test-seed\""));
        assert!(json.contains("\"status\":\"success\""));
        assert!(json.contains("\"md5Before\":\"abc123\""));
        assert!(json.contains("\"md5After\":\"def456\""));
        assert!(json.contains("\"modified\":true"));
        assert!(json.contains("\"outputPath\":\"/out/video_processed.mp4\""));
        assert!(json.contains("\"durationMs\":4200"));
        // errorMessage should NOT be present when None
        assert!(!json.contains("errorMessage"));
    }

    /// Verify ProcessingLogEntry round-trips through serde_json.
    #[test]
    fn processing_log_entry_round_trip() {
        let entry = ProcessingLogEntry {
            id: "log-2".into(),
            timestamp: "2026-05-16T12:00:00Z".into(),
            file: "fail.mp4".into(),
            seed_alias: "bad-seed".into(),
            status: "failure".into(),
            md5_before: "aaa".into(),
            md5_after: "aaa".into(),
            modified: false,
            output_path: None,
            error_message: Some("FFmpeg crashed".into()),
            duration_ms: 100,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: ProcessingLogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "log-2");
        assert_eq!(parsed.status, "failure");
        assert_eq!(parsed.error_message, Some("FFmpeg crashed".into()));
        assert_eq!(parsed.output_path, None);
        assert_eq!(parsed.duration_ms, 100);
    }
}

/// Configuration for a batch processing run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
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
    /// Successfully processed file outputs with MD5 integrity comparison.
    pub succeeded: Vec<FileSuccess>,
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

/// Detailed success result with MD5 integrity information (Phase 5: MD5-01, MD5-02).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSuccess {
    /// Output file path.
    pub path: String,
    /// Seed alias used for this output.
    pub seed_alias: String,
    /// Input file path (for correlation).
    pub source_file: String,
    /// MD5 hash before processing (hex string, or "N/A" if hash failed).
    pub md5_before: String,
    /// MD5 hash after processing (hex string, or "N/A" if hash failed).
    pub md5_after: String,
    /// true if md5_before != md5_after (file was modified by processing).
    pub modified: bool,
    /// File size before processing (bytes).
    pub size_bytes: u64,
}

/// Persisted processing log entry for the history panel (D-16 / PROD-03).
/// Accumulated in a separate processing-log.json store file (max 500 entries).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingLogEntry {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// ISO 8601 timestamp of completion.
    pub timestamp: String,
    /// Source filename (not full path).
    pub file: String,
    /// Seed alias used for this processing run.
    pub seed_alias: String,
    /// "success" or "failure".
    pub status: String,
    /// MD5 hash before processing (hex string).
    pub md5_before: String,
    /// MD5 hash after processing (hex string).
    pub md5_after: String,
    /// true if md5_before != md5_after (file was modified).
    pub modified: bool,
    /// Output file path (null if failed before output).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    /// Error message (null if success).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Processing duration in milliseconds.
    pub duration_ms: u64,
}

/// Per-file frame-level progress emitted during FFmpeg execution.
/// Emitted via "batch-file-progress" event from executor.rs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerFileProgress {
    /// Display filename being processed.
    pub file: String,
    /// Which seed is producing this output (Phase 5: MULTI-01, MULTI-02).
    pub seed_alias: String,
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

/// Compute MD5 hash of a file via streaming I/O (Phase 5: MD5-01, MD5-02).
/// Uses an 8KB buffer to avoid loading the entire file into memory.
pub fn file_md5(path: &Path) -> Result<String, String> {
    use md5::Context;
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Cannot open file for MD5: {}", e))?;
    let mut hasher = Context::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| format!("MD5 read error: {}", e))?;
        if n == 0 {
            break;
        }
        hasher.consume(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
