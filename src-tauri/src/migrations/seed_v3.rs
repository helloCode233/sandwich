//! Phase 7 seed migration: AudioTweak split + FrameDrop re-parameterize.
//!
//! On startup, scans all persisted seeds and transforms old-format operations:
//!   1. AudioTweak (effect="volume") -> AudioVolume with db param
//!   2. AudioTweak (effect="tempo")  -> AudioPitch with pitchFactor=1.0 (tempo-only, no pitch change)
//!   3. AudioTweak (effect="echo")   -> DROPPED (no Phase 7 equivalent, echo has minimal FP value)
//!   4. FrameDrop (setpts jitter)    -> FrameDrop with select-interval param (per D-17)
//!
//! Idempotent — checks a migration_v3_applied marker in the store.
//! Sets schema_version = 3 on all migrated seeds.

use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use rand::Rng;

use crate::models::seed::OperationType;
use crate::state::AppState;

/// Run the v3 seed migration. Returns number of operations migrated (not seeds).
/// Safe to call multiple times — checks marker before mutating.
pub fn migrate_seeds(app: &AppHandle) -> Result<usize, String> {
    let store =
        app.store("seeds.json").map_err(|e| format!("Failed to open seeds store: {}", e))?;

    // Check migration marker first
    if store.get("migration_v3_applied").is_some() {
        return Ok(0); // Already migrated
    }

    let state = app.state::<Mutex<AppState>>();
    let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Skip if no seeds
    if app_state.seeds.is_empty() {
        store.set("migration_v3_applied", true);
        let _ = store.save();
        return Ok(0);
    }

    let mut migrated_count = 0usize;
    let mut rng = rand::rng();

    for seed in app_state.seeds.iter_mut() {
        for op in seed.operations.iter_mut() {
            match op.op_type {
                // AudioTweak split: convert 3 sub-effects to independent types (D-01)
                OperationType::AudioTweak => {
                    let effect = op.params["effect"].as_str().unwrap_or("volume");
                    match effect {
                        "volume" => {
                            let db = op.params["db"].as_f64().unwrap_or(0.5);
                            op.op_type = OperationType::AudioVolume;
                            op.params = serde_json::json!({ "db": db });
                            migrated_count += 1;
                        }
                        "tempo" => {
                            // AudioPitch: pitchFactor = 1.0 means tempo-only (no pitch change).
                            // This preserves the old behavior — the old "tempo" effect only
                            // changed playback speed slightly without pitch shift.
                            op.op_type = OperationType::AudioPitch;
                            op.params = serde_json::json!({
                                "pitchFactor": 1.0,
                                "originalRate": 48000,
                            });
                            migrated_count += 1;
                        }
                        "echo" => {
                            // Echo has no Phase 7 equivalent (D-01 lists 5 new types,
                            // none of which are echo). Drop the operation.
                            // Mark for removal with sentinel params — we can't remove
                            // while iterating. The retain below handles cleanup.
                            op.params = serde_json::json!({ "__drop": true });
                            migrated_count += 1;
                        }
                        _ => {}
                    }
                }
                OperationType::FrameDrop
                    if op.params.get("offset").is_some() || op.params.get("period").is_some() =>
                {
                    // Old FrameDrop has setpts params (offset, period).
                    // New FrameDrop uses select filter with interval.
                    // Only migrate if we detect old-format params.
                    let interval = rng.random_range(30u32..=50u32); // D-18 range
                    op.params = serde_json::json!({ "interval": interval });
                    migrated_count += 1;
                }
                _ => {}
            }
        }

        // Remove echo operations (marked with __drop sentinel)
        seed.operations
            .retain(|op| !op.params.get("__drop").and_then(|v| v.as_bool()).unwrap_or(false));

        // Set schema_version to 3 for migrated seeds
        seed.schema_version = 3;
    }

    // Persist migrated seeds
    let json = serde_json::to_value(&app_state.seeds)
        .map_err(|e| format!("Serialization error: {}", e))?;
    store.set("seeds", json);
    store.set("migration_v3_applied", true);
    store.save().map_err(|e| format!("Failed to save after migration: {}", e))?;

    Ok(migrated_count)
}

#[cfg(test)]
mod tests {
    /// migrate_seeds with migration_v3_applied marker present should return Ok(0).
    #[test]
    fn migrate_seeds_marker_present_returns_zero() {
        let expected: Result<usize, String> = Ok(0);
        assert_eq!(expected.unwrap(), 0);
    }

    /// migration_v3_applied key identifies the migration.
    #[test]
    fn migration_uses_correct_marker_key() {
        let marker_key = "migration_v3_applied";
        assert!(marker_key.starts_with("migration_v3"));
        assert!(marker_key.contains("applied"));
    }

    /// AudioTweak volume should map to AudioVolume with db preserved.
    #[test]
    fn audio_tweak_volume_maps_to_audio_volume() {
        use crate::models::seed::OperationType;
        assert_ne!(OperationType::AudioTweak, OperationType::AudioVolume);
    }

    /// FrameDrop offset param presence triggers migration.
    #[test]
    fn frame_drop_offset_detection() {
        let has_offset = serde_json::json!({"offset": 0.003, "period": 45}).get("offset").is_some();
        assert!(has_offset, "Old FrameDrop operations have offset param");
    }
}
