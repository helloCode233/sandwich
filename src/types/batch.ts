// Mirrors Rust structs in src-tauri/src/models/batch.rs
// All field names use camelCase matching #[serde(rename_all = "camelCase")]
export interface BatchProgress {
  total: number;
  completed: number;
  succeeded: number;
  failed: number;
  currentFile: string | null; // serde: current_file -> currentFile; Rust Option<String>
}

export interface BatchResult {
  succeeded: string[];
  failed: FileResult[];
}

export interface FileResult {
  file: string;
  seed: string;
  error: string;
}

/** Per-file frame-level progress from the Rust executor via batch-file-progress event.
 *  Mirrors Rust struct PerFileProgress in src-tauri/src/models/batch.rs */
export interface PerFileProgress {
  /** Display filename being processed. */
  file: string;
  /** Percentage complete for this file (0-100). */
  percent: number;
  /** Current frame number being encoded (from ffmpeg-sidecar FfmpegProgress.frame). */
  currentFrame: number;
  /** Total frames in this file (duration_secs * fps from ffprobe metadata). */
  totalFrames: number;
  /** Frames per second encoding speed. */
  fps: number;
  /** Estimated remaining seconds for this file (clamped to >= 0). */
  remainingSeconds: number;
}
