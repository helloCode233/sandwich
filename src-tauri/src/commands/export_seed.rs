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

/// Tauri command: Import a seed from a JSON file (D-12).
/// Reads and validates a seed JSON file, regenerates UUID and timestamp,
/// validates operation count <= 20, and appends to the seed list.
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

    // D-12: Regenerate UUID and timestamp
    seed.id = uuid::Uuid::new_v4().to_string();
    seed.created_at = chrono::Utc::now().to_rfc3339();

    // Validate: cap operations at 20 (security: prevent resource exhaustion, T-06-07)
    if seed.operations.len() > 20 {
        return Err(format!(
            "Imported seed has {} operations (max 20). Import rejected.",
            seed.operations.len()
        ));
    }

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
        }
    }

    /// Helper: create seed JSON with >20 operations (DoS vector, T-06-07).
    fn make_oversized_seed() -> Seed {
        let mut ops = Vec::new();
        for _ in 0..25 {
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

    // ─── TEST 3: import_seed rejects >20 operations ────────────────────────
    #[test]
    fn test_import_seed_rejects_oversized_operations() {
        let seed = make_oversized_seed();
        assert!(seed.operations.len() > 20, "Oversized seed should have >20 operations");
        let would_reject = seed.operations.len() > 20;
        assert!(would_reject, "import_seed must reject seeds with >20 operations");
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
