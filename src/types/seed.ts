// Mirrors Rust structs in src-tauri/src/models/seed.rs
// All field names use camelCase matching #[serde(rename_all = "camelCase")]
export interface Seed {
  id: string;
  alias: string;
  operations: Operation[];
  createdAt: string; // ISO 8601 — serde: created_at -> createdAt
  /** Strength tier used when generating this seed (D-07). Optional for backward compat with old seeds. */
  strengthTier?: 'conservative' | 'standard' | 'aggressive';
  /** Schema version for migration tracking. Phase 6 = 2, Phase 7 = 3. Old seeds default to 0. */
  schemaVersion?: number;
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
  | 'remux'
  // Phase 6: Color processing (4)
  | 'hueRotate'
  | 'saturationAdjust'
  | 'brightnessContrast'
  | 'colorBalance'
  // Phase 6: Noise texture (3)
  | 'filmGrain'
  | 'gaussianBlur'
  | 'sharpen'
  // Phase 6: Geometric fine-tuning (3)
  | 'microRotate'
  | 'tinyScale'
  | 'flip'
  // Phase 6: Blend overlay (3)
  | 'solidColorOverlay'
  | 'gradientOverlay'
  | 'watermarkBlend'
  // Phase 7: Audio operations (5) — replace audioTweak sub-effects
  | 'audioResample' // Rust: AudioResample -> camelCase
  | 'audioVolume' // Rust: AudioVolume
  | 'audioPitch' // Rust: AudioPitch
  | 'audioEQ' // Rust: AudioEQ
  | 'audioChannel' // Rust: AudioChannel
  // Phase 7: Crop (1) — default operation
  | 'crop' // Rust: Crop
  // Phase 7: Metadata (2) — supplement existing 'metadataErase'
  | 'metadataWrite' // Rust: MetadataWrite
  | 'metadataSelectiveErase' // Rust: MetadataSelectiveErase
  // Phase 7: Duration (2)
  | 'videoSpeed' // Rust: VideoSpeed
  | 'trimEdges'; // Rust: TrimEdges
