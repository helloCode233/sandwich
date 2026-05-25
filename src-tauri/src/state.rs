use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use crate::models::batch::BatchProgress;
use crate::models::gpu::GpuEncoder;
use crate::models::seed::Seed;
use crate::models::video::VideoEntry;

/// Central managed state for the entire application.
/// Wrapped in `Mutex<AppState>` and registered via `app.manage()`.
pub struct AppState {
    /// All saved seeds.
    pub seeds: Vec<Seed>,
    /// Video processing queue.
    pub queue: Vec<VideoEntry>,
    /// Batch processing state (idle when not processing).
    pub batch_state: Mutex<BatchState>,
    /// Detected GPU encoder (None = CPU only). Set at startup via ffmpeg -encoders probe.
    /// Per D-06: used internally for auto-selection; no manual user override.
    pub gpu_encoder: Option<GpuEncoder>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            seeds: Vec::new(),
            queue: Vec::new(),
            batch_state: Mutex::new(BatchState::default()),
            gpu_encoder: None,
        }
    }
}

/// State specific to batch processing.
pub struct BatchState {
    /// Cancel flag checked between files and within FFmpeg iteration.
    /// D-10: set by cancel_batch command, checked by processing loop.
    #[allow(dead_code)]
    pub cancel_flag: AtomicBool,
    /// Current processing status.
    pub status: BatchStatus,
    /// Live progress counters.
    pub progress: BatchProgress,
}

impl Default for BatchState {
    fn default() -> Self {
        Self {
            cancel_flag: AtomicBool::new(false),
            status: BatchStatus::Idle,
            progress: BatchProgress {
                total: 0,
                completed: 0,
                succeeded: 0,
                failed: 0,
                current_file: None,
            },
        }
    }
}

/// Batch processing lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStatus {
    /// No batch in progress.
    Idle,
    /// Batch is actively processing files.
    Running,
    /// Cancel has been requested; processing is winding down.
    Cancelling,
}
