---
phase: 07-audio-crop-meta
plan: 04
subsystem: seed-generation
tags: [seed, generate_operation, pick_operation_type, generate_seed, tier-driven, FrameDrop, Crop]
requires: ["07-01"]
provides: ["Seed generation with default ops + 10 new operation types"]
affects: ["07-02", "07-03", "07-05", "07-06"]
tech-stack:
  added: []
  patterns: [tier-driven-param-ranges, weighted-random-selection, pre-injection, select-based-decimation]
key-files:
  created: []
  modified:
    - src-tauri/src/commands/seed.rs
    - src-tauri/src/models/seed.rs
    - src-tauri/src/ffmpeg/filters.rs
    - src-tauri/src/commands/export_seed.rs
decisions:
  - "29 types in pick pool (30 total minus deprecated AudioTweak)"
  - "FrameDrop uses select-based interval (25-50), replaces setpts jitter (offset/period)"
  - "Crop + FrameDrop pre-injected as defaults, can also be randomly picked for dual guarantee"
  - "MetadataWrite and MetadataSelectiveErase skip strength tiers per D-13"
metrics:
  duration: "~15 min"
  completed_date: "2026-05-18"
---

# Phase 7 Plan 4: Seed Generation Extension Summary

**One-liner:** Extended seed generation to pre-inject Crop + FrameDrop as default operations, added 10 new generate_operation arms with tier-driven parameter ranges, and redistributed pick_operation_type weights for 29 pool types.

## Completed Tasks

| # | Name | Commit | Key Changes |
|---|------|--------|-------------|
| 1 | Redistribute weights in pick_operation_type | `3f45971` | Replaced entire pick_operation_type function: removed AudioTweak from pool, added 10 new types, redistributed all weights across 1000 buckets |
| 2 | Pre-inject Crop + FrameDrop + 11 new generate_operation arms | `ab563c7` | Added pre-injection of Crop + FrameDrop before random loop, Vec::with_capacity(step_count + 2), replaced old FrameDrop arm (setpts jitter) with select-based interval, added 10 new match arms (AudioResample, AudioVolume, AudioPitch, AudioEQ, AudioChannel, Crop, MetadataWrite, MetadataSelectiveErase, VideoSpeed, TrimEdges) |
| 3 | Update tests | `bcb69c2` | Extended variant_index to 30, updated pick_operation_type test to 29 pool types, replaced FrameDrop test (select filter verification), added 4 new tests |

## Verification

- `cargo check` — passes (pre-existing warnings only)
- `cargo test --lib commands::seed::tests` — 16 passed, 0 failed
- `pick_operation_type_covers_all_29_pool_types` — verifies all 29 pool types appear in 10k random draws
- `generate_operation_frame_drop_uses_select_filter` — verifies select-based interval, no setpts jitter
- All new operation arms generate valid tier-driven params

## Deviations from Plan

### Rule 3 - Missing Dependency (Plan 07-01)

**1. [Rule 3 - Missing dependency] Added OperationType enum variants (10 new Phase 7 types)**
- **Found during:** Task 1
- **Issue:** Worktree reset to pre-07-01 commit `da55cd4`. The 10 new OperationType variants (AudioResample, AudioVolume, AudioPitch, AudioEQ, AudioChannel, Crop, MetadataWrite, MetadataSelectiveErase, VideoSpeed, TrimEdges) were missing from the enum, causing compilation failures.
- **Fix:** Added all 10 variants to `src-tauri/src/models/seed.rs` OperationType enum with doc comments. Updated test from `operation_type_has_20_variants` to `operation_type_has_30_variants`.
- **Files modified:** `src-tauri/src/models/seed.rs`

**2. [Rule 3 - Missing dependency] Added schema_version field to Seed struct**
- **Found during:** Task 1
- **Issue:** The `Seed` struct in the worktree was missing the `schema_version` field (added by Plan 07-01), causing compilation errors in `generate_seed` and `copy_seed`.
- **Fix:** Added `schema_version: u32` with `#[serde(default)]` to the Seed struct. Added `schema_version: 3` to all Seed constructions in `commands/seed.rs` and `commands/export_seed.rs`.
- **Files modified:** `src-tauri/src/models/seed.rs`, `src-tauri/src/commands/export_seed.rs`

**3. [Rule 3 - Missing dependency] Added catch-all match arms for new enum variants**
- **Found during:** Task 1
- **Issue:** The `generate_operation` match and `build_filter_args` match lacked catch-all arms for the 10 new OperationType variants.
- **Fix:** Added `_ => serde_json::json!({})` catch-all to `generate_operation` (later replaced by explicit arms in Task 2). Added `_ => Ok(vec![])` catch-all to `build_filter_args`.
- **Files modified:** `src-tauri/src/commands/seed.rs`, `src-tauri/src/ffmpeg/filters.rs`

## Known Stubs

None. All 30 OperationType variants have explicit match arms in generate_operation. The remaining catch-all `_ => serde_json::json!({})` is only a future-proof guard for variants not yet implemented.

## Threat Flags

None. All threat surface is within the plan's threat model. The threat register (T-07-04-01 through T-07-04-03) covers all introduced surfaces.

## Self-Check: PASSED

- [x] All 3 task commits exist: `3f45971`, `ab563c7`, `bcb69c2`
- [x] SUMMARY.md created at `.planning/phases/07-audio-crop-meta/07-04-SUMMARY.md`
- [x] All 16 tests pass (0 failures)
- [x] `cargo check` passes
- [x] All deleted/modified files tracked in commits
