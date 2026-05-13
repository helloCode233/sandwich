// Mirrors Rust structs in src-tauri/src/models/seed.rs
// All field names use camelCase matching #[serde(rename_all = "camelCase")]
export interface Seed {
  id: string;
  alias: string;
  operations: Operation[];
  createdAt: string; // ISO 8601 — serde: created_at -> createdAt
}

export interface Operation {
  opType: OperationType; // explicit #[serde(rename = "opType")] on Rust op_type field
  startFrame: number; // serde: start_frame -> startFrame
  durationFrames: number; // serde: duration_frames -> durationFrames
  params: Record<string, unknown>; // serde_json::Value
}

export type OperationType =
  | 'mathOverlay' // Rust: MathOverlay -> camelCase -> mathOverlay
  | 'pixelShift' // Rust: PixelShift -> camelCase -> pixelShift
  | 'frameDrop'
  | 'gopModify'
  | 'metadataErase'
  | 'audioTweak'
  | 'remux';
