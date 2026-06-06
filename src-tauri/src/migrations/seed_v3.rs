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

/// Apply Phase 7 migration transformations to a list of operations.
/// Pure function — no Tauri dependencies. Testable in unit tests.
/// Mirrors the transformation logic in migrate_seeds (lines 46-101).
#[allow(dead_code)]
pub fn transform_operations(operations: &mut Vec<crate::models::seed::Operation>) -> usize {
    let mut count = 0usize;
    let mut rng = rand::rng();

    for op in operations.iter_mut() {
        match op.op_type {
            OperationType::AudioTweak => {
                let effect = op.params["effect"].as_str().unwrap_or("volume");
                match effect {
                    "volume" => {
                        let db = op.params["db"].as_f64().unwrap_or(0.5);
                        op.op_type = OperationType::AudioVolume;
                        op.params = serde_json::json!({ "db": db });
                        count += 1;
                    }
                    "tempo" => {
                        op.op_type = OperationType::AudioPitch;
                        op.params = serde_json::json!({
                            "pitchFactor": 1.0,
                            "originalRate": 48000,
                        });
                        count += 1;
                    }
                    "echo" => {
                        op.params = serde_json::json!({ "__drop": true });
                        count += 1;
                    }
                    _ => {}
                }
            }
            OperationType::FrameDrop
                if op.params.get("offset").is_some() || op.params.get("period").is_some() =>
            {
                let interval = rng.random_range(30u32..=50u32);
                op.params = serde_json::json!({ "interval": interval });
                count += 1;
            }
            _ => {}
        }
    }

    // Remove echo operations
    operations.retain(|op| !op.params.get("__drop").and_then(|v| v.as_bool()).unwrap_or(false));

    count
}

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
    use super::*;
    use crate::models::seed::{Operation, OperationType};
    use serde_json::json;

    fn make_op(op_type: OperationType, params: serde_json::Value) -> Operation {
        Operation { op_type, start_frame: 0, duration_frames: 300, params }
    }

    /// AudioTweak volume -> AudioVolume: db value preserved.
    #[test]
    fn migrate_audio_tweak_volume_to_audio_volume() {
        let mut ops =
            vec![make_op(OperationType::AudioTweak, json!({"effect": "volume", "db": 1.2}))];
        let count = transform_operations(&mut ops);
        assert_eq!(count, 1, "One operation should be transformed");
        assert_eq!(ops.len(), 1, "Operation should not be dropped");
        assert_eq!(
            ops[0].op_type,
            OperationType::AudioVolume,
            "AudioTweak(volume) should become AudioVolume"
        );
        assert!(
            (ops[0].params["db"].as_f64().unwrap() - 1.2).abs() < 0.001,
            "db value should be preserved"
        );
        assert!(ops[0].params.get("effect").is_none(), "effect field should be removed");
    }

    /// AudioTweak tempo -> AudioPitch: pitchFactor=1.0, originalRate=48000.
    #[test]
    fn migrate_audio_tweak_tempo_to_audio_pitch() {
        let mut ops =
            vec![make_op(OperationType::AudioTweak, json!({"effect": "tempo", "factor": 1.01}))];
        let count = transform_operations(&mut ops);
        assert_eq!(count, 1);
        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0].op_type,
            OperationType::AudioPitch,
            "AudioTweak(tempo) should become AudioPitch"
        );
        assert!(
            (ops[0].params["pitchFactor"].as_f64().unwrap() - 1.0).abs() < 0.001,
            "pitchFactor should be 1.0 (tempo-only, no pitch change)"
        );
        assert_eq!(ops[0].params["originalRate"].as_u64().unwrap(), 48000);
    }

    /// AudioTweak echo -> dropped (no Phase 7 equivalent).
    #[test]
    fn migrate_audio_tweak_echo_dropped() {
        let mut ops = vec![make_op(OperationType::AudioTweak, json!({"effect": "echo"}))];
        let count = transform_operations(&mut ops);
        assert_eq!(count, 1, "Echo op should count as migrated (dropped)");
        assert!(ops.is_empty(), "Echo operation should be removed entirely");
    }

    /// FrameDrop setpts (offset/period) -> select-based interval.
    #[test]
    fn migrate_frame_drop_setpts_to_select_interval() {
        let mut ops =
            vec![make_op(OperationType::FrameDrop, json!({"offset": 0.003, "period": 45}))];
        let count = transform_operations(&mut ops);
        assert_eq!(count, 1);
        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0].op_type,
            OperationType::FrameDrop,
            "FrameDrop should remain FrameDrop after re-parameterize"
        );
        let interval = ops[0].params["interval"].as_u64().unwrap();
        assert!(
            interval >= 30 && interval <= 50,
            "Interval should be in D-18 range 30..50, got {}",
            interval
        );
        assert!(ops[0].params.get("offset").is_none(), "offset param should be removed");
        assert!(ops[0].params.get("period").is_none(), "period param should be removed");
    }

    /// New FrameDrop (already has interval) -> NOT re-migrated.
    #[test]
    fn migrate_frame_drop_already_select_based_not_remigrated() {
        let mut ops = vec![make_op(OperationType::FrameDrop, json!({"interval": 40}))];
        let count = transform_operations(&mut ops);
        assert_eq!(count, 0, "Already-migrated FrameDrop should not be touched");
        assert_eq!(
            ops[0].params["interval"].as_u64().unwrap(),
            40,
            "Existing interval should be preserved"
        );
    }

    /// Mixed batch: volume + tempo + echo + old FrameDrop + new FrameDrop.
    #[test]
    fn migrate_mixed_phase6_operations() {
        let mut ops = vec![
            make_op(OperationType::AudioTweak, json!({"effect": "volume", "db": 1.5})),
            make_op(OperationType::AudioTweak, json!({"effect": "tempo", "factor": 1.02})),
            make_op(OperationType::AudioTweak, json!({"effect": "echo"})),
            make_op(OperationType::FrameDrop, json!({"offset": 0.002, "period": 40})),
            make_op(OperationType::FrameDrop, json!({"interval": 35})),
            make_op(
                OperationType::Crop,
                json!({"leftPct": 1.0, "rightPct": 1.5, "topPct": 0.8, "bottomPct": 1.2}),
            ),
        ];
        let count = transform_operations(&mut ops);
        // volume(1) + tempo(1) + echo(1) + old FrameDrop(1) = 4 transformed
        // new FrameDrop(0) + Crop(0) = 0 transformed
        assert_eq!(
            count, 4,
            "Should transform 4 operations (volume + tempo + echo + old FrameDrop)"
        );
        // echo dropped → 6 - 1 = 5 remaining
        assert_eq!(ops.len(), 5, "5 operations remain after echo drop (6 - 1)");

        // Verify each operation's final type
        assert_eq!(ops[0].op_type, OperationType::AudioVolume, "Op 0: volume → AudioVolume");
        assert_eq!(ops[1].op_type, OperationType::AudioPitch, "Op 1: tempo → AudioPitch");
        assert_eq!(ops[2].op_type, OperationType::FrameDrop, "Op 2: old FrameDrop → FrameDrop");
        assert_eq!(ops[3].op_type, OperationType::FrameDrop, "Op 3: new FrameDrop unchanged");
        assert_eq!(ops[4].op_type, OperationType::Crop, "Op 4: Crop unchanged");

        // Verify old FrameDrop got re-parameterized
        assert!(ops[2].params.get("interval").is_some());
        assert!(ops[2].params.get("offset").is_none());
    }
}
