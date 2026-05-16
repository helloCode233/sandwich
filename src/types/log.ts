// Mirrors Rust struct ProcessingLogEntry in src-tauri/src/models/batch.rs
// All field names use camelCase matching #[serde(rename_all = "camelCase")]

/** Processing log entry persisted to store for the history panel (D-16 / PROD-03).
 *  Accumulated in Pinia logStore; capped at 500 most recent entries. */
export interface ProcessingLogEntry {
  /** Unique identifier (UUID v4). */
  id: string;
  /** ISO 8601 timestamp of processing completion. */
  timestamp: string;
  /** Source filename (not full path). */
  file: string;
  /** Seed alias used for this processing run. */
  seedAlias: string;
  /** "success" or "failure". */
  status: 'success' | 'failure';
  /** MD5 hash before processing (hex string). */
  md5Before: string;
  /** MD5 hash after processing (hex string). */
  md5After: string;
  /** true if md5Before !== md5After (file was modified by processing). */
  modified: boolean;
  /** Output file path, or null if processing failed before output. */
  outputPath: string | null;
  /** Error message, or null if processing succeeded. */
  errorMessage: string | null;
  /** Processing duration in milliseconds. */
  durationMs: number;
}
