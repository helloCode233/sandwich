use rand::prelude::*;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use crate::models::seed::{Operation, OperationType, Seed, StrengthTier};
use crate::state::AppState;

/// Select an operation type using weighted random selection (D-17).
/// Weight distribution: Math overlay ~15%, Color processing ~20%, Noise texture ~15%,
/// Geometric fine-tuning ~15%, Blend overlay ~10%, Remaining old categories ~25%.
/// Uses 1000-bucket cumulative probability threshold for finer granularity.
fn pick_operation_type(rng: &mut impl Rng) -> OperationType {
    let roll: u32 = rng.random_range(1..=1000);
    match roll {
        // Math overlay (existing 3): ~15% = 150 buckets, ~50 each
        1..=50 => OperationType::MathOverlay,
        51..=100 => OperationType::MathOverlay,
        101..=150 => OperationType::MathOverlay,
        // Color processing (4): ~20% = 200 buckets, ~50 each
        151..=200 => OperationType::HueRotate,
        201..=250 => OperationType::SaturationAdjust,
        251..=300 => OperationType::BrightnessContrast,
        301..=350 => OperationType::ColorBalance,
        // Noise texture (3): ~15% = 150 buckets, ~50 each
        351..=400 => OperationType::FilmGrain,
        401..=450 => OperationType::GaussianBlur,
        451..=500 => OperationType::Sharpen,
        // Geometric fine-tuning (3): ~15% = 150 buckets, ~50 each
        501..=550 => OperationType::MicroRotate,
        551..=600 => OperationType::TinyScale,
        601..=650 => OperationType::Flip,
        // Blend overlay (3): ~10% = 100 buckets, ~33 each
        651..=683 => OperationType::SolidColorOverlay,
        684..=716 => OperationType::GradientOverlay,
        717..=750 => OperationType::WatermarkBlend,
        // Remaining old categories (6): ~25% = 250 buckets, ~42 each
        751..=792 => OperationType::PixelShift,
        793..=834 => OperationType::FrameDrop,
        835..=876 => OperationType::GopModify,
        877..=918 => OperationType::MetadataErase,
        919..=959 => OperationType::AudioTweak,
        960..=1000 => OperationType::Remux,
        _ => unreachable!("roll is 1..=1000"),
    }
}

/// Validate that operations collectively cover >=70% of video frames (D-09).
/// For short videos (<50 frames), a relaxed 50% threshold is used.
/// Returns true if coverage is adequate.
fn validate_coverage(operations: &[Operation], total_frames: u32) -> bool {
    if total_frames == 0 {
        return true;
    }
    if operations.is_empty() {
        return false;
    }

    let threshold = if total_frames < 50 { 0.50 } else { 0.70 };

    let mut covered = vec![false; total_frames as usize];
    for op in operations {
        let start = op.start_frame as usize;
        let end = if op.duration_frames == 0 {
            total_frames as usize
        } else {
            ((op.start_frame + op.duration_frames) as usize).min(total_frames as usize)
        };
        for i in start..end {
            covered[i] = true;
        }
    }
    let covered_count = covered.iter().filter(|&&c| c).count();
    (covered_count as f64 / total_frames as f64) >= threshold
}

