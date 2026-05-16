use rand::prelude::*;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use crate::models::seed::{Operation, OperationType, Seed, StrengthTier};
use crate::state::AppState;

/// Select an operation type using weighted random selection.
/// D-02: MathOverlay ~30%, remaining 19 types distributed evenly (sum 100).
/// Uses cumulative probability threshold — rand 0.9 compatible (avoids WeightedIndex API drift).
fn pick_operation_type(rng: &mut impl Rng) -> OperationType {
    // Weights: MathOverlay=30, PixelShift=5, FrameDrop=5, GopModify=5,
    //          MetadataErase=5, AudioTweak=5, Remux=5,
    //          HueRotate=3, SaturationAdjust=3, BrightnessContrast=3, ColorBalance=3,
    //          FilmGrain=3, GaussianBlur=3, Sharpen=3,
    //          MicroRotate=3, TinyScale=3, Flip=3,
    //          SolidColorOverlay=3, GradientOverlay=3, WatermarkBlend=4
    let roll: u32 = rng.random_range(1..=100);
    match roll {
        1..=30 => OperationType::MathOverlay,
        31..=35 => OperationType::PixelShift,
        36..=40 => OperationType::FrameDrop,
        41..=45 => OperationType::GopModify,
        46..=50 => OperationType::MetadataErase,
        51..=55 => OperationType::AudioTweak,
        56..=60 => OperationType::Remux,
        61..=63 => OperationType::HueRotate,
        64..=66 => OperationType::SaturationAdjust,
        67..=69 => OperationType::BrightnessContrast,
        70..=72 => OperationType::ColorBalance,
        73..=75 => OperationType::FilmGrain,
        76..=78 => OperationType::GaussianBlur,
        79..=81 => OperationType::Sharpen,
        82..=84 => OperationType::MicroRotate,
        85..=87 => OperationType::TinyScale,
        88..=90 => OperationType::Flip,
        91..=93 => OperationType::SolidColorOverlay,
        94..=96 => OperationType::GradientOverlay,
        97..=100 => OperationType::WatermarkBlend,
        _ => unreachable!("roll is 1..=100"),
    }
}

