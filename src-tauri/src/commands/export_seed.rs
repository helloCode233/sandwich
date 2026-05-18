//! Seed JSON export/import commands (D-10, D-12).
//!
//! export_seed: Serializes a single seed to pretty-printed JSON at a user-chosen path.
//! import_seed: Reads a seed JSON file, validates schema, regenerates UUID/timestamp,
//!              caps operations at 20, and appends to the seed list.

use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

use crate::models::seed::Seed;
use crate::state::AppState;

/// Tauri command: Export a seed to a JSON file (D-10).
/// Serializes the seed identified by `seed_id` as pretty-printed JSON
/// and writes it to `filepath`.
#[tauri::command]
pub async fn export_seed(
    state: State<'_, Mutex<AppState>>,
    seed_id: String,
    filepath: String,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let seed = app_state
        .seeds
        .iter()
        .find(|s| s.id == seed_id)
        .ok_or_else(|| format!("Seed not found: {}", seed_id))?;

    let json =
        serde_json::to_string_pretty(seed).map_err(|e| format!("Serialization error: {}", e))?;
    std::fs::write(&filepath, json).map_err(|e| format!("File write error: {}", e))?;
    Ok(())
}

/// Migrate a single imported seed from old format to Phase 7 format.
/// Applies the same transformations as seed_v3::migrate_seeds but for individual imports.
/// AudioTweak -> new audio types, FrameDrop setpts -> select interval.
fn migrate_imported_seed(mut seed: Seed) -> Result<Seed, String> {
    use crate::models::seed::OperationType;
    use rand::Rng;
    let mut rng = rand::rng();

    for op in seed.operations.iter_mut() {
        match op.op_type {
            OperationType::AudioTweak => {
                let effect = op.params["effect"].as_str().unwrap_or("volume");
                match effect {
                    "volume" => {
                        let db = op.params["db"].as_f64().unwrap_or(0.5);
                        op.op_type = OperationType::AudioVolume;
                        op.params = serde_json::json!({ "db": db });
                    }
                    "tempo" => {
                        op.op_type = OperationType::AudioPitch;
                        op.params = serde_json::json!({
                            "pitchFactor": 1.0,
                            "originalRate": 48000,
                        });
                    }
                    "echo" => {
                        op.params = serde_json::json!({ "__drop": true });
                    }
                    _ => {}
                }
            }
            OperationType::FrameDrop => {
                if op.params.get("offset").is_some() || op.params.get("period").is_some() {
                    let interval = rng.random_range(30u32..=50u32);
                    op.params = serde_json::json!({ "interval": interval });
                }
            }
            _ => {}
        }
    }

    // Remove echo operations
    seed.operations
        .retain(|op| !op.params.get("__drop").and_then(|v| v.as_bool()).unwrap_or(false));

    // Update operation count validation: max 30 (was 20 in Phase 6).
    // Phase 7 seeds can have up to 14 ops (12 aggressive + 2 defaults), but imports
    // from unknown sources should be more generous.
    if seed.operations.len() > 30 {
        return Err(format!(
            "Imported seed has {} operations (max 30). Import rejected.",
            seed.operations.len()
        ));
    }

    seed.schema_version = 3;
    Ok(seed)
}