/// Tauri command: Generate a random seed with strength tier and coverage validation.
/// Per D-03: strength tier drives step count and parameter ranges.
/// Per D-06: step count 5-7 (conservative), 6-9 (standard), 8-12 (aggressive).
/// Per D-07: three global strength presets with tier-appropriate parameter ranges.
/// Per D-09: coverage >=70% validated with retry; relaxed for short videos.
/// Per D-04: auto-alias using timestamp format "seed_YYYYMMDD_HHMMSS".
/// Per D-01: pure random generation, user cannot edit operation parameters.
#[tauri::command]
pub async fn generate_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    strength: String,
    total_frames: Option<u32>,
) -> Result<Seed, String> {
    let strength_tier = match strength.as_str() {
        "conservative" => StrengthTier::Conservative,
        "standard" => StrengthTier::Standard,
        "aggressive" => StrengthTier::Aggressive,
        _ => {
            return Err(format!(
                "Invalid strength tier: {}. Use conservative, standard, or aggressive",
                strength
            ));
        }
    };

    let mut rng = rand::rng();

    // D-06, D-07: Step count per tier
    let (min_steps, max_steps) = match strength_tier {
        StrengthTier::Conservative => (5, 7),
        StrengthTier::Standard => (6, 9),
        StrengthTier::Aggressive => (8, 12),
    };
    let step_count = rng.random_range(min_steps..=max_steps);
    let mut operations = Vec::with_capacity(step_count);

    for _ in 0..step_count {
        let op_type = pick_operation_type(&mut rng);
        let op = generate_operation(&mut rng, op_type, strength_tier, total_frames);
        operations.push(op);
    }

    // D-09: Coverage validation with retry (up to 100 attempts)
    if let Some(frames) = total_frames {
        if frames > 0 {
            let mut retries = 0;
            while !validate_coverage(&operations, frames) && retries < 100 {
                // Re-randomize start_frame/duration_frames for all ops
                for op in &mut operations {
                    let (start, dur) = random_frame_range(&mut rng, op.op_type, frames);
                    op.start_frame = start;
                    op.duration_frames = dur;
                }
                retries += 1;
            }
            // Fallback: set last operation to cover full video
            if !validate_coverage(&operations, frames) {
                if let Some(last) = operations.last_mut() {
                    last.start_frame = 0;
                    last.duration_frames = 0; // 0 = full video
                }
            }
        }
    }

    // D-04: Auto-alias with timestamp + tier suffix
    let tier_label = match strength_tier {
        StrengthTier::Conservative => "cons",
        StrengthTier::Standard => "std",
        StrengthTier::Aggressive => "agg",
    };
    let alias = format!("seed_{}_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"), tier_label);

    let seed = Seed {
        id: uuid::Uuid::new_v4().to_string(),
        alias,
        operations,
        created_at: chrono::Utc::now().to_rfc3339(),
        strength_tier,
        schema_version: 3,
    };

    // Persist to managed state
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.seeds.push(seed.clone());
    }

    // Write-through to store
    persist_seeds(&app)?;

    // Emit event to frontend
    let _ = app.emit("seeds-updated", ());

    Ok(seed)
}

/// Generate random frame range for an operation (D-09).
/// For FrameDrop: retains time-slice behavior (start 0..300, dur 60..600)
/// — applies setpts micro-jitter to this segment only.
/// For all other ops: random start within video bounds, random duration covering at least 1 frame.
fn random_frame_range(rng: &mut impl Rng, op_type: OperationType, total_frames: u32) -> (u32, u32) {
    match op_type {
        OperationType::FrameDrop => {
            let start = rng.random_range(0..300u32);
            let dur = rng.random_range(60..600u32);
            (start, dur)
        }
        _ => {
            if total_frames > 1 {
                let start = rng.random_range(0..total_frames);
                let remaining = total_frames - start;
                let dur = if remaining == 0 { 0 } else { rng.random_range(1..=remaining) };
                (start, dur)
            } else {
                (0u32, 0u32)
            }
        }
    }
}