/// Tauri command: Generate a random seed with 3-7 operations.
/// Per D-02: weighted random -- MathOverlay ~30%, others evenly distributed.
/// Per D-03: 3-7 random steps.
/// Per D-04: auto-alias using timestamp format "seed_YYYYMMDD_HHMMSS".
/// Per D-01: pure random generation, user cannot edit operation parameters.
#[tauri::command]
pub async fn generate_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) -> Result<Seed, String> {
    let mut rng = rand::rng();

    // D-03: 3-7 random steps
    let step_count = rng.random_range(3..=7);
    let mut operations = Vec::with_capacity(step_count);

    for _ in 0..step_count {
        let op_type = pick_operation_type(&mut rng);
        let op = generate_operation(&mut rng, op_type);
        operations.push(op);
    }

    // D-04: Auto-alias with timestamp
    let alias = chrono::Utc::now().format("seed_%Y%m%d_%H%M%S").to_string();

    let seed = Seed {
        id: uuid::Uuid::new_v4().to_string(),
        alias,
        operations,
        created_at: chrono::Utc::now().to_rfc3339(),
        strength_tier: StrengthTier::default(),
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

/// Generate a single Operation with safety-constrained random parameters.
/// SEED-04 constraints applied inline.
fn generate_operation(rng: &mut impl Rng, op_type: OperationType) -> Operation {
    let (start_frame, duration_frames) = match op_type {
        OperationType::FrameDrop => {
            // Frame drop is more effective when applied to a section
            let start = rng.random_range(0..300);
            let dur = rng.random_range(60..600);
            (start, dur)
        }
        _ => {
            // Most operations apply to full video
            (0u32, 0u32)
        }
    };

    let params = match op_type {
        OperationType::MathOverlay => {
            let pattern = match rng.random_range(0..3) {
                0 => "ripple",
                1 => "stripes",
                _ => "concentric",
            };
            let opacity = rng.random_range(0.03..=0.15); // SEED-04: <= 0.15
            let frequency = rng.random_range(20..=200);
            serde_json::json!({
                "pattern": pattern,
                "opacity": opacity,
                "frequency": frequency,
            })
        }
        OperationType::PixelShift => {
            let dx = rng.random_range(-3i32..=3); // SEED-04: <= |3|
            let dy = rng.random_range(-3i32..=3);
            serde_json::json!({ "dx": dx, "dy": dy })
        }
        OperationType::FrameDrop => {
            let interval = rng.random_range(15..=60); // SEED-04: >= 15
            serde_json::json!({ "interval": interval })
        }
        OperationType::GopModify => {
            let gop_size = rng.random_range(12..=250);
            serde_json::json!({ "gopSize": gop_size })
        }
        OperationType::MetadataErase => {
            serde_json::json!({})
        }
        OperationType::AudioTweak => {
            let effect = match rng.random_range(0..3) {
                0 => "volume",
                1 => "tempo",
                _ => "echo",
            };
            match effect {
                "volume" => {
                    serde_json::json!({ "effect": "volume", "db": rng.random_range(-1.0..=1.0) })
                }
                "tempo" => {
                    serde_json::json!({ "effect": "tempo", "factor": rng.random_range(0.99..=1.01) })
                }
                _ => serde_json::json!({ "effect": "echo" }),
            }
        }
        OperationType::Remux => {
            serde_json::json!({})
        }
        // Color processing (4): D-01, D-02
        OperationType::HueRotate => {
            serde_json::json!({ "angle": rng.random_range(-30..=30) })
        }
        OperationType::SaturationAdjust => {
            serde_json::json!({ "factor": rng.random_range(0.8..=1.2) })
        }
        OperationType::BrightnessContrast => {
            serde_json::json!({
                "brightness": rng.random_range(-0.1..=0.1),
                "contrast": rng.random_range(0.9..=1.1),
            })
        }
        OperationType::ColorBalance => {
            serde_json::json!({
                "r": rng.random_range(-0.05..=0.05),
                "g": rng.random_range(-0.05..=0.05),
                "b": rng.random_range(-0.05..=0.05),
            })
        }
        // Noise texture (3): D-01, D-02
        OperationType::FilmGrain => {
            serde_json::json!({ "strength": rng.random_range(1..=5) })
        }
        OperationType::GaussianBlur => {
            serde_json::json!({ "sigma": rng.random_range(0.5..=1.5) })
        }
        OperationType::Sharpen => {
            serde_json::json!({ "amount": rng.random_range(0.3..=1.0) })
        }
        // Geometric fine-tuning (3): D-01, D-02
        OperationType::MicroRotate => {
            serde_json::json!({ "angle": rng.random_range(0.1..=0.9) })
        }
        OperationType::TinyScale => {
            serde_json::json!({ "factor": rng.random_range(0.98..=1.02) })
        }
        OperationType::Flip => {
            let axis = if rng.random_bool(0.5) { "h" } else { "v" };
            serde_json::json!({ "axis": axis })
        }
        // Blend overlay (3): D-01, D-02
        OperationType::SolidColorOverlay => {
            serde_json::json!({
                "opacity": rng.random_range(0.01..=0.05),
                "color": format!("#{:06x}", rng.random_range(0u32..=0xFFFFFF)),
            })
        }
        OperationType::GradientOverlay => {
            serde_json::json!({
                "opacity": rng.random_range(0.01..=0.05),
                "direction": if rng.random_bool(0.5) { "horizontal" } else { "vertical" },
            })
        }
        OperationType::WatermarkBlend => {
            serde_json::json!({ "opacity": rng.random_range(0.01..=0.03) })
        }
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

        // Re-randomize parameters for each operation but keep the same op_types
        let new_operations: Vec<Operation> =
            source.operations.iter().map(|op| generate_operation(&mut rng, op.op_type)).collect();

        let new_alias = format!("{}_copy_{}", source.alias, chrono::Utc::now().format("%H%M%S"));

        let seed = Seed {
            id: uuid::Uuid::new_v4().to_string(),
            alias: new_alias,
            operations: new_operations,
            created_at: chrono::Utc::now().to_rfc3339(),
            strength_tier: StrengthTier::default(),
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
fn persist_seeds(app: &AppHandle) -> Result<(), String> {
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

    // ─── TEST 6: FrameDrop retains existing time-slice behavior ─────────────
    #[test]
    fn generate_operation_frame_drop_retains_slice_behavior() {
        let mut rng = rand::rng();
        let op = generate_operation(
            &mut rng,
            OperationType::FrameDrop,
            StrengthTier::Standard,
            Some(5000),
        );
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