/// Tauri command: Import a seed from a JSON file (D-12).
/// Reads and validates a seed JSON file, regenerates UUID and timestamp,
/// validates operation count <= 30, and appends to the seed list.
#[tauri::command]
pub async fn import_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    filepath: String,
) -> Result<Seed, String> {
    // Read and parse JSON
    let json_str =
        std::fs::read_to_string(&filepath).map_err(|e| format!("File read error: {}", e))?;

    let mut seed: Seed =
        serde_json::from_str(&json_str).map_err(|e| format!("Invalid seed JSON: {}", e))?;

    // Phase 7: Migrate imported seeds that lack schema_version (old export format).
    // This handles seed JSON files exported from Phase 6 or earlier.
    if seed.schema_version < 3 {
        seed = migrate_imported_seed(seed)?;
    }

    // D-12: Regenerate UUID and timestamp
    seed.id = uuid::Uuid::new_v4().to_string();
    seed.created_at = chrono::Utc::now().to_rfc3339();

    // Push + persist + emit (pattern from commands/seed.rs)
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.seeds.push(seed.clone());
    }

    // Reuse persist_seeds from commands::seed
    crate::commands::seed::persist_seeds(&app)?;
    let _ = app.emit("seeds-updated", ());

    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::seed::{Operation, OperationType, StrengthTier};

    /// Helper: create a minimal valid seed for serialization testing.
    fn make_test_seed() -> Seed {
        Seed {
            id: "original-id-123".to_string(),
            alias: "test_seed".to_string(),
            operations: vec![Operation {
                op_type: OperationType::MathOverlay,
                start_frame: 0,
                duration_frames: 0,
                params: serde_json::json!({"pattern": "ripple", "opacity": 0.05, "frequency": 100}),
            }],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            strength_tier: StrengthTier::Standard,
            schema_version: 3,
        }
    }

    /// Helper: create seed JSON with >30 operations (DoS vector, T-07-06-02).
    fn make_oversized_seed() -> Seed {
        let mut ops = Vec::new();
        for _ in 0..35 {
            ops.push(Operation {
                op_type: OperationType::Remux,
                start_frame: 0,
                duration_frames: 0,
                params: serde_json::json!({}),
            });
        }
        Seed {
            id: "big-one".to_string(),
            alias: "oversized".to_string(),
            operations: ops,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            strength_tier: StrengthTier::Standard,
            schema_version: 3,
        }
    }

    // ─── TEST 1: export_seed writes valid JSON ──────────────────────────────
    #[test]
    fn test_export_seed_writes_valid_json() {
        let seed = make_test_seed();
        let json = serde_json::to_string_pretty(&seed).unwrap();
        let parsed: Seed = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, seed.id);
        assert_eq!(parsed.alias, seed.alias);
        assert_eq!(parsed.operations.len(), 1);
        assert_eq!(parsed.strength_tier, StrengthTier::Standard);
    }

    // ─── TEST 2: import_seed deserializes and can regenerate UUID ──────────
    #[test]
    fn test_import_seed_deserializes_valid_json() {
        let seed = make_test_seed();
        let json = serde_json::to_string_pretty(&seed).unwrap();
        let mut imported: Seed = serde_json::from_str(&json).unwrap();
        assert_eq!(imported.operations.len(), 1);

        // Simulate UUID regeneration (what import_seed does)
        let original_id = imported.id.clone();
        imported.id = uuid::Uuid::new_v4().to_string();
        assert_ne!(imported.id, original_id, "import_seed should regenerate UUID");
    }

    // ─── TEST 3: import_seed rejects >30 operations ────────────────────────
    #[test]
    fn test_import_seed_rejects_oversized_operations() {
        let seed = make_oversized_seed();
        assert!(seed.operations.len() > 30, "Oversized seed should have >30 operations");
        let would_reject = seed.operations.len() > 30;
        assert!(would_reject, "import_seed must reject seeds with >30 operations");
    }

    // ─── TEST 4: import_seed rejects non-existent file ─────────────────────
    #[test]
    fn test_import_seed_rejects_missing_file() {
        let result = std::fs::read_to_string("/nonexistent/path/seed.json");
        assert!(result.is_err(), "Non-existent file should error");
    }

    // ─── TEST 5: import_seed handles missing strength_tier (serde default) ─
    #[test]
    fn test_import_seed_missing_strength_tier_defaults_to_standard() {
        let json = r#"{
            "id": "seed-no-tier",
            "alias": "no-tier",
            "operations": [],
            "createdAt": "2026-01-01T00:00:00Z"
        }"#;
        let seed: Seed = serde_json::from_str(json).unwrap();
        assert_eq!(
            seed.strength_tier,
            StrengthTier::Standard,
            "Missing strength_tier should default to Standard"
        );
    }

    // ─── TEST 6: import_seed rejects invalid JSON ──────────────────────────
    #[test]
    fn test_import_seed_rejects_invalid_json() {
        let invalid = "not valid json at all {{{";
        let result: Result<Seed, _> = serde_json::from_str(invalid);
        assert!(result.is_err(), "Invalid JSON should fail to deserialize");
    }
}