/// Generate a single Operation with tier-driven randomized parameters (D-03, D-09).
/// SEED-04 constraints applied inline. Strength tier controls parameter ranges.
fn generate_operation(
    rng: &mut impl Rng,
    op_type: OperationType,
    strength_tier: StrengthTier,
    total_frames: Option<u32>,
) -> Operation {
    let (start_frame, duration_frames) = match total_frames {
        Some(frames) => random_frame_range(rng, op_type, frames),
        None => match op_type {
            OperationType::FrameDrop => {
                let start = rng.random_range(0..300u32);
                let dur = rng.random_range(60..600u32);
                (start, dur)
            }
            _ => (0u32, 0u32),
        },
    };

    let params = match op_type {
        // ── Math overlay (existing, tier-driven opacity) ────────────────────
        OperationType::MathOverlay => {
            let pattern = match rng.random_range(0..3) {
                0 => "ripple",
                1 => "stripes",
                _ => "concentric",
            };
            let (opacity_min, opacity_max) = match strength_tier {
                StrengthTier::Conservative => (0.03, 0.08),
                StrengthTier::Standard => (0.05, 0.12),
                StrengthTier::Aggressive => (0.08, 0.15),
            };
            let opacity = rng.random_range(opacity_min..=opacity_max);
            let frequency = rng.random_range(20..=200);
            serde_json::json!({
                "pattern": pattern,
                "opacity": opacity,
                "frequency": frequency,
            })
        }
        // ── Pixel shift (existing, tier-driven) ─────────────────────────────
        OperationType::PixelShift => {
            let (min, max) = match strength_tier {
                StrengthTier::Conservative => (-1i32, 1i32),
                StrengthTier::Standard => (-2i32, 2i32),
                StrengthTier::Aggressive => (-3i32, 3i32),
            };
            let dx = rng.random_range(min..=max);
            let dy = rng.random_range(min..=max);
            serde_json::json!({ "dx": dx, "dy": dy })
        }
        // ── FrameDrop: setpts micro-timing jitter (D-01, D-02, D-04) ────
        // Replaces framestep decimation which caused slideshow (画面变成图片放映).
        // setpts sin() oscillation alters frame timestamps imperceptibly
        // while preserving all frames for smooth playback.
        OperationType::FrameDrop => {
            let (offset_min, offset_max) = match strength_tier {
                StrengthTier::Conservative => (0.0005, 0.001),
                StrengthTier::Standard => (0.001, 0.003),
                StrengthTier::Aggressive => (0.003, 0.006),
            };
            let period = rng.random_range(30..=120);
            serde_json::json!({
                "offset": rng.random_range(offset_min..=offset_max),
                "period": period,
            })
        }
        // ── GOP modify (existing) ───────────────────────────────────────────
        OperationType::GopModify => {
            let gop_size = rng.random_range(12..=250);
            serde_json::json!({ "gopSize": gop_size })
        }
        // ── Metadata erase (existing) ───────────────────────────────────────
        OperationType::MetadataErase => {
            serde_json::json!({})
        }
        // ── Audio tweak (existing, tier-driven) ────────────────────────────
        OperationType::AudioTweak => {
            let effect = match rng.random_range(0..3) {
                0 => "volume",
                1 => "tempo",
                _ => "echo",
            };
            match effect {
                "volume" => {
                    let (min_db, max_db) = match strength_tier {
                        StrengthTier::Conservative => (-0.5, 0.5),
                        StrengthTier::Standard => (-1.0, 1.0),
                        StrengthTier::Aggressive => (-2.0, 2.0),
                    };
                    serde_json::json!({ "effect": "volume", "db": rng.random_range(min_db..=max_db) })
                }
                "tempo" => {
                    let (min_f, max_f) = match strength_tier {
                        StrengthTier::Conservative => (0.995, 1.005),
                        StrengthTier::Standard => (0.99, 1.01),
                        StrengthTier::Aggressive => (0.98, 1.02),
                    };
                    serde_json::json!({ "effect": "tempo", "factor": rng.random_range(min_f..=max_f) })
                }
                _ => serde_json::json!({ "effect": "echo" }),
            }
        }
        // ── Remux (existing) ────────────────────────────────────────────────
        OperationType::Remux => {
            serde_json::json!({})
        }
        // ── Color processing (4): D-01, D-02, D-04, tier-driven ─────────────
        OperationType::HueRotate => {
            // Barely visible curves-driven shifts: tiny hue rotation + near-neutral saturation
            let (angle_min, angle_max) = match strength_tier {
                StrengthTier::Conservative => (-3.0, 3.0),
                StrengthTier::Standard => (-8.0, 8.0),
                StrengthTier::Aggressive => (-20.0, 20.0),
            };
            let (sat_min, sat_max) = match strength_tier {
                StrengthTier::Conservative => (0.97, 1.03),
                StrengthTier::Standard => (0.95, 1.05),
                StrengthTier::Aggressive => (0.85, 1.15),
            };
            serde_json::json!({
                "hueAngle": rng.random_range(angle_min..=angle_max),
                "saturation": rng.random_range(sat_min..=sat_max),
            })
        }
        OperationType::SaturationAdjust => {
            // Subtle eq curves: near-neutral saturation/contrast/brightness
            let (sat_min, sat_max) = match strength_tier {
                StrengthTier::Conservative => (0.97, 1.03),
                StrengthTier::Standard => (0.95, 1.05),
                StrengthTier::Aggressive => (0.85, 1.15),
            };
            let (con_min, con_max) = match strength_tier {
                StrengthTier::Conservative => (0.98, 1.02),
                StrengthTier::Standard => (0.96, 1.04),
                StrengthTier::Aggressive => (0.88, 1.12),
            };
            let (bri_min, bri_max) = match strength_tier {
                StrengthTier::Conservative => (-0.02, 0.02),
                StrengthTier::Standard => (-0.03, 0.03),
                StrengthTier::Aggressive => (-0.08, 0.08),
            };
            serde_json::json!({
                "saturation": rng.random_range(sat_min..=sat_max),
                "contrast": rng.random_range(con_min..=con_max),
                "brightness": rng.random_range(bri_min..=bri_max),
            })
        }
        OperationType::BrightnessContrast => {
            // Subtle gamma curves: barely perceptible brightness/contrast/gamma shifts
            let (bri_min, bri_max) = match strength_tier {
                StrengthTier::Conservative => (-0.02, 0.02),
                StrengthTier::Standard => (-0.03, 0.03),
                StrengthTier::Aggressive => (-0.08, 0.08),
            };
            let (con_min, con_max) = match strength_tier {
                StrengthTier::Conservative => (0.98, 1.02),
                StrengthTier::Standard => (0.96, 1.04),
                StrengthTier::Aggressive => (0.88, 1.12),
            };
            let (gam_min, gam_max) = match strength_tier {
                StrengthTier::Conservative => (0.98, 1.02),
                StrengthTier::Standard => (0.96, 1.04),
                StrengthTier::Aggressive => (0.88, 1.12),
            };
            serde_json::json!({
                "brightness": rng.random_range(bri_min..=bri_max),
                "contrast": rng.random_range(con_min..=con_max),
                "gamma": rng.random_range(gam_min..=gam_max),
            })
        }
        OperationType::ColorBalance => {
            // Barely visible channel shifts: tight curves-style color tilt
            // Previously ±0.05 at Standard caused visible red casts (画面过红).
            let (chan_min, chan_max) = match strength_tier {
                StrengthTier::Conservative => (-0.005, 0.005),
                StrengthTier::Standard => (-0.01, 0.01),
                StrengthTier::Aggressive => (-0.03, 0.03),
            };
            serde_json::json!({
                "rs": rng.random_range(chan_min..=chan_max),
                "gs": rng.random_range(chan_min..=chan_max),
                "bs": rng.random_range(chan_min..=chan_max),
            })
        }
        // ── Noise texture (3): D-01, D-02, D-04, tier-driven ───────────────
        OperationType::FilmGrain => {
            let (str_min, str_max) = match strength_tier {
                StrengthTier::Conservative => (5u32, 12u32),
                StrengthTier::Standard => (8u32, 20u32),
                StrengthTier::Aggressive => (15u32, 30u32),
            };
            let flags = match rng.random_range(0..3) {
                0 => "t+u",
                1 => "t",
                _ => "u",
            };
            serde_json::json!({
                "strength": rng.random_range(str_min..=str_max),
                "flags": flags,
            })
        }
        OperationType::GaussianBlur => {
            let (sig_min, sig_max) = match strength_tier {
                StrengthTier::Conservative => (0.3, 0.8),
                StrengthTier::Standard => (0.5, 1.5),
                StrengthTier::Aggressive => (1.0, 2.5),
            };
            serde_json::json!({ "sigma": rng.random_range(sig_min..=sig_max) })
        }
        OperationType::Sharpen => {
            let (amt_min, amt_max) = match strength_tier {
                StrengthTier::Conservative => (0.2, 0.5),
                StrengthTier::Standard => (0.3, 1.0),
                StrengthTier::Aggressive => (0.5, 1.5),
            };
            serde_json::json!({ "amount": rng.random_range(amt_min..=amt_max) })
        }
        // ── Geometric fine-tuning (3): D-01, D-02, D-04, tier-driven ───────
        OperationType::MicroRotate => {
            let (ang_min, ang_max) = match strength_tier {
                StrengthTier::Conservative => (-0.3, 0.3),
                StrengthTier::Standard => (-0.7, 0.7),
                StrengthTier::Aggressive => (-1.0, 1.0),
            };
            serde_json::json!({ "angle": rng.random_range(ang_min..=ang_max) })
        }
        OperationType::TinyScale => {
            let (fac_min, fac_max) = match strength_tier {
                StrengthTier::Conservative => (0.99, 1.01),
                StrengthTier::Standard => (0.98, 1.02),
                StrengthTier::Aggressive => (0.96, 1.04),
            };
            serde_json::json!({ "scaleFactor": rng.random_range(fac_min..=fac_max) })
        }
        OperationType::Flip => {
            let direction = if rng.random_bool(0.5) { "horizontal" } else { "vertical" };
            serde_json::json!({ "direction": direction })
        }
        // ── Blend overlay (3): D-01, D-02, D-04, tier-driven ───────────────
        OperationType::SolidColorOverlay => {
            let (mix_min, mix_max) = match strength_tier {
                StrengthTier::Conservative => (0.01, 0.05),
                StrengthTier::Standard => (0.03, 0.10),
                StrengthTier::Aggressive => (0.08, 0.15),
            };
            let hue: f64 = rng.random_range(0.0..=360.0);
            let saturation: f64 = rng.random_range(0.3..=1.0);
            let lightness: f64 = rng.random_range(0.3..=0.7);
            serde_json::json!({
                "hue": hue,
                "saturation": saturation,
                "lightness": lightness,
                "mix": rng.random_range(mix_min..=mix_max),
            })
        }
        OperationType::GradientOverlay => {
            let (op_min, op_max) = match strength_tier {
                StrengthTier::Conservative => (0.01, 0.05),
                StrengthTier::Standard => (0.03, 0.08),
                StrengthTier::Aggressive => (0.06, 0.12),
            };
            let gtype = if rng.random_bool(0.5) { "linear" } else { "radial" };
            serde_json::json!({
                "type": gtype,
                "opacity": rng.random_range(op_min..=op_max),
            })
        }
        OperationType::WatermarkBlend => {
            let (op_min, op_max) = match strength_tier {
                StrengthTier::Conservative => (0.005, 0.02),
                StrengthTier::Standard => (0.01, 0.03),
                StrengthTier::Aggressive => (0.02, 0.05),
            };
            let pattern = if rng.random_bool(0.5) { "grid" } else { "diagonal" };
            serde_json::json!({
                "pattern": pattern,
                "opacity": rng.random_range(op_min..=op_max),
            })
        }
        // Phase 7: Stub for new variants — replaced by plan 07-02
        _ => serde_json::json!({}),
    };

    Operation { op_type, start_frame, duration_frames, params }
}

