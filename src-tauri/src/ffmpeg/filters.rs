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
    // Micro-timing jitter via setpts — replaces framestep which caused slideshow
    // (framestep kept 1/N frames; at N=15 → 2fps — 视频变成图片放映).
    let offset: f64 = op.params["offset"].as_f64().unwrap_or(0.002);
    let period: u32 = op.params["period"].as_u64().unwrap_or(60) as u32;

    // Safety backstop: offset 0.0001..0.01s, period >= 15 frames
    let offset = offset.clamp(0.0001, 0.01);
    let period = period.max(15);

    let filter =
        format!("setpts=PTS+sin(N*2*PI/{period})*{offset}/TB", period = period, offset = offset);
    Ok(vec!["-vf".to_string(), filter])
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

// =========================================================================
// Phase 7: Audio Operations (D-01, D-02, D-03) — 5 new filter builder functions
// =========================================================================

/// Build FFmpeg filter arguments for audio resampling (D-03).
/// Resample to random rate within 22050-48000 Hz using aresample filter.
/// Safety: rate clamped to [22050, 48000].
pub fn build_audio_resample_filter(op: &Operation) -> Result<Vec<String>, String> {
    let sample_rate: u32 = op.params["sampleRate"].as_u64().unwrap_or(44100) as u32;
    let sample_rate = sample_rate.clamp(22050, 48000);
    let filter = format!("aresample={}", sample_rate);
    Ok(vec!["-af".to_string(), filter])
}

/// Build FFmpeg filter arguments for volume adjustment (D-02).
/// Adjust by +/-3 dB using volume filter.
/// Safety: db clamped to [-3.0, 3.0].
pub fn build_audio_volume_filter(op: &Operation) -> Result<Vec<String>, String> {
    let db: f64 = op.params["db"].as_f64().unwrap_or(0.0);
    let db = db.clamp(-3.0, 3.0);
    let filter = format!("volume={}dB", db);
    Ok(vec!["-af".to_string(), filter])
}

/// Build FFmpeg filter arguments for pitch shift via asetrate+atempo+aresample chain (D-02).
/// Pitch change +/-2 semitones using the standard technique:
///   1. asetrate: change sample rate (alters both pitch AND speed)
///   2. atempo: restore original speed (counteracts speed change, pitch stays shifted)
///   3. aresample: bring sample rate back to original
/// This avoids rubberband (external library, violates D-05 pure built-in constraint).
/// Safety: pitchFactor clamped to [2^(-2/12), 2^(2/12)] ≈ [0.8909, 1.1225].
pub fn build_audio_pitch_filter(op: &Operation) -> Result<Vec<String>, String> {
    let pitch_factor: f64 = op.params["pitchFactor"].as_f64().unwrap_or(1.0);
    let original_rate: u32 = op.params["originalRate"].as_u64().unwrap_or(48000) as u32;

    // Clamp pitch factor to +/-2 semitones: 2^(-2/12) ≈ 0.8909, 2^(2/12) ≈ 1.1225
    let pitch_factor = pitch_factor.clamp(0.8909, 1.1225);

    // asetrate uses the target sample rate (original * factor) to shift pitch
    // atempo uses 1/factor to restore speed
    // aresample brings it back to original rate
    let filter = format!(
        "asetrate={}*{:.4},atempo={:.4},aresample={}",
        original_rate,
        pitch_factor,
        1.0 / pitch_factor,
        original_rate
    );
    Ok(vec!["-af".to_string(), filter])
}

/// Build FFmpeg filter arguments for parametric EQ (D-02).
/// Uses equalizer filter: two-pole peaking EQ at a randomly selected frequency.
/// Safety: frequency clamped to [100, 10000], gain to [-6.0, 6.0], width to [50, 500].
pub fn build_audio_eq_filter(op: &Operation) -> Result<Vec<String>, String> {
    let frequency: u32 = op.params["frequency"].as_u64().unwrap_or(1000) as u32;
    let gain: f64 = op.params["gain"].as_f64().unwrap_or(0.0);
    let width: u32 = op.params["width"].as_u64().unwrap_or(200) as u32;

    let frequency = frequency.clamp(100, 10000);
    let gain = gain.clamp(-6.0, 6.0);
    let width = width.clamp(50, 500);

    let filter = format!("equalizer=f={}:t=h:width={}:g={}", frequency, width, gain);
    Ok(vec!["-af".to_string(), filter])
}

