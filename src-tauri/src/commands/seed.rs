use rand::prelude::*;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use crate::models::seed::{Operation, OperationType, Seed};
use crate::state::AppState;

/// Select an operation type using weighted random selection.
/// D-02: MathOverlay ~30%, remaining 6 types: 12, 12, 12, 12, 11, 11 (sum 100).
/// Uses cumulative probability threshold — rand 0.9 compatible (avoids WeightedIndex API drift).
fn pick_operation_type(rng: &mut impl Rng) -> OperationType {
    // Weights: MathOverlay=30, PixelShift=12, FrameDrop=12,
    //          GopModify=12, MetadataErase=12, AudioTweak=11, Remux=11
    let roll: u32 = rng.random_range(1..=100);
    match roll {
        1..=30 => OperationType::MathOverlay,
        31..=42 => OperationType::PixelShift,
        43..=54 => OperationType::FrameDrop,
        55..=66 => OperationType::GopModify,
        67..=78 => OperationType::MetadataErase,
        79..=89 => OperationType::AudioTweak,
        90..=100 => OperationType::Remux,
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