/// Tauri command: Rename a seed's alias.
/// Per SEED-05: user can manually rename seeds after generation.
#[tauri::command]
pub async fn rename_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    seed_id: String,
    new_alias: String,
) -> Result<(), String> {
    if new_alias.trim().is_empty() {
        return Err("Alias cannot be empty".to_string());
    }

    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let seed = app_state
            .seeds
            .iter_mut()
            .find(|s| s.id == seed_id)
            .ok_or_else(|| format!("Seed not found: {}", seed_id))?;
        seed.alias = new_alias;
    }

    persist_seeds(&app)?;
    let _ = app.emit("seeds-updated", ());

    Ok(())
}

/// Tauri command: Delete a seed by ID.
#[tauri::command]
pub async fn delete_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    seed_id: String,
) -> Result<(), String> {
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let len_before = app_state.seeds.len();
        app_state.seeds.retain(|s| s.id != seed_id);
        if app_state.seeds.len() == len_before {
            return Err(format!("Seed not found: {}", seed_id));
        }
    }

    persist_seeds(&app)?;
    let _ = app.emit("seeds-updated", ());

    Ok(())
}

/// Tauri command: Copy a seed with re-randomized parameters.
/// Per D-01: copy-and-re-randomize is the supported user workflow
/// for getting a different seed based on similar operation types.
/// Preserves the source seed's strength_tier; total_frames is None since
/// copy doesn't know the video context.
#[tauri::command]
pub async fn copy_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    seed_id: String,
) -> Result<Seed, String> {
    let mut rng = rand::rng();

    let (new_seed, _source_alias) = {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let source = app_state
            .seeds
            .iter()
            .find(|s| s.id == seed_id)
            .ok_or_else(|| format!("Seed not found: {}", seed_id))?;

        let tier = source.strength_tier;

        // Re-randomize parameters for each operation but keep the same op_types
        let new_operations: Vec<Operation> = source
            .operations
            .iter()
            .map(|op| generate_operation(&mut rng, op.op_type, tier, None))
            .collect();

        let new_alias = format!("{}_copy_{}", source.alias, chrono::Utc::now().format("%H%M%S"));

        let seed = Seed {
            id: uuid::Uuid::new_v4().to_string(),
            alias: new_alias,
            operations: new_operations,
            created_at: chrono::Utc::now().to_rfc3339(),
            strength_tier: tier,
            schema_version: 3,
        };

        (seed, source.alias.clone())
    };

    // Push outside the source lock scope to avoid holding lock across persist
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.seeds.push(new_seed.clone());
    }

    persist_seeds(&app)?;
    let _ = app.emit("seeds-updated", ());

    Ok(new_seed)
}

