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
