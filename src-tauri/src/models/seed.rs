use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify OperationType has exactly 30 variants (20 existing + 10 new in Phase 7).
    #[test]
    fn operation_type_has_30_variants() {
        let variants = &[
            OperationType::MathOverlay,
            OperationType::PixelShift,
            OperationType::FrameDrop,
            OperationType::GopModify,
            OperationType::MetadataErase,
            OperationType::AudioTweak,
            OperationType::Remux,
            OperationType::HueRotate,
            OperationType::SaturationAdjust,
            OperationType::BrightnessContrast,
            OperationType::ColorBalance,
            OperationType::FilmGrain,
            OperationType::GaussianBlur,
            OperationType::Sharpen,
            OperationType::MicroRotate,
            OperationType::TinyScale,
            OperationType::Flip,
            OperationType::SolidColorOverlay,
            OperationType::GradientOverlay,
            OperationType::WatermarkBlend,
            // Phase 7: Audio (5)
            OperationType::AudioResample,
            OperationType::AudioVolume,
            OperationType::AudioPitch,
            OperationType::AudioEQ,
            OperationType::AudioChannel,
            // Phase 7: Crop (1)
            OperationType::Crop,
            // Phase 7: Metadata (2)
            OperationType::MetadataWrite,
            OperationType::MetadataSelectiveErase,
            // Phase 7: Duration (2)
            OperationType::VideoSpeed,
            OperationType::TrimEdges,
        ];
        assert_eq!(variants.len(), 30, "OperationType must have exactly 30 variants");
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
            schema_version: 3,
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
    /// Strength tier used when generating this seed (D-07).
    /// #[serde(default)] ensures old seeds without this field deserialize as Standard.
    #[serde(default)]
    pub strength_tier: StrengthTier,
    /// Schema version for migration tracking. Incremented per phase.
    /// Phase 6 = 2, Phase 7 = 3. Old seeds without this field default to 0.
    #[serde(default)]
    pub schema_version: u32,
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

/// Three-tier strength preset for seed generation (D-03, D-07).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StrengthTier {
    Conservative,
    Standard,
    Aggressive,
}

impl Default for StrengthTier {
    fn default() -> Self {
        StrengthTier::Standard
    }
}

/// The 30 operation types covering all fingerprint modification categories.
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
    // Color processing (4): D-01, D-02
    /// Adjust hue angle by +/- degrees.
    HueRotate,
    /// Adjust saturation multiplier.
    SaturationAdjust,
    /// Adjust brightness and contrast.
    BrightnessContrast,
    /// Rebalance color channels.
    ColorBalance,
    // Noise texture (3): D-01, D-02
    /// Apply synthetic film grain.
    FilmGrain,
    /// Apply Gaussian blur kernel.
    GaussianBlur,
    /// Apply unsharp mask / sharpen filter.
    Sharpen,
    // Geometric fine-tuning (3): D-01, D-02
    /// Rotate by sub-degree angle (0.1-0.9 degrees).
    MicroRotate,
    /// Scale by tiny factor (0.98x-1.02x).
    TinyScale,
    /// Horizontal or vertical flip.
    Flip,
    // Blend overlay (3): D-01, D-02
    /// Overlay semi-transparent solid color.
    SolidColorOverlay,
    /// Overlay gradient ramp.
    GradientOverlay,
    /// Blend transparent watermark pattern.
    WatermarkBlend,
    // Phase 7: Audio operations (5) — replace AudioTweak's 3 sub-effects (D-01, D-02, D-03)
    /// Resample audio to random rate 22050-48000 Hz.
    AudioResample,
    /// Adjust volume by +/-3 dB (D-02).
    AudioVolume,
    /// Pitch shift via asetrate+atempo chain, +/-2 semitones (D-02).
    AudioPitch,
    /// Parametric EQ at random frequency (D-02).
    AudioEQ,
    /// Channel remapping (swap, mono mixdown, etc.) (D-02).
    AudioChannel,
    // Phase 7: Crop (1) — default operation (D-04, D-05, D-06, D-07, D-08)
    /// Asymmetric crop (0.5%-3.5% per side, tier-driven) then scale back to original resolution.
    Crop,
    // Phase 7: Metadata (2) — supplement existing MetadataErase (D-09, D-10, D-11, D-12, D-13)
    /// Write fake metadata fields (creation_time, title, author, comment, copyright, encoder).
    MetadataWrite,
    /// Selectively erase metadata by category (time/device/description). Requires ffprobe context.
    MetadataSelectiveErase,
    // Phase 7: Duration (2) (D-14, D-15, D-16)
    /// Video speed change (setpts for video + atempo for audio, synchronized), 0.95-1.05x.
    VideoSpeed,
    /// Trim head/tail frames (1-30 frames from start, end, or both).
    TrimEdges,
}
