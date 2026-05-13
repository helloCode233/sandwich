use serde::{Deserialize, Serialize};

/// A seed recipe: a named collection of operations applied to a video.
/// Maps to a TypeScript `Seed` interface in the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seed {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// User-assigned alias. Auto-generated as timestamp on creation per D-04.
    pub alias: String,
    /// Ordered list of operations to apply. 3-7 steps per D-03.
    pub operations: Vec<Operation>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
}

/// A single operation step within a seed's operation chain.
/// Format per SEED-03: [op type] + [start frame] + [duration frames] + [params]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    /// Type of operation.
    #[serde(rename = "opType")]
    pub op_type: OperationType,
    /// Starting frame for this operation (0 = from beginning).
    pub start_frame: u32,
    /// Number of frames to apply (0 = until end of video).
    pub duration_frames: u32,
    /// Type-specific parameters as a JSON object.
    pub params: serde_json::Value,
}

/// The 7 operation types per SEED-02.
/// D-02: MathOverlay has highest weight (~30%) in random generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    /// Mathematical overlay: ripple, stripes, or concentric patterns.
    MathOverlay,
    /// Pixel shift: crop + pad with small offset.
    PixelShift,
    /// Frame drop: remove frames at interval.
    FrameDrop,
    /// GOP modification: change keyframe interval.
    GopModify,
    /// Metadata erasure: strip metadata.
    MetadataErase,
    /// Audio tweak: volume, tempo, echo.
    AudioTweak,
    /// Remux: change container format.
    Remux,
}