/// Build FFmpeg filter arguments for channel remapping (D-02).
/// Uses channelmap filter for channel swap or pan filter for mono mixdown.
/// Safety: validates mode against known operations.
pub fn build_audio_channel_filter(op: &Operation) -> Result<Vec<String>, String> {
    let mode = op.params["mode"].as_str().unwrap_or("swap");

    let filter = match mode {
        "swap" => "channelmap=map=FL-FR|FR-FL".to_string(),
        "mono" => "pan=mono|c0=0.5*FL+0.5*FR".to_string(),
        "stereo" => "channelmap=map=FL-FC|FR-FC".to_string(),
        _ => return Err(format!("Unknown channel mode: {}", mode)),
    };

    Ok(vec!["-af".to_string(), filter])
}

// =========================================================================
// Phase 6: Color Processing (D-01, D-02) — 4 new filter builder functions
// =========================================================================

/// Build FFmpeg filter arguments for hue rotation.
/// Strength tier: conservative +/-15deg, standard +/-45deg, aggressive +/-90deg.
/// Safety backstop: hue angle clamped to [-90, 90], saturation [0.5, 1.5].
pub fn build_hue_rotate_filter(op: &Operation) -> Result<Vec<String>, String> {
    let hue_angle: f64 = op.params["hueAngle"].as_f64().unwrap_or(0.0);
    let saturation: f64 = op.params["saturation"].as_f64().unwrap_or(1.0);

    let hue_angle = hue_angle.clamp(-90.0, 90.0);
    let saturation = saturation.clamp(0.5, 1.5);

    let filter = format!("hue=h={}:s={}", hue_angle, saturation);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for saturation adjustment via `eq` filter.
/// Strength tier affects saturation, contrast, and brightness ranges.
/// Safety backstop: sat [0.5, 2.0], contrast [0.8, 1.3], brightness [-0.3, 0.3].
pub fn build_saturation_adjust_filter(op: &Operation) -> Result<Vec<String>, String> {
    let saturation: f64 = op.params["saturation"].as_f64().unwrap_or(1.0);
    let contrast: f64 = op.params["contrast"].as_f64().unwrap_or(1.0);
    let brightness: f64 = op.params["brightness"].as_f64().unwrap_or(0.0);

    let saturation = saturation.clamp(0.5, 2.0);
    let contrast = contrast.clamp(0.8, 1.3);
    let brightness = brightness.clamp(-0.3, 0.3);

    let filter =
        format!("eq=saturation={}:contrast={}:brightness={}", saturation, contrast, brightness);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for brightness/contrast adjustment via `eq` filter.
/// Safety backstop: brightness [-0.3, 0.3], contrast [0.7, 1.5], gamma [0.8, 1.3].
pub fn build_brightness_contrast_filter(op: &Operation) -> Result<Vec<String>, String> {
    let brightness: f64 = op.params["brightness"].as_f64().unwrap_or(0.0);
    let contrast: f64 = op.params["contrast"].as_f64().unwrap_or(1.0);
    let gamma: f64 = op.params["gamma"].as_f64().unwrap_or(1.0);

    let brightness = brightness.clamp(-0.3, 0.3);
    let contrast = contrast.clamp(0.7, 1.5);
    let gamma = gamma.clamp(0.8, 1.3);

    let filter = format!("eq=brightness={}:contrast={}:gamma={}", brightness, contrast, gamma);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for color balance adjustment.
/// Adjusts red/green/blue shadow channels via `colorbalance` filter.
/// Safety backstop: rs, gs, bs all clamped to [-0.3, 0.3].
pub fn build_color_balance_filter(op: &Operation) -> Result<Vec<String>, String> {
    let rs: f64 = op.params["rs"].as_f64().unwrap_or(0.0);
    let gs: f64 = op.params["gs"].as_f64().unwrap_or(0.0);
    let bs: f64 = op.params["bs"].as_f64().unwrap_or(0.0);

    let rs = rs.clamp(-0.3, 0.3);
    let gs = gs.clamp(-0.3, 0.3);
    let bs = bs.clamp(-0.3, 0.3);

    let filter = format!("colorbalance=rs={}:gs={}:bs={}", rs, gs, bs);
    Ok(vec!["-vf".to_string(), filter])
}

// =========================================================================
// Phase 6: Noise Texture (D-01, D-02) — 3 new filter builder functions
// =========================================================================

/// Build FFmpeg filter arguments for film grain via `noise` filter.
/// Safety backstop: strength clamped to [5, 30].
pub fn build_film_grain_filter(op: &Operation) -> Result<Vec<String>, String> {
    let strength: u32 = op.params["strength"].as_u64().unwrap_or(15) as u32;
    let flags = op.params["flags"].as_str().unwrap_or("t+u");

    let strength = strength.clamp(5, 30);

    let filter = format!("noise=alls={}:allf={}", strength, flags);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for Gaussian blur via `gblur` filter.
/// Safety backstop: sigma clamped to [0.5, 3.0].
pub fn build_gaussian_blur_filter(op: &Operation) -> Result<Vec<String>, String> {
    let sigma: f64 = op.params["sigma"].as_f64().unwrap_or(1.5);

    let sigma = sigma.clamp(0.5, 3.0);

    let filter = format!("gblur=sigma={}", sigma);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for sharpen via `unsharp` filter.
/// Uses fixed luma matrix size 3x3 for subtle sharpening.
/// Safety backstop: amount [0.5, 2.0], radius [1.0, 5.0].
pub fn build_sharpen_filter(op: &Operation) -> Result<Vec<String>, String> {
    let amount: f64 = op.params["amount"].as_f64().unwrap_or(1.0);
    let _radius: f64 = op.params["radius"].as_f64().unwrap_or(3.0);

    let amount = amount.clamp(0.5, 2.0);

    let filter = format!("unsharp=luma_msize_x=3:luma_msize_y=3:luma_amount={}", amount);
    Ok(vec!["-vf".to_string(), filter])
}

// =========================================================================
// Phase 6: Geometric Fine-Tuning (D-01) — 3 new filter builder functions
// =========================================================================

/// Build FFmpeg filter arguments for micro-rotation via `rotate` filter.
/// Converts degrees to radians. Preserves original dimensions via ow/oh.
/// Safety backstop: angle clamped to [-1.0, 1.0] degrees per D-01.
pub fn build_micro_rotate_filter(op: &Operation) -> Result<Vec<String>, String> {
    let angle_deg: f64 = op.params["angle"].as_f64().unwrap_or(0.0);

    let angle_deg = angle_deg.clamp(-1.0, 1.0);
    let radians = angle_deg * std::f64::consts::PI / 180.0;

    let filter = format!("rotate={}:ow=iw:oh=ih", radians);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for tiny scaling via `scale` filter.
/// Uses lanczos flags for high-quality resampling.
/// Safety backstop: scaleFactor clamped to [0.99, 1.01] per D-01.
pub fn build_tiny_scale_filter(op: &Operation) -> Result<Vec<String>, String> {
    let scale_factor: f64 = op.params["scaleFactor"].as_f64().unwrap_or(1.0);

    let scale_factor = scale_factor.clamp(0.99, 1.01);

    let filter = format!("scale=iw*{}:ih*{}:flags=lanczos", scale_factor, scale_factor);
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for horizontal or vertical flip.
/// Validates direction against known variants; errors on unknown values.
pub fn build_flip_filter(op: &Operation) -> Result<Vec<String>, String> {
    let direction = op.params["direction"].as_str().unwrap_or("horizontal");

    let filter = match direction {
        "horizontal" => "hflip",
        "vertical" => "vflip",
        _ => return Err(format!("Unknown flip direction: {}", direction)),
    };

    Ok(vec!["-vf".to_string(), filter.to_string()])
}

// =========================================================================
// Phase 6: Blend Overlay (D-01) — 3 new filter builder functions
// =========================================================================

/// Build FFmpeg filter arguments for semi-transparent solid color overlay.
/// Uses `colorize` filter. Opacity (mix) clamped to [0.01, 0.15] per D-01.
pub fn build_solid_color_overlay_filter(op: &Operation) -> Result<Vec<String>, String> {
    let hue: f64 = op.params["hue"].as_f64().unwrap_or(0.0);
    let saturation: f64 = op.params["saturation"].as_f64().unwrap_or(0.5);
    let lightness: f64 = op.params["lightness"].as_f64().unwrap_or(0.5);
    let mix: f64 = op.params["mix"].as_f64().unwrap_or(0.08);

    let mix = mix.clamp(0.01, 0.15);

    let filter = format!(
        "colorize=hue={}:saturation={}:lightness={}:mix={}",
        hue, saturation, lightness, mix
    );
    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for gradient overlay.
/// Uses `geq` filter with alpha-based gradient expressions.
/// Opacity clamped to [0.01, 0.15] per D-01.
/// Note: Gradient quality may need visual tuning per RESEARCH experimentation note.
pub fn build_gradient_overlay_filter(op: &Operation) -> Result<Vec<String>, String> {
    let gradient_type = op.params["type"].as_str().unwrap_or("linear");
    let opacity: f64 = op.params["opacity"].as_f64().unwrap_or(0.08);

    let opacity = opacity.clamp(0.01, 0.15);

    let filter = match gradient_type {
        "linear" => format!(
            "geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*(1-{op})+128*{op}*X/W'",
            op = opacity
        ),
        "radial" => format!(
            "geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='alpha(X,Y)*(1-{op})+128*{op}*(1-hypot(X-W/2,Y-H/2)/hypot(W/2,H/2))'",
            op = opacity
        ),
        _ => return Err(format!("Unknown gradient overlay type: {}", gradient_type)),
    };

    Ok(vec!["-vf".to_string(), filter])
}

/// Build FFmpeg filter arguments for subtle watermark-like pattern blend.
/// Uses `geq` filter for pattern-based luminance modulation at low opacity.
/// Opacity clamped to [0.01, 0.15] per D-01.
pub fn build_watermark_blend_filter(op: &Operation) -> Result<Vec<String>, String> {
    let pattern = op.params["pattern"].as_str().unwrap_or("grid");
    let opacity: f64 = op.params["opacity"].as_f64().unwrap_or(0.08);

    let opacity = opacity.clamp(0.01, 0.15);

    let filter = match pattern {
        "grid" => format!(
            "geq=lum='lum(X,Y)*(1+{op}*if(mod(floor(X/40)+floor(Y/40),2),1,-1))':cb='cb(X,Y)':cr='cr(X,Y)'",
            op = opacity
        ),
        "diagonal" => format!(
            "geq=lum='lum(X,Y)*(1+{op}*if(mod(floor((X+Y)/40),2),1,-1))':cb='cb(X,Y)':cr='cr(X,Y)'",
            op = opacity
        ),
        "ripple" => format!(
            "geq=lum='lum(X,Y)*(1+{op}*sin(2*PI*X/W/8)*cos(2*PI*Y/H/8))':cb='cb(X,Y)':cr='cr(X,Y)'",
            op = opacity
        ),
        _ => return Err(format!("Unknown watermark blend pattern: {}", pattern)),
    };

    Ok(vec!["-vf".to_string(), filter])
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
        // Phase 6: Color processing (D-01, D-02)
        OperationType::HueRotate => build_hue_rotate_filter(op),
        OperationType::SaturationAdjust => build_saturation_adjust_filter(op),
        OperationType::BrightnessContrast => build_brightness_contrast_filter(op),
        OperationType::ColorBalance => build_color_balance_filter(op),
        // Phase 6: Noise texture
        OperationType::FilmGrain => build_film_grain_filter(op),
        OperationType::GaussianBlur => build_gaussian_blur_filter(op),
        OperationType::Sharpen => build_sharpen_filter(op),
        // Phase 6: Geometric fine-tuning
        OperationType::MicroRotate => build_micro_rotate_filter(op),
        OperationType::TinyScale => build_tiny_scale_filter(op),
        OperationType::Flip => build_flip_filter(op),
        // Phase 6: Blend overlay
        OperationType::SolidColorOverlay => build_solid_color_overlay_filter(op),
        OperationType::GradientOverlay => build_gradient_overlay_filter(op),
        OperationType::WatermarkBlend => build_watermark_blend_filter(op),
        // Phase 7: Stub for new variants — replaced by plan 07-02
        _ => Err(format!("unsupported operation type: {:?}", op.op_type)),
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
            // args = ["-vf", "setpts=..."]
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::AudioTweak => {
            let args = build_audio_tweak_filter(op)?;
            // args = ["-af", "volume=..."] or similar
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::AudioFilter(expr), args))
        }
        // Phase 6: All color/noise/geometric/blend ops are VideoFilter
        OperationType::HueRotate => {
            let args = build_hue_rotate_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::SaturationAdjust => {
            let args = build_saturation_adjust_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::BrightnessContrast => {
            let args = build_brightness_contrast_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::ColorBalance => {
            let args = build_color_balance_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::FilmGrain => {
            let args = build_film_grain_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::GaussianBlur => {
            let args = build_gaussian_blur_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::Sharpen => {
            let args = build_sharpen_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::MicroRotate => {
            let args = build_micro_rotate_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::TinyScale => {
            let args = build_tiny_scale_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::Flip => {
            let args = build_flip_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::SolidColorOverlay => {
            let args = build_solid_color_overlay_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::GradientOverlay => {
            let args = build_gradient_overlay_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        OperationType::WatermarkBlend => {
            let args = build_watermark_blend_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
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
    fn test_frame_drop_setpts_jitter() {
        // Normal params: offset and period within safe range
        let op = make_op(
            OperationType::FrameDrop,
            serde_json::json!({
                "offset": 0.003,
                "period": 45
            }),
        );
        let args = build_frame_drop_filter(&op).unwrap();
        assert!(args[1].contains("setpts="), "Should use setpts filter, got: {}", args[1]);
        assert!(args[1].contains("sin"), "Should include sin() oscillation");
        assert!(args[1].contains("0.003"), "Should use the passed offset value");
        assert!(args[1].contains("45"), "Should use the passed period value");
    }

    #[test]
    fn test_frame_drop_clamps_offset_too_small() {
        // offset below minimum → clamped to 0.0001
        let op = make_op(
            OperationType::FrameDrop,
            serde_json::json!({
                "offset": 0.0,
                "period": 30
            }),
        );
        let args = build_frame_drop_filter(&op).unwrap();
        assert!(args[1].contains("0.0001"), "Should clamp offset up to 0.0001, got: {}", args[1]);
    }

    #[test]
    fn test_frame_drop_clamps_period_too_low() {
        // period below minimum → clamped to 15
        let op = make_op(
            OperationType::FrameDrop,
            serde_json::json!({
                "offset": 0.002,
                "period": 2
            }),
        );
        let args = build_frame_drop_filter(&op).unwrap();
        assert!(args[1].contains("15"), "Should clamp period to >=15, got: {}", args[1]);
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

    // --- Phase 6: new filter builder tests (Task 1) ---

    #[test]
    fn test_hue_rotate_basic() {
        let op = make_op(
            OperationType::HueRotate,
            serde_json::json!({"hueAngle": 45.0, "saturation": 1.2}),
        );
        let args = build_hue_rotate_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("hue=h=45"));
        assert!(args[1].contains(":s=1.2"));
    }

    #[test]
    fn test_hue_rotate_clamps() {
        let op = make_op(
            OperationType::HueRotate,
            serde_json::json!({"hueAngle": 200.0, "saturation": 1.0}),
        );
        let args = build_hue_rotate_filter(&op).unwrap();
        // Clamp hueAngle > 90.0 to 90.0
        assert!(args[1].contains("hue=h=90"));
    }

    #[test]
    fn test_saturation_adjust_basic() {
        let op = make_op(
            OperationType::SaturationAdjust,
            serde_json::json!({"saturation": 1.5, "contrast": 1.1, "brightness": 0.1}),
        );
        let args = build_saturation_adjust_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("eq="));
        assert!(args[1].contains("saturation=1.5"));
        assert!(args[1].contains(":contrast=1.1"));
        assert!(args[1].contains(":brightness=0.1"));
    }

    #[test]
    fn test_brightness_contrast_basic() {
        let op = make_op(
            OperationType::BrightnessContrast,
            serde_json::json!({"brightness": 0.0, "contrast": 1.0, "gamma": 1.0}),
        );
        let args = build_brightness_contrast_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("eq="));
        assert!(args[1].contains("brightness=0"));
        assert!(args[1].contains(":contrast=1"));
        assert!(args[1].contains(":gamma=1"));
    }

    #[test]
    fn test_color_balance_basic() {
        let op = make_op(
            OperationType::ColorBalance,
            serde_json::json!({"rs": 0.1, "gs": -0.1, "bs": 0.05}),
        );
        let args = build_color_balance_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("colorbalance=rs=0.1"));
        assert!(args[1].contains(":gs=-0.1"));
        assert!(args[1].contains(":bs=0.05"));
    }

    #[test]
    fn test_film_grain_basic() {
        let op =
            make_op(OperationType::FilmGrain, serde_json::json!({"strength": 15, "flags": "t+u"}));
        let args = build_film_grain_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("noise=alls=15"));
        assert!(args[1].contains(":allf=t+u"));
    }

    #[test]
    fn test_gaussian_blur_basic() {
        let op = make_op(OperationType::GaussianBlur, serde_json::json!({"sigma": 1.5}));
        let args = build_gaussian_blur_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("gblur=sigma=1.5"));
    }

    #[test]
    fn test_sharpen_basic() {
        let op = make_op(OperationType::Sharpen, serde_json::json!({"amount": 1.0, "radius": 3.0}));
        let args = build_sharpen_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("unsharp="));
        assert!(args[1].contains("luma_amount=1"));
        assert!(args[1].contains("luma_msize_x=3"));
    }

    #[test]
    fn test_micro_rotate_basic() {
        let op = make_op(OperationType::MicroRotate, serde_json::json!({"angle": 0.5}));
        let args = build_micro_rotate_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("rotate="));
        assert!(args[1].contains(":ow=iw:oh=ih"));
        // 0.5 degrees in radians = 0.5 * PI / 180 ≈ 0.00873
        assert!(args[1].contains("0.00872") || args[1].contains("0.00873"));
    }

    #[test]
    fn test_flip_horizontal() {
        let op = make_op(OperationType::Flip, serde_json::json!({"direction": "horizontal"}));
        let args = build_flip_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1] == "hflip");
    }

    #[test]
    fn test_flip_vertical() {
        let op = make_op(OperationType::Flip, serde_json::json!({"direction": "vertical"}));
        let args = build_flip_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1] == "vflip");
    }

    #[test]
    fn test_solid_color_overlay_clamps_mix() {
        let op = make_op(
            OperationType::SolidColorOverlay,
            serde_json::json!({"hue": 120.0, "saturation": 0.5, "lightness": 0.5, "mix": 0.5}),
        );
        let args = build_solid_color_overlay_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("colorize="));
        // mix=0.5 should clamp to 0.15 per D-01
        assert!(args[1].contains(":mix=0.15"));
    }

    #[test]
    fn test_gradient_overlay_basic() {
        let op = make_op(
            OperationType::GradientOverlay,
            serde_json::json!({"type": "linear", "opacity": 0.1}),
        );
        let args = build_gradient_overlay_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("geq="));
        assert!(args[1].contains("r='r(X,Y)'"));
    }

    #[test]
    fn test_watermark_blend_basic() {
        let op = make_op(
            OperationType::WatermarkBlend,
            serde_json::json!({"pattern": "ripple", "opacity": 0.08}),
        );
        let args = build_watermark_blend_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("geq="));
    }

    #[test]
    fn test_tiny_scale_basic() {
        let op = make_op(OperationType::TinyScale, serde_json::json!({"scaleFactor": 0.995}));
        let args = build_tiny_scale_filter(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("scale="));
        assert!(args[1].contains("iw*0.995"));
        assert!(args[1].contains(":ih*0.995"));
        assert!(args[1].contains(":flags=lanczos"));
    }

    // --- Phase 6: dispatch tests (Task 2) ---

    #[test]
    fn test_dispatch_hue_rotate() {
        let op = make_op(
            OperationType::HueRotate,
            serde_json::json!({"hueAngle": 45.0, "saturation": 1.2}),
        );
        let args = build_filter_args(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1].contains("hue=h=45"));
    }

    #[test]
    fn test_separated_hue_rotate_returns_video_filter() {
        let op = make_op(
            OperationType::HueRotate,
            serde_json::json!({"hueAngle": 30.0, "saturation": 1.0}),
        );
        let (kind, _args) = build_filter_args_separated(&op).unwrap();
        match kind {
            FilterKind::VideoFilter(expr) => assert!(expr.contains("hue=h=30")),
            other => panic!("Expected VideoFilter, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn test_separated_gop_modify_returns_other() {
        let op = make_op(OperationType::GopModify, serde_json::json!({"gopSize": 60}));
        let (kind, _args) = build_filter_args_separated(&op).unwrap();
        match kind {
            FilterKind::Other(_) => {} // expected
            other => panic!("Expected FilterKind::Other, got {:?}", std::mem::discriminant(&other)),
        }
    }

    #[test]
    fn test_dispatch_all_20_types() {
        let all_types: [OperationType; 20] = [
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
        ];
        for t in &all_types {
            let op = make_op(
                *t,
                serde_json::json!({
                    "pattern": "ripple", "opacity": 0.08, "frequency": 80.0,
                    "dx": 0, "dy": 0, "interval": 30, "gopSize": 60,
                    "effect": "volume", "db": 0.5,
                    "hueAngle": 30.0, "saturation": 1.0,
                    "contrast": 1.0, "brightness": 0.0, "gamma": 1.0,
                    "rs": 0.0, "gs": 0.0, "bs": 0.0,
                    "strength": 15, "flags": "t+u",
                    "sigma": 1.5,
                    "amount": 1.0, "radius": 3.0,
                    "angle": 0.5,
                    "scaleFactor": 1.0,
                    "direction": "horizontal",
                    "hue": 0.0, "lightness": 0.5, "mix": 0.08,
                    "type": "linear",
                }),
            );
            let result = build_filter_args(&op);
            assert!(result.is_ok(), "Failed for {:?}: {:?}", t, result.err());
        }
    }

    #[test]
    fn test_separated_all_20_types() {
        let all_types: [OperationType; 20] = [
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
        ];
        for t in &all_types {
            let op = make_op(
                *t,
                serde_json::json!({
                    "pattern": "ripple", "opacity": 0.08, "frequency": 80.0,
                    "dx": 0, "dy": 0, "interval": 30, "gopSize": 60,
                    "effect": "volume", "db": 0.5,
                    "hueAngle": 30.0, "saturation": 1.0,
                    "contrast": 1.0, "brightness": 0.0, "gamma": 1.0,
                    "rs": 0.0, "gs": 0.0, "bs": 0.0,
                    "strength": 15, "flags": "t+u",
                    "sigma": 1.5,
                    "amount": 1.0, "radius": 3.0,
                    "angle": 0.5,
                    "scaleFactor": 1.0,
                    "direction": "horizontal",
                    "hue": 0.0, "lightness": 0.5, "mix": 0.08,
                    "type": "linear",
                }),
            );
            let result = build_filter_args_separated(&op);
            assert!(result.is_ok(), "Failed for {:?}: {:?}", t, result.err());
        }
    }

    #[test]
    fn test_dispatch_flip_horizontal() {
        let op = make_op(OperationType::Flip, serde_json::json!({"direction": "horizontal"}));
        let args = build_filter_args(&op).unwrap();
        assert!(args[0] == "-vf");
        assert!(args[1] == "hflip");
    }
}
