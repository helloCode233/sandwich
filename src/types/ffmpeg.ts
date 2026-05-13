// Mirrors Rust `FfmpegInfo` struct in src-tauri/src/commands/ffmpeg.rs
export interface FfmpegInfo {
  found: boolean;
  path: string | null;
  version: string | null;
  outdated: boolean;
  needsDownload: boolean;
}

// Mirrors Rust `DownloadProgress` struct in src-tauri/src/commands/download.rs
export interface DownloadProgress {
  percent: number;
  downloadedBytes: number;
  totalBytes: number;
  speedBytesPerSec: number;
  stage: DownloadStage;
}

// Mirrors Rust `DownloadStage` enum
export type DownloadStage =
  | 'connecting'
  | 'downloading'
  | 'extracting'
  | 'verifying'
  | 'complete'
  | 'error';

// UI state machine status (D-34)
export type FfmpegStatus =
  | 'detecting'
  | 'found'
  | 'missing'
  | 'outdated'
  | 'selecting-dir'
  | 'downloading'
  | 'verifying'
  | 'verified'
  | 'error';
