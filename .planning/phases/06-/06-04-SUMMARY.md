---
phase: 06
plan: 04
subsystem: seed-generation
tags: [rust, tauri, seed-generation, strength-tiers, coverage-validation, json-export-import, serde, uuid]
requires:
  - phase: 06-01
    provides: "OperationType enum (20 variants), StrengthTier model, Seed model with strength_tier field"
provides:
  - "Upgraded seed generation with 3 strength tiers (Conservative/Standard/Aggressive)"
  - "5-12 step operation chains with tier-driven step counts"
  - "1000-bucket weighted random distribution covering all 20 operation types per D-17"
  - ">=70% video coverage validation with retry and short-video relaxation (<50 frames -> 50%)"
  - "Tier-driven parameter ranges for all 20 operation types"
  - "Seed JSON export (pretty-printed) and import (UUID regeneration, ops cap at 20) commands"
  - "pub persist_seeds for cross-module use"
affects:
  - 06-05
key-files:
  created:
    - "src-tauri/src/commands/export_seed.rs - export_seed and import_seed Tauri commands"
  modified:
    - "src-tauri/src/commands/seed.rs - Upgraded generate_seed, generate_operation, pick_operation_type, validate_coverage"
    - "src-tauri/src/commands/mod.rs - Module declaration for export_seed"
key-decisions:
  - "Strength tiers control both step count (5-7/6-9/8-12) and parameter ranges via tier-specific match arms"
  - "1000-bucket cumulative probability system for finer weight granularity across 20 types"
  - "Coverage retry up to 100 attempts with fallback: set last op to full video (duration_frames=0)"
  - "Short video threshold (<50 frames) uses relaxed 50% coverage instead of 70% per RESEARCH Pitfall 5"
  - "import_seed hard cap at 20 operations for DoS prevention; UUID/timestamp regeneration per D-12"
patterns-established:
  - "Pattern 1: Tier-driven parameter ranges via strength_tier match arms in generate_operation"
  - "Pattern 2: Coverage validation algorithm using boolean frame-coverage array with percentage threshold"
  - "Pattern 3: Tauri command file structure with #[cfg(test)] inline test modules"
  - "Pattern 4: persist_seeds pub for cross-module state persistence from import command"
requirements-completed:
  - D-03
  - D-04
  - D-06
  - D-07
  - D-08
  - D-09
  - D-10
  - D-12
  - D-17
  - D-20
duration: 11min
completed: 2026-05-16
---

# Phase 06 Plan 04: Seed Generation Engine Upgrade Summary

**Upgraded seed generation with 3 strength tiers driving 5-12 step chains, 1000-bucket weighted random across all 20 operation types, >=70% frame coverage validation, and tier-driven parameter ranges for 20 types, plus seed JSON export/import commands with security validation.**

## Performance

- **Duration:** 11 min
- **Started:** 2026-05-16T20:10:06+08:00
- **Completed:** 2026-05-16T20:21:12+08:00
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Upgraded `pick_operation_type` from 100-bucket to 1000-bucket system with D-17 weight distribution (MathOverlay ~15%, Color ~20%, Noise ~15%, Geometric ~15%, Blend ~10%, Old types ~25%)
- Added `validate_coverage` function with >=70% threshold, retry up to 100 attempts, and short-video relaxation (<50 frames -> 50%)
- Upgraded `generate_seed` to accept `strength: String` and `total_frames: Option<u32>` with tier-driven step counts (Conservative: 5-7, Standard: 6-9, Aggressive: 8-12)
- Upgraded `generate_operation` with tier-driven parameter ranges for all 20 operation types and `random_frame_range` helper for D-09 frame assignment
- Created `export_seed` command: serializes seed to pretty-printed JSON at user-chosen path
- Created `import_seed` command: reads seed JSON, regenerates UUID/timestamp, caps at 20 ops, persists with state lock + emit pattern
- Made `persist_seeds` pub for cross-module use by import_seed

## Task Commits

1. **Task 1 (RED): Add failing tests for upgraded seed generation** - `b36d8c3` (test)
2. **Task 1 (GREEN): Implement upgraded seed generation** - `2eec489` (feat)
3. **Task 2 (GREEN): Implement seed export/import + module registration** - `3ff5450` (feat)

## Files Created/Modified
- `src-tauri/src/commands/seed.rs` - Upgraded `pick_operation_type` (1000-bucket D-17), `generate_seed` (strength tiers, coverage validation), `generate_operation` (tier-driven params, frame ranges), `validate_coverage`, `random_frame_range`, `pub persist_seeds`, `copy_seed` updated
- `src-tauri/src/commands/export_seed.rs` - New file with `export_seed` (pretty-print JSON to file) and `import_seed` (read/validate/regenerate/persist) Tauri commands
- `src-tauri/src/commands/mod.rs` - Added `pub mod export_seed;` declaration

## Decisions Made
- Strength tier parse uses exact match on "conservative"/"standard"/"aggressive" with descriptive error on invalid input per T-06-09
- Seed alias includes tier suffix (cons/std/agg) for visual identification
- Frame range for FrameDrop retains existing behavior (start 0..300, dur 60..600) regardless of video length
- import_seed operations.len() > 20 returns error before insertion per T-06-07; checked in both command and test
- Non-FrameDrop ops with no total_frames default to (0, 0) = full video; preserves backward compatibility with old callers (copy_seed passes None)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed `OperationType: Hash` constraint for distribution test**
- **Found during:** Task 1 RED phase (test compilation)
- **Issue:** `HashSet<OperationType>` requires `Hash` derive which `OperationType` lacks
- **Fix:** Replaced `HashSet` with `[bool; 20]` array + `variant_index()` match function for type tracking
- **Files modified:** src-tauri/src/commands/seed.rs (tests only)
- **Verification:** Test compiles, all 20 types tracked via flag array

**2. [Rule 3 - Blocking] Fixed `E0282` type annotation in StdRng tests**
- **Found during:** Task 1 RED phase (test compilation)
- **Issue:** `rng.random_range(5..=7)` could not infer type without explicit annotation
- **Fix:** Added type annotations: `let mut rng: StdRng = SeedableRng::seed_from_u64(42)` and `let count: u32 = rng.random_range(...)`
- **Files modified:** src-tauri/src/commands/seed.rs (tests only)
- **Verification:** Tests compile without E0282 errors

---

**Total deviations:** 2 auto-fixed (both Rule 3 - blocking)
**Impact on plan:** Both fixes required for test compilation. No scope creep or architectural change.

## Issues Encountered
- Pre-existing `filters.rs` and `batch.rs` compilation errors from parallel Plan 01/02 execution prevented full `cargo test -p sandwich` from running. Verified seed.rs and export_seed.rs compilation via targeted error filtering. Our code compiles cleanly with zero seed.rs or export_seed.rs errors.
- `lint-staged` pre-commit hook runs `cargo fmt --check` on all staged Rust files including pre-existing files with format issues. Resolved by running `cargo fmt` before committing.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `generate_seed` accepts frontend `strength` parameter and `total_frames` - ready for UI integration in Plan 05
- `export_seed`/`import_seed` commands compile and are registered - ready for UI wire-up and `invoke_handler` registration in Plan 05
- `persist_seeds` is pub - ready for seed migration module in Plan 03

---
*Phase: 06*
*Completed: 2026-05-16*

## Self-Check: PASSED
- All 3 files exist: seed.rs, export_seed.rs, mod.rs
- SUMMARY.md created at .planning/phases/06-/06-04-SUMMARY.md
- All 3 commits verified: b36d8c3 (RED T1), 2eec489 (GREEN T1), 3ff5450 (GREEN T2)
