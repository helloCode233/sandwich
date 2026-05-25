//! FFmpeg filter chain builders for all 20 operation types.
//!
//! Each function takes an `Operation` reference and returns `Vec<String>` of
//! FFmpeg CLI arguments. SEED-04 safety constraints are enforced via clamping.

use crate::models::seed::{Operation, OperationType};
use std::collections::HashMap;

/// Metadata context passed from executor to filter builders that need current file metadata.
/// Populated by ffprobe in the executor before building filter args for MetadataSelectiveErase.
/// All other operations receive None.
/// Defined here in filters.rs (not probe.rs) so Plan 03 compiles independently in Wave 2.
/// Plan 05 (executor + probe) references this struct from filters.rs.
pub struct MetadataContext {
    /// All global metadata tags from the input file, as key-value pairs.
    pub fields: HashMap<String, String>,
}

/// Build FFmpeg filter arguments for mathematical overlay.
/// SEED-04: opacity <= 0.15, frequency range 20-200.
pub fn build_math_overlay_filter(op: &Operation) -> Result<Vec<String>, String> {
    let pattern = op.params["pattern"].as_str().unwrap_or("ripple");
    let opacity: f64 = op.params["opacity"].as_f64().unwrap_or(0.08);
    let frequency: f64 = op.params["frequency"].as_f64().unwrap_or(80.0);

    // Clamp to safety constraints per SEED-04
    let opacity = opacity.clamp(0.01, 0.15);
    let frequency = frequency.clamp(20.0, 200.0);

    // Build geq expression based on pattern.
    // CRITICAL: FFmpeg's geq filter wraps uint8 values (mod 256) instead of
    // clamping. Any luma expression that can exceed 255 must be wrapped with
    // clip(..., 0, 255). Without clip, white pixels (Y≈235) overflow to
    // near-black — e.g. 235 * 1.095 = 257 → 257 % 256 = 1.
    let expr = match pattern {
        "ripple" => format!(
            "lum='clip(lum(X,Y)*(1+{opacity}*sin(2*PI*{freq}*X/W)*sin(2*PI*{freq}*Y/H)), 0, 255)':cb='cb(X,Y)':cr='cr(X,Y)'",
            opacity = opacity,
            freq = frequency / 100.0
        ),
        "stripes" => format!(
            "lum='clip(lum(X,Y)*(1+{opacity}*sin(2*PI*{freq}*X/W)), 0, 255)':cb='cb(X,Y)':cr='cr(X,Y)'",
            opacity = opacity,
            freq = frequency / 100.0
        ),
        "concentric" => format!(
            "lum='clip(lum(X,Y)*(1+{opacity}*sin(2*PI*{freq}*hypot(X-W/2,Y-H/2)/W)), 0, 255)':cb='cb(X,Y)':cr='cr(X,Y)'",
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

/// Build FFmpeg filter arguments for frame dropping via select filter (D-17, D-18, D-19).
/// Replaces Phase 6's setpts micro-timing jitter approach with true frame decimation.
/// select='mod(n+1,N)' keeps frames where mod(n+1,N) != 0, dropping 1 frame every N.
/// D-18: interval 30-50 (drop 1 frame every 30-50 frames).
/// D-19: tier-driven — Conservative 40-50, Standard 30-45, Aggressive 25-35.
/// Requires -vsync vfr to prevent ffmpeg from inserting duplicate frames.
/// Safety: interval clamped to [15, 100].
pub fn build_frame_drop_filter(op: &Operation) -> Result<Vec<String>, String> {
    let interval: u32 = op.params["interval"].as_u64().unwrap_or(40) as u32;
    let interval = interval.clamp(15, 100);

    // select='mod(n+1,N)': drops frame when mod(n+1, N) == 0
    // Example N=40: frame 39 -> mod(40,40)=0 -> dropped; frame 40 -> mod(41,40)=1 -> kept
    // setpts=N/FRAME_RATE/TB resets PTS to maintain smooth playback after frame removal
    let filter = format!("select='mod(n+1,{})',setpts=N/FRAME_RATE/TB", interval);
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

    // asetrate takes a plain integer — arithmetic expressions (e.g. 48000*1.0595)
    // are NOT supported and will cause FFmpeg to exit with an error.
    // Compute the target rate in Rust before formatting.
    let target_rate = (original_rate as f64 * pitch_factor).round() as u32;
    // atempo uses 1/factor to restore speed after the pitch shift
    // aresample brings it back to the original sample rate
    let filter = format!(
        "asetrate={},atempo={:.4},aresample={}",
        target_rate,
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

    let filter =
        format!("equalizer=frequency={}:width_type=h:width={}:gain={}", frequency, width, gain);
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
// Phase 7: Crop + Duration Operations (D-04~D-08, D-14~D-16) — 3 filter builders
// =========================================================================

/// Build crop+scale filter chain (D-05: asymmetric per-side, D-06: scale back, D-07: tier-driven).
/// crop=W:H:X:Y extracts a sub-rectangle, then scale=OW:OH scales back with lanczos resampling.
/// Each side percentage is independently random: leftPct, rightPct, topPct, bottomPct.
/// Safety: each percentage clamped to [0.5, 3.5]. Expressions use iw/ih for resolution independence.
pub fn build_crop_filter(op: &Operation) -> Result<Vec<String>, String> {
    let left_pct: f64 = op.params["leftPct"].as_f64().unwrap_or(1.0);
    let right_pct: f64 = op.params["rightPct"].as_f64().unwrap_or(1.0);
    let top_pct: f64 = op.params["topPct"].as_f64().unwrap_or(1.0);
    let bottom_pct: f64 = op.params["bottomPct"].as_f64().unwrap_or(1.0);

    // Clamp to safety range (0.5%-3.5% per D-07)
    let left_pct = left_pct.clamp(0.5, 3.5);
    let right_pct = right_pct.clamp(0.5, 3.5);
    let top_pct = top_pct.clamp(0.5, 3.5);
    let bottom_pct = bottom_pct.clamp(0.5, 3.5);

    // crop=out_w:out_h:x:y using iw/ih expressions
    // After crop, iw/ih refer to the cropped dimensions — scale=iw:ih would be a no-op.
    // Use inverse factor so scale restores original dimensions (D-06: scale-back).
    //
    // CRITICAL: YUV 4:2:0 chroma subsampling requires even crop offsets.
    // Odd X/Y offsets shift chroma by 1 luma-pixel relative to luma, producing
    // black-white ghosting artifacts on sharp edges (subtitles, UI elements).
    // 2*floor(iw*PCT/200) rounds the offset down to the nearest even integer.
    //
    // CRITICAL: FFmpeg truncates crop dimensions (927.12 → 926 even), then
    // scale=iw*inv:ih*inv compounds the error (926*1.035=958 instead of 960).
    // When origW/origH are provided (injected by executor), use explicit scale
    // targets to survive FFmpeg's rounding.
    let has_orig_dims = op.params["origW"].is_number() && op.params["origH"].is_number();
    let scale_expr = if has_orig_dims {
        let orig_w = op.params["origW"].as_u64().unwrap_or(544);
        let orig_h = op.params["origH"].as_u64().unwrap_or(960);
        format!("scale={}:{}:flags=lanczos", orig_w, orig_h)
    } else {
        let inv_w = 1.0 / (1.0 - left_pct / 100.0 - right_pct / 100.0);
        let inv_h = 1.0 / (1.0 - top_pct / 100.0 - bottom_pct / 100.0);
        format!("scale=iw*{:.6}:ih*{:.6}:flags=lanczos", inv_w, inv_h)
    };
    let filter = format!(
        "crop=iw*(1-{lp}/100-{rp}/100):ih*(1-{tp}/100-{bp}/100):2*floor(iw*{lp}/200):2*floor(ih*{tp}/200),{scale}",
        lp = left_pct,
        rp = right_pct,
        tp = top_pct,
        bp = bottom_pct,
        scale = scale_expr,
    );
    Ok(vec!["-vf".to_string(), filter])
}

/// Build VideoSpeed filter (D-14, D-15): synchronized setpts (video) + atempo (audio).
/// Speed factor 0.95-1.05x. Video PTS / factor, audio tempo = factor.
/// This is a multi-filter operation: returns BOTH video and audio filter expressions.
/// Safety: speedFactor clamped to [0.95, 1.05].
pub fn build_video_speed_filter(op: &Operation) -> Result<Vec<String>, String> {
    let speed_factor: f64 = op.params["speedFactor"].as_f64().unwrap_or(1.0);
    let speed_factor = speed_factor.clamp(0.95, 1.05);

    // setpts: speed up video by inverse factor
    // atempo: speed up audio to match — FFmpeg requires 0.5-2.0, our range is safe
    let vf_expr = format!("setpts={:.4}*PTS", 1.0 / speed_factor);
    let af_expr = format!("atempo={:.4}", speed_factor);

    // Return as combined filter args: -vf setpts=... -af atempo=...
    Ok(vec!["-vf".to_string(), vf_expr, "-af".to_string(), af_expr])
}

/// Build TrimEdges filter (D-16): trim head/tail frames using trim+atrim filters.
/// Randomly selects: head-only, tail-only, or both.
/// Trims 1-30 frames per edge.
/// Uses setpts=PTS-STARTPTS and asetpts=PTS-STARTPTS to reset timestamps after trim.
/// Safety: trimFrames clamped to [1, 30]. Trim mode validated against known values.
pub fn build_trim_edges_filter(op: &Operation) -> Result<Vec<String>, String> {
    let trim_frames: u32 = op.params["trimFrames"].as_u64().unwrap_or(10) as u32;
    let mode = op.params["mode"].as_str().unwrap_or("both");
    let total_frames: u32 = op.params["totalFrames"].as_u64().unwrap_or(0) as u32;

    let trim_frames = trim_frames.clamp(1, 30);

    // Approximate fps for converting frames to seconds (audio trim companion).
    // total_frames / 30.0 gives a coarse duration estimate; exact fps is not
    // available at filter-build time (only at execution via ffprobe).
    let approx_fps: f64 = 30.0;
    let trim_secs = trim_frames as f64 / approx_fps;
    let total_secs = total_frames as f64 / approx_fps;

    let (vf, af) = match mode {
        "head" => (
            format!("trim=start_frame={},setpts=PTS-STARTPTS", trim_frames),
            format!("atrim=start={},asetpts=PTS-STARTPTS", trim_secs),
        ),
        "tail" => (
            // end_frame=N: FFmpeg docs — "Number of the first frame that will be dropped."
            // end_frame = total - trim keeps frames 0..(total-trim-1), drops last trim frames.
            format!(
                "trim=start_frame=0:end_frame={},setpts=PTS-STARTPTS",
                total_frames.saturating_sub(trim_frames)
            ),
            format!("atrim=end={},asetpts=PTS-STARTPTS", (total_secs - trim_secs).max(0.0)),
        ),
        "both" => (
            format!(
                "trim=start_frame={}:end_frame={},setpts=PTS-STARTPTS",
                trim_frames,
                total_frames.saturating_sub(trim_frames)
            ),
            format!(
                "atrim=start={}:end={},asetpts=PTS-STARTPTS",
                trim_secs,
                (total_secs - trim_secs).max(0.0)
            ),
        ),
        _ => return Err(format!("Unknown trim mode: {}", mode)),
    };

    Ok(vec!["-vf".to_string(), vf, "-af".to_string(), af])
}

// =========================================================================
// Phase 7: Metadata Operations (D-09~D-13) — 2 filter builders (MetadataErase stays unchanged)
// =========================================================================

/// Build FFmpeg arguments for metadata writing (D-10, D-11).
/// Reads fake field values from the operation's params and outputs -metadata key=value pairs.
/// Fields: creation_time, title, author, comment, copyright, encoder.
/// These are CLI-level arguments, not video/audio filters — returned as Other.
pub fn build_metadata_write_filter(op: &Operation) -> Result<Vec<String>, String> {
    let fields = [
        ("creation_time", "creationTime"),
        ("title", "title"),
        ("author", "author"),
        ("comment", "comment"),
        ("copyright", "copyright"),
        ("encoder", "encoder"),
    ];
    let mut args = Vec::new();
    for (ffmpeg_key, param_key) in &fields {
        if let Some(val) = op.params.get(param_key) {
            if let Some(s) = val.as_str() {
                if !s.is_empty() {
                    args.push("-metadata".to_string());
                    args.push(format!("{}={}", ffmpeg_key, s));
                }
            }
        }
    }
    Ok(args)
}

/// Build FFmpeg arguments for selective metadata erase (D-12).
/// Requires the file's current metadata to know which fields to keep.
/// This function receives the erase categories from op.params and uses MetadataContext
/// (passed via build_filter_args_separated's optional parameter) to determine field mappings.
///
/// Strategy: -map_metadata -1 strips all metadata, then selectively write back kept fields.
/// The kept fields are those NOT in the erased categories.
/// Category field mappings (from RESEARCH.md):
///   time: creation_time, date, modify_date, timecode, year
///   device: make, model, camera, lens, com.android.*, apple.*
///   description: title, comment, author, copyright, description, album, artist, genre
///
/// When MetadataContext is None (no ffprobe data available), falls back to -map_metadata -1
/// with no writeback (effectively full erase).
pub fn build_metadata_selective_erase_filter(
    op: &Operation,
    metadata_ctx: Option<&MetadataContext>,
) -> Result<Vec<String>, String> {
    let mut args = vec!["-map_metadata".to_string(), "-1".to_string()];

    let ctx = match metadata_ctx {
        Some(c) => c,
        None => return Ok(args), // No metadata context → full erase only
    };

    // Read which categories to erase from params
    let categories: Vec<&str> = op.params["categories"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    if categories.is_empty() {
        return Ok(args); // No categories specified → full erase
    }

    // Category field mappings
    let time_fields = ["creation_time", "date", "modify_date", "timecode", "year"];
    let device_fields = ["make", "model", "camera", "lens"];
    let description_fields =
        ["title", "comment", "author", "copyright", "description", "album", "artist", "genre"];

    // Collect all fields to erase
    let mut erase_fields: Vec<&str> = Vec::new();
    for cat in &categories {
        match *cat {
            "time" => erase_fields.extend_from_slice(&time_fields),
            "device" => erase_fields.extend_from_slice(&device_fields),
            "description" => erase_fields.extend_from_slice(&description_fields),
            _ => {}
        }
    }

    // Write back fields NOT in the erase set
    for (key, value) in &ctx.fields {
        let key_lower = key.to_lowercase();
        let should_erase = erase_fields.iter().any(|ef| {
            key_lower == *ef
                || key_lower.starts_with("com.android.")
                || key_lower.starts_with("apple.")
        });
        if !should_erase && !value.is_empty() {
            args.push("-metadata".to_string());
            args.push(format!("{}={}", key, value));
        }
    }

    Ok(args)
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
            "geq=lum='clip(lum(X,Y)*(1+{op}*if(mod(floor(X/40)+floor(Y/40),2),1,-1)), 0, 255)':cb='cb(X,Y)':cr='cr(X,Y)'",
            op = opacity
        ),
        "diagonal" => format!(
            "geq=lum='clip(lum(X,Y)*(1+{op}*if(mod(floor((X+Y)/40),2),1,-1)), 0, 255)':cb='cb(X,Y)':cr='cr(X,Y)'",
            op = opacity
        ),
        "ripple" => format!(
            "geq=lum='clip(lum(X,Y)*(1+{op}*sin(2*PI*X/W/8)*cos(2*PI*Y/H/8)), 0, 255)':cb='cb(X,Y)':cr='cr(X,Y)'",
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
        // Phase 7: Audio (5)
        OperationType::AudioResample => build_audio_resample_filter(op),
        OperationType::AudioVolume => build_audio_volume_filter(op),
        OperationType::AudioPitch => build_audio_pitch_filter(op),
        OperationType::AudioEQ => build_audio_eq_filter(op),
        OperationType::AudioChannel => build_audio_channel_filter(op),
        // Phase 7: Crop (1)
        OperationType::Crop => build_crop_filter(op),
        // Phase 7: Metadata (2)
        OperationType::MetadataWrite => build_metadata_write_filter(op),
        OperationType::MetadataSelectiveErase => build_metadata_selective_erase_filter(op, None),
        // Phase 7: Duration (2)
        OperationType::VideoSpeed => build_video_speed_filter(op),
        OperationType::TrimEdges => build_trim_edges_filter(op),
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
///
/// Phase 7 change: returns `Vec<(FilterKind, Vec<String>)>` instead of a single tuple
/// to support multi-filter operations (VideoSpeed, TrimEdges) that need both -vf and -af.
/// Accepts optional MetadataContext for MetadataSelectiveErase's ffprobe data dependency.
pub fn build_filter_args_separated(
    op: &Operation,
    metadata_ctx: Option<&MetadataContext>,
) -> Result<Vec<(FilterKind, Vec<String>)>, String> {
    match op.op_type {
        OperationType::MathOverlay => {
            let args = build_math_overlay_filter(op)?;
            // args = ["-vf", "geq=..."]
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::PixelShift => {
            let args = build_pixel_shift_filter(op)?;
            // args = ["-vf", "crop=...,pad=..."]
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::FrameDrop => {
            let args = build_frame_drop_filter(op)?;
            // args = ["-vf", "select='mod(n+1,N)',setpts=..."]
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::AudioTweak => {
            let args = build_audio_tweak_filter(op)?;
            // args = ["-af", "volume=..."] or similar
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::AudioFilter(expr), args)])
        }
        // Phase 6: All color/noise/geometric/blend ops are VideoFilter
        OperationType::HueRotate => {
            let args = build_hue_rotate_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::SaturationAdjust => {
            let args = build_saturation_adjust_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::BrightnessContrast => {
            let args = build_brightness_contrast_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::ColorBalance => {
            let args = build_color_balance_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::FilmGrain => {
            let args = build_film_grain_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::GaussianBlur => {
            let args = build_gaussian_blur_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::Sharpen => {
            let args = build_sharpen_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::MicroRotate => {
            let args = build_micro_rotate_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::TinyScale => {
            let args = build_tiny_scale_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::Flip => {
            let args = build_flip_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::SolidColorOverlay => {
            let args = build_solid_color_overlay_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::GradientOverlay => {
            let args = build_gradient_overlay_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        OperationType::WatermarkBlend => {
            let args = build_watermark_blend_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        // Phase 7: Audio (5) — all return AudioFilter
        OperationType::AudioResample => {
            let args = build_audio_resample_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::AudioFilter(expr), args)])
        }
        OperationType::AudioVolume => {
            let args = build_audio_volume_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::AudioFilter(expr), args)])
        }
        OperationType::AudioPitch => {
            let args = build_audio_pitch_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::AudioFilter(expr), args)])
        }
        OperationType::AudioEQ => {
            let args = build_audio_eq_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::AudioFilter(expr), args)])
        }
        OperationType::AudioChannel => {
            let args = build_audio_channel_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::AudioFilter(expr), args)])
        }
        // Phase 7: Crop (1) — VideoFilter
        OperationType::Crop => {
            let args = build_crop_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok(vec![(FilterKind::VideoFilter(expr), args)])
        }
        // Phase 7: Metadata (2) — Other
        OperationType::MetadataWrite => {
            let args = build_metadata_write_filter(op)?;
            Ok(vec![(FilterKind::Other(args.clone()), args)])
        }
        OperationType::MetadataSelectiveErase => {
            let args = build_metadata_selective_erase_filter(op, metadata_ctx)?;
            Ok(vec![(FilterKind::Other(args.clone()), args)])
        }
        // Phase 7: Duration (2) — VideoSpeed returns BOTH VideoFilter and AudioFilter
        OperationType::VideoSpeed => {
            let args = build_video_speed_filter(op)?;
            // args = ["-vf", "setpts=N*PTS", "-af", "atempo=N"]
            let vf_expr = args.get(1).cloned().unwrap_or_default();
            let af_expr = args.get(3).cloned().unwrap_or_default();
            Ok(vec![
                (FilterKind::VideoFilter(vf_expr.clone()), vec!["-vf".to_string(), vf_expr]),
                (FilterKind::AudioFilter(af_expr.clone()), vec!["-af".to_string(), af_expr]),
            ])
        }
        OperationType::TrimEdges => {
            let args = build_trim_edges_filter(op)?;
            let vf_expr = args.get(1).cloned().unwrap_or_default();
            let af_expr = args.get(3).cloned().unwrap_or_default();
            Ok(vec![
                (FilterKind::VideoFilter(vf_expr.clone()), vec!["-vf".to_string(), vf_expr]),
                (FilterKind::AudioFilter(af_expr.clone()), vec!["-af".to_string(), af_expr]),
            ])
        }
        _ => {
            // GopModify, MetadataErase, Remux — pass through as Other
            let args = build_filter_args(op)?;
            Ok(vec![(FilterKind::Other(args.clone()), args)])
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
    fn test_frame_drop_select_based() {
        // Normal params: interval within safe range, verify select filter
        let op = make_op(
            OperationType::FrameDrop,
            serde_json::json!({
                "interval": 45
            }),
        );
        let args = build_frame_drop_filter(&op).unwrap();
        assert!(args[1].contains("select='mod(n+1"), "Should use select filter, got: {}", args[1]);
        assert!(
            args[1].contains("setpts=N/FRAME_RATE/TB"),
            "Should include setpts for PTS reset, got: {}",
            args[1]
        );
        assert!(args[1].contains("45"), "Should use the passed interval value");
    }

    #[test]
    fn test_frame_drop_clamps_interval() {
        // interval below minimum → clamped to 15
        let op = make_op(
            OperationType::FrameDrop,
            serde_json::json!({
                "interval": 2
            }),
        );
        let args = build_frame_drop_filter(&op).unwrap();
        assert!(args[1].contains("15"), "Should clamp interval to >=15, got: {}", args[1]);
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
        let result = build_filter_args_separated(&op, None).unwrap();
        let (kind, _args) = &result[0];
        match kind {
            FilterKind::VideoFilter(expr) => assert!(expr.contains("hue=h=30")),
            other => panic!("Expected VideoFilter, got {:?}", std::mem::discriminant(other)),
        }
    }

    #[test]
    fn test_separated_gop_modify_returns_other() {
        let op = make_op(OperationType::GopModify, serde_json::json!({"gopSize": 60}));
        let result = build_filter_args_separated(&op, None).unwrap();
        let (kind, _args) = &result[0];
        match kind {
            FilterKind::Other(_) => {} // expected
            other => panic!("Expected FilterKind::Other, got {:?}", std::mem::discriminant(other)),
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
            let result = build_filter_args_separated(&op, None);
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
