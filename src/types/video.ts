// Mirrors Rust structs in src-tauri/src/models/video.rs
// All field names use camelCase matching #[serde(rename_all = "camelCase")]
export interface VideoEntry {
  filename: string; // stays filename (one word, no transform)
  filepath: string; // stays filepath (one word, no transform)
  metadata: VideoMetadata;
  status: VideoStatus;
}

export interface VideoMetadata {
  durationSecs: number; // serde: duration_secs -> durationSecs
  width: number;
  height: number;
  sizeBytes: number; // serde: size_bytes -> sizeBytes
  codec: string;
  fps: number;
}

export type VideoStatus = 'valid' | 'invalid';
// Rust: Valid -> valid, Invalid -> invalid (camelCase on enum variants)