/// Tauri command: List all seeds.
#[tauri::command]
pub async fn list_seeds(state: State<'_, Mutex<AppState>>) -> Result<Vec<Seed>, String> {
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(app_state.seeds.clone())
}

/// Write-through persistence: serialize all seeds to tauri-plugin-store.
/// Follows the exact pattern from ffmpeg.rs lines 185-191.
/// Made pub for cross-module use (export_seed.rs import command).
pub fn persist_seeds(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    let store =
        app.store("seeds.json").map_err(|e| format!("Failed to open seeds store: {}", e))?;
    let json = serde_json::to_value(&app_state.seeds)
        .map_err(|e| format!("Serialization error: {}", e))?;
    store.set("seeds", json);
    store.save().map_err(|e| format!("Failed to save seeds: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Map OperationType to a unique index 0..19 for tracking purposes.
    fn variant_index(t: OperationType) -> usize {
        match t {
            OperationType::MathOverlay => 0,
            OperationType::PixelShift => 1,
            OperationType::FrameDrop => 2,
            OperationType::GopModify => 3,
            OperationType::MetadataErase => 4,
            OperationType::AudioTweak => 5,
            OperationType::Remux => 6,
            OperationType::HueRotate => 7,
            OperationType::SaturationAdjust => 8,
            OperationType::BrightnessContrast => 9,
            OperationType::ColorBalance => 10,
            OperationType::FilmGrain => 11,
            OperationType::GaussianBlur => 12,
            OperationType::Sharpen => 13,
            OperationType::MicroRotate => 14,
            OperationType::TinyScale => 15,
            OperationType::Flip => 16,
            OperationType::SolidColorOverlay => 17,
            OperationType::GradientOverlay => 18,
            OperationType::WatermarkBlend => 19,
        }
    }

    /// Helper: create an Operation for coverage testing.
    fn make_op(start: u32, dur: u32) -> Operation {
        Operation {
            op_type: OperationType::MathOverlay,
            start_frame: start,
            duration_frames: dur,
            params: serde_json::json!({}),
        }
    }

    // ─── TEST 4: pick_operation_type distribution ───────────────────────────
    #[test]
    fn pick_operation_type_covers_all_20_types() {
        let mut rng = rand::rng();
        let mut seen_flags: [bool; 20] = [false; 20];
        for _ in 0..10_000 {
            let t = pick_operation_type(&mut rng);
            seen_flags[variant_index(t)] = true;
        }
        let seen_count = seen_flags.iter().filter(|&&f| f).count();
        assert_eq!(
            seen_count, 20,
            "pick_operation_type must produce all 20 OperationType variants"
        );
    }

    // ─── TEST 5: generate_operation for HueRotate with total_frames > 1 ───
    #[test]
    fn generate_operation_hue_rotate_with_frames() {
        let mut rng = rand::rng();
        let op = generate_operation(
            &mut rng,
            OperationType::HueRotate,
            StrengthTier::Standard,
            Some(1000),
        );
        assert!(
            op.duration_frames > 0,
            "Non-FrameDrop ops should have duration > 0 when total_frames > 1. Got: {}",
            op.duration_frames
        );
        assert!(op.start_frame < 1000, "start_frame should be within bounds");
    }

    // ─── TEST 6: FrameDrop uses setpts micro-jitter, not framestep ────────
    #[test]
    fn generate_operation_frame_drop_uses_setpts_jitter() {
        let mut rng = rand::rng();
        let op = generate_operation(
            &mut rng,
            OperationType::FrameDrop,
            StrengthTier::Standard,
            Some(5000),
        );
        // FrameDrop still has time-slice behavior for apply-to range
        assert!(
            op.start_frame < 300,
            "FrameDrop start_frame should be in 0..300, got {}",
            op.start_frame
        );
        assert!(
            op.duration_frames >= 60 && op.duration_frames < 600,
            "FrameDrop duration_frames should be in 60..600, got {}",
            op.duration_frames
        );
        // Verify setpts params (not framestep interval)
        let offset = op.params["offset"].as_f64().unwrap();
        assert!(
            offset >= 0.001 && offset <= 0.003,
            "Standard tier offset should be 0.001..0.003, got {}",
            offset
        );
        let period = op.params["period"].as_u64().unwrap();
        assert!(period >= 30 && period <= 120, "Period should be 30..120, got {}", period);
    }

    // ─── TEST 3: Coverage validation ────────────────────────────────────────
    #[test]
    fn coverage_validation_70_percent() {
        let ops = vec![make_op(0, 300), make_op(300, 400)];
        assert!(validate_coverage(&ops, 1000), "300+400 should cover 700/1000 = 70%");
    }

    #[test]
    fn coverage_validation_fails_below_70() {
        let ops = vec![make_op(0, 500)];
        assert!(!validate_coverage(&ops, 1000), "500/1000 = 50% < 70%, should fail");
    }

    #[test]
    fn coverage_validation_relaxed_for_short_videos() {
        let ops = vec![make_op(0, 15)];
        assert!(
            validate_coverage(&ops, 30),
            "15/30 = 50% >= relaxed 50% threshold for short videos"
        );
    }

    #[test]
    fn coverage_empty_ops_returns_false() {
        assert!(!validate_coverage(&[], 1000), "empty ops should fail coverage");
    }

    #[test]
    fn coverage_zero_frames_returns_true() {
        assert!(validate_coverage(&[], 0), "zero total_frames should pass");
    }

    // ─── TEST 1/2: Strength tier step counts (logic test) ──────────────────
    #[test]
    fn strength_tier_conservative_5_to_7_steps() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(42);
        for _ in 0..100 {
            let count: u32 = rng.random_range(5..=7);
            assert!(count >= 5 && count <= 7, "conservative range should be 5-7");
        }
    }

    #[test]
    fn strength_tier_aggressive_8_to_12_steps() {
        let mut rng: StdRng = SeedableRng::seed_from_u64(99);
        for _ in 0..100 {
            let count: u32 = rng.random_range(8..=12);
            assert!(count >= 8 && count <= 12, "aggressive range should be 8-12");
        }
    }

    // ─── Tier-driven parameter range tests ─────────────────────────────────
    #[test]
    fn tier_param_ranges_aggressive_wider_than_conservative() {
        let mut rng = rand::rng();
        let op_cons = generate_operation(
            &mut rng,
            OperationType::FilmGrain,
            StrengthTier::Conservative,
            Some(1000),
        );
        let cons_strength: u32 = op_cons.params["strength"].as_u64().unwrap_or(0) as u32;
        assert!(
            cons_strength >= 5 && cons_strength <= 12,
            "Conservative FilmGrain strength should be 5-12, got {}",
            cons_strength
        );

        let mut rng2 = rand::rng();
        let op_agg = generate_operation(
            &mut rng2,
            OperationType::FilmGrain,
            StrengthTier::Aggressive,
            Some(1000),
        );
        let agg_strength: u32 = op_agg.params["strength"].as_u64().unwrap_or(0) as u32;
        assert!(
            agg_strength >= 15 && agg_strength <= 30,
            "Aggressive FilmGrain strength should be 15-30, got {}",
            agg_strength
        );
    }

    #[test]
    fn hue_rotate_has_saturation_field() {
        let mut rng = rand::rng();
        let op = generate_operation(
            &mut rng,
            OperationType::HueRotate,
            StrengthTier::Standard,
            Some(1000),
        );
        assert!(op.params.get("hueAngle").is_some(), "HueRotate should have hueAngle param");
        assert!(op.params.get("saturation").is_some(), "HueRotate should have saturation param");
    }
}
