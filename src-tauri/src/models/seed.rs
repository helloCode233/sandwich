use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify OperationType has exactly 20 variants (7 existing + 13 new in Phase 6).
    #[test]
    fn operation_type_has_20_variants() {
        let variants = &[
            OperationType::MathOverlay,
            OperationType::PixelShift,
            OperationType::FrameDrop,
            OperationType::GopModify,
            OperationType::MetadataErase,
            OperationType::AudioTweak,
            OperationType::Remux,
        ];
        assert_eq!(variants.len(), 20, "OperationType must have exactly 20 variants");
    }

    /// Verify StrengthTier serializes to camelCase correctly.
    #[test]
    fn strength_tier_serialization() {
        assert_eq!(
            serde_json::to_string(&StrengthTier::Conservative).unwrap(),
            r#""conservative""#
        );
        assert_eq!(serde_json::to_string(&StrengthTier::Standard).unwrap(), r#""standard""#);
        assert_eq!(serde_json::to_string(&StrengthTier::Aggressive).unwrap(), r#""aggressive""#);
    }

    /// Verify Seed deserializes successfully when strength_tier is missing (serde default).
    #[test]
    fn seed_deserialize_missing_strength_tier() {
        let json = r#"{
            "id": "seed-1",
            "alias": "test",
            "operations": [],
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let seed: Seed = serde_json::from_str(json).unwrap();
        assert_eq!(seed.strength_tier, StrengthTier::Standard);
    }

    /// Verify Seed with strength_tier round-trips through serde_json correctly.
    #[test]
    fn seed_strength_tier_round_trip() {
        let seed = Seed {
            id: "s1".into(),
            alias: "aggro-seed".into(),
            operations: vec![],
            created_at: "2026-01-01T00:00:00Z".into(),
            strength_tier: StrengthTier::Aggressive,
        };
        let json = serde_json::to_string(&seed).unwrap();
        let parsed: Seed = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.strength_tier, StrengthTier::Aggressive);
    }
}

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
