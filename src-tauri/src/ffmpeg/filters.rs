//! FFmpeg filter chain builders for all 20 operation types.
//!
//! Each function takes an `Operation` reference and returns `Vec<String>` of
//! FFmpeg CLI arguments. SEED-04 safety constraints are enforced via clamping.

use crate::models::seed::{Operation, OperationType};

/// Build FFmpeg filter arguments for mathematical overlay.
/// SEED-04: opacity <= 0.15, frequency range 20-200.
pub fn build_math_overlay_filter(op: &Operation) -> Result<Vec<String>, String> {
    let pattern = op.params["pattern"].as_str().unwrap_or("ripple");
    let opacity: f64 = op.params["opacity"].as_f64().unwrap_or(0.08);
    let frequency: f64 = op.params["frequency"].as_f64().unwrap_or(80.0);

    // Clamp to safety constraints per SEED-04
    let opacity = opacity.clamp(0.01, 0.15);
    let frequency = frequency.clamp(20.0, 200.0);

    // Build geq expression based on pattern
    let expr = match pattern {
        "ripple" => format!(
            "lum='lum(X,Y)*(1+{opacity}*sin(2*PI*{freq}*X/W)*sin(2*PI*{freq}*Y/H))':cb='cb(X,Y)':cr='cr(X,Y)'",
            opacity = opacity,
            freq = frequency / 100.0
        ),
        "stripes" => format!(
            "lum='lum(X,Y)*(1+{opacity}*sin(2*PI*{freq}*X/W))':cb='cb(X,Y)':cr='cr(X,Y)'",
            opacity = opacity,
            freq = frequency / 100.0
        ),
        "concentric" => format!(
            "lum='lum(X,Y)*(1+{opacity}*sin(2*PI*{freq}*hypot(X-W/2,Y-H/2)/W))':cb='cb(X,Y)':cr='cr(X,Y)'",
            opacity = opacity,
            freq = frequency / 100.0
        ),
        _ => return Err(format!("Unknown math overlay pattern: {}", pattern)),
    };

    let filter = format!("geq={}", expr);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for pixel shift.
/// SEED-04: dx and dy must be within [-3, 3].
pub fn build_pixel_shift_filter(op: &Operation) -> Result<Vec<String>, String> {
    let dx: i32 = op.params["dx"].as_i64().unwrap_or(0) as i32;
    let dy: i32 = op.params["dy"].as_i64().unwrap_or(0) as i32;

    // Clamp to safety constraints per SEED-04
    let dx = dx.clamp(-3, 3);
    let dy = dy.clamp(-3, 3);

    let crop_x = if dx >= 0 { dx } else { 0 };
    let crop_y = if dy >= 0 { dy } else { 0 };
    let pad_x = if dx < 0 { -dx } else { 0 };
    let pad_y = if dy < 0 { -dy } else { 0 };

    let crop_filter = format!("crop=iw-{}:ih-{}:{}:{}", dx.abs(), dy.abs(), crop_x, crop_y);
    let pad_filter = format!("pad=iw+{}:ih+{}:{}:{}", dx.abs(), dy.abs(), pad_x, pad_y);

    Ok(vec!["-vf".to_string(), format!("{},{}", crop_filter, pad_filter)])
}

/// Build FFmpeg filter arguments for frame dropping.
/// SEED-04: interval >= 15.
pub fn build_frame_drop_filter(op: &Operation) -> Result<Vec<String>, String> {
    let interval: u32 = op.params["interval"].as_u64().unwrap_or(30) as u32;

    // Clamp to safety constraint per SEED-04
    let interval = interval.max(15);

    Ok(vec!["-vf".to_string(), format!("framestep={}", interval)])
}

/// Build FFmpeg arguments for GOP modification.
/// Sets keyframe interval via -g parameter.
pub fn build_gop_modify_filter(op: &Operation) -> Result<Vec<String>, String> {
    let gop_size: u32 = op.params["gopSize"].as_u64().unwrap_or(60) as u32;

    // Clamp to reasonable range
    let gop_size = gop_size.clamp(12, 250);

    Ok(vec!["-g".to_string(), gop_size.to_string()])
}

/// Build FFmpeg arguments for metadata erasure.
pub fn build_metadata_erase_filter(_op: &Operation) -> Result<Vec<String>, String> {
    Ok(vec![
        "-map_metadata".to_string(),
        "-1".to_string(),
        "-map_chapters".to_string(),
        "-1".to_string(),
    ])
}

/// Build FFmpeg filter arguments for audio tweaking.
pub fn build_audio_tweak_filter(op: &Operation) -> Result<Vec<String>, String> {
    let effect = op.params["effect"].as_str().unwrap_or("volume");

    match effect {
        "volume" => {
            let db: f64 = op.params["db"].as_f64().unwrap_or(0.5);
            let db = db.clamp(-2.0, 2.0);
            Ok(vec!["-af".to_string(), format!("volume={}dB", db)])
        }
        "tempo" => {
            let factor: f64 = op.params["factor"].as_f64().unwrap_or(1.01);
            let factor = factor.clamp(0.98, 1.02);
            Ok(vec!["-af".to_string(), format!("atempo={:.3}", factor)])
        }
        "echo" => Ok(vec!["-af".to_string(), "aecho=0.8:0.9:20:0.1".to_string()]),
        _ => Err(format!("Unknown audio tweak effect: {}", effect)),
    }
}

/// Build FFmpeg arguments for remuxing (no re-encode).
/// Sets -c copy for stream copy mode.
pub fn build_remux_filter(_op: &Operation) -> Result<Vec<String>, String> {
    Ok(vec!["-c".to_string(), "copy".to_string()])
}

/// Dispatch to the correct filter builder based on OperationType.
pub fn build_filter_args(op: &Operation) -> Result<Vec<String>, String> {
    match op.op_type {
        OperationType::MathOverlay => build_math_overlay_filter(op),
        OperationType::PixelShift => build_pixel_shift_filter(op),
        OperationType::FrameDrop => build_frame_drop_filter(op),
        OperationType::GopModify => build_gop_modify_filter(op),
        OperationType::MetadataErase => build_metadata_erase_filter(op),
        OperationType::AudioTweak => build_audio_tweak_filter(op),
        OperationType::Remux => build_remux_filter(op),
        // Phase 6 new operation types — filter builders to be implemented in plan 06-02.
        _ => Err(format!("Filter not yet implemented for operation type: {:?}", op.op_type)),
    }
}

/// Classifies a filter argument by how it should be merged into the final command.
pub enum FilterKind {
    /// Video filter expression (without -vf prefix), to be comma-joined with others.
    VideoFilter(String),
    /// Audio filter expression (without -af prefix), to be comma-joined with others.
    AudioFilter(String),
    /// Non-filter arguments passed through directly to FFmpeg.
    Other(Vec<String>),
}

/// Like `build_filter_args` but separates video/audio filter expressions from other args.
/// This allows the executor to merge multiple -vf / -af into single comma-joined chains
/// and resolve conflicts between -c copy (remux) and filtering operations.
pub fn build_filter_args_separated(op: &Operation) -> Result<(FilterKind, Vec<String>), String> {
    match op.op_type {
        OperationType::MathOverlay => {
            let args = build_math_overlay_filter(op)?;
            // args = ["-vf", "geq=..."]
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::PixelShift => {
            let args = build_pixel_shift_filter(op)?;
            // args = ["-vf", "crop=...,pad=..."]
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::FrameDrop => {
            let args = build_frame_drop_filter(op)?;
            // args = ["-vf", "framestep=..."]
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::AudioTweak => {
            let args = build_audio_tweak_filter(op)?;
            // args = ["-af", "volume=..."] or similar
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::AudioFilter(expr), args))
        }
        _ => {
            // GopModify, MetadataErase, Remux — pass through as Other
            let args = build_filter_args(op)?;
            Ok((FilterKind::Other(args.clone()), args))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::seed::{Operation, OperationType};

    fn make_op(op_type: OperationType, params: serde_json::Value) -> Operation {
        Operation { op_type, start_frame: 0, duration_frames: 0, params }
    }

    #[test]
    fn test_math_overlay_ripple() {
        let op = make_op(
            OperationType::MathOverlay,
            serde_json::json!({
                "pattern": "ripple",
                "opacity": 0.1,
                "frequency": 80.0
            }),
        );
        let args = build_math_overlay_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("geq="));
        assert!(args[1].contains("cb='cb(X,Y)'"));
        assert!(args[1].contains("cr='cr(X,Y)'"));
        assert!(args[1].contains("ripple") || args[1].contains("sin"));
    }

    #[test]
    fn test_math_overlay_clamps_opacity() {
        let op = make_op(
            OperationType::MathOverlay,
            serde_json::json!({
                "pattern": "ripple",
                "opacity": 0.5,
                "frequency": 80.0
            }),
        );
        let args = build_math_overlay_filter(&op).unwrap();
        assert!(args[1].contains("0.15"));
    }

    #[test]
    fn test_pixel_shift_clamps_dx() {
        let op = make_op(
            OperationType::PixelShift,
            serde_json::json!({
                "dx": 10,
                "dy": 0
            }),
        );
        let args = build_pixel_shift_filter(&op).unwrap();
        let combined = &args[1];
        assert!(combined.contains("-3") || combined.contains("iw-3"));
    }

    #[test]
    fn test_frame_drop_min_interval() {
        let op = make_op(
            OperationType::FrameDrop,
            serde_json::json!({
                "interval": 2
            }),
        );
        let args = build_frame_drop_filter(&op).unwrap();
        assert!(args[1].contains("framestep=15"));
    }

    #[test]
    fn test_gop_modify_range() {
        let op = make_op(
            OperationType::GopModify,
            serde_json::json!({
                "gopSize": 300
            }),
        );
        let args = build_gop_modify_filter(&op).unwrap();
        assert!(args[1] == "250");
    }

    #[test]
    fn test_metadata_erase() {
        let op = make_op(OperationType::MetadataErase, serde_json::json!({}));
        let args = build_metadata_erase_filter(&op).unwrap();
        assert!(args.contains(&"-map_metadata".to_string()));
        assert!(args.contains(&"-1".to_string()));
    }

    #[test]
    fn test_remux() {
        let op = make_op(OperationType::Remux, serde_json::json!({}));
        let args = build_remux_filter(&op).unwrap();
        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"copy".to_string()));
    }

    #[test]
    fn test_dispatch_all_types() {
        let types = [
            OperationType::MathOverlay,
            OperationType::PixelShift,
            OperationType::FrameDrop,
            OperationType::GopModify,
            OperationType::MetadataErase,
            OperationType::AudioTweak,
            OperationType::Remux,
        ];
        for t in &types {
            let op = make_op(
                *t,
                serde_json::json!({"pattern": "ripple", "opacity": 0.08, "frequency": 80.0, "dx": 0, "dy": 0, "interval": 30, "gopSize": 60, "effect": "volume", "db": 0.5}),
            );
            let result = build_filter_args(&op);
            assert!(result.is_ok(), "Failed for {:?}: {:?}", t, result.err());
        }
    }
}
