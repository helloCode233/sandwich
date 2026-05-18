---
phase: 07-audio-crop-meta
plan: "06"
subsystem: migration
tags: [rust, tauri, seed-migration, audiotweak, framedrop, serde]

# Dependency graph
requires:
  - phase: "07-01"
    provides: "Phase 7 OperationType enum variants (AudioVolume, AudioPitch, etc.) and expanded Seed schema"
provides:
  - "seed_v3 migration: AudioTweak split into 5 independent audio types + FrameDrop re-parameterize to select-based"
  - "Idempotent startup migration via migration_v3_applied store marker"
  - "Import-time migration for old-format seed JSON files"
  - "Max ops cap raised from 20 to 30"
affects: ["07-audio-crop-meta", "seed-generation", "seed-import"]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Migration pattern: lock AppState, iterate seeds, persist via tauri_plugin_store, set marker for idempotency"
    - "Import migration pattern: transform individual seed before it enters the store"

key-files:
  created:
    - "src-tauri/src/migrations/seed_v3.rs"
  modified:
    - "src-tauri/src/migrations/mod.rs"
    - "src-tauri/src/lib.rs"
    - "src-tauri/src/commands/export_seed.rs"

key-decisions:
  - "AudioTweak echo sub-effect is dropped during migration (no Phase 7 equivalent, minimal fingerprint value)"
  - "AudioTweak tempo maps to AudioPitch with pitchFactor=1.0 (tempo-only, no pitch change — preserves old behavior)"
  - "FrameDrop setpts parameters replaced with random interval in 30-50 range (D-18)"
  - "Max operations cap raised from 20 to 30 to accommodate Phase 7's expanded op type count"

patterns-established:
  - "Migration v3 pattern: match on OperationType, transform params, set schema_version=3, persist via store API"
  - "Echo cleanup pattern: mark with __drop sentinel during iteration, then retain() to filter"

requirements-completed: [D-01, D-17, D-18, D-19]

# Metrics
duration: 4min
completed: 2026-05-18
---

# Phase 07 Plan 06: Seed v3 Migration — AudioTweak Split + FrameDrop Re-parameterize

**Idempotent seed migration splitting AudioTweak into 5 independent audio types, dropping echo operations, and converting FrameDrop from setpts to select-based interval**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-18T13:14:01Z
- **Completed:** 2026-05-18T13:17:51Z
- **Tasks:** 2
- **Files modified:** 4 (1 created, 3 modified)

## Accomplishments
- seed_v3 migration module: AudioTweak volume->AudioVolume, tempo->AudioPitch, echo->dropped
- FrameDrop setpts params (offset/period) re-parameterized to interval param (30-50 range per D-18)
- Idempotent migration via migration_v3_applied store marker (safe to run multiple times)
- Import-time migration for old-format seed JSON files (handles seeds exported from Phase 6 and earlier)
- Max operations cap raised from 20 to 30 to accommodate Phase 7's expanded operation types

## Task Commits

Each task was committed atomically:

1. **Task 1: Create seed_v3 migration module** - `78f798f` (feat)
2. **Task 2: Register migration in mod.rs, lib.rs, and export_seed.rs** - `8efe5d9` (feat)

## Files Created/Modified
- `src-tauri/src/migrations/seed_v3.rs` - Migration logic: AudioTweak split, FrameDrop re-parameterize, echo cleanup, idempotent via store marker
- `src-tauri/src/migrations/mod.rs` - Added `pub mod seed_v3` module declaration
- `src-tauri/src/lib.rs` - Phase 7 startup migration block (runs after Phase 6 v2 migration)
- `src-tauri/src/commands/export_seed.rs` - `migrate_imported_seed` function + call in import_seed for schema_version < 3, raised max ops to 30

## Decisions Made
- Followed plan's specified migration behavior: echo dropped with `__drop` sentinel + `retain()` cleanup
- Followed plan's import-time migration design: check `schema_version < 3` before UUID regeneration
- Maintained plan's use of `rand::rng()` for FrameDrop interval randomization (30-50 range per D-18)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `use rand::Rng;` import to seed_v3.rs**
- **Found during:** Task 1 (seed_v3.rs creation)
- **Issue:** Plan code used `rng.random_range()` which requires the `Rng` trait in scope, but the trait import was missing from the plan's import list
- **Fix:** Added `use rand::Rng;` to imports
- **Files modified:** src-tauri/src/migrations/seed_v3.rs
- **Verification:** `cargo check` passes cleanly
- **Committed in:** 78f798f (Task 1 commit)

**2. [Rule 1 - Bug] Updated test thresholds from 20 to 30 in export_seed.rs**
- **Found during:** Task 2 (export_seed.rs modifications)
- **Issue:** Test `test_import_seed_rejects_oversized_operations` and helper `make_oversized_seed` still referenced the old 20-operation cap. With the new 30 cap, the test's 25-operation seed would no longer trigger rejection, and the assertion `seed.operations.len() > 20` would fail for well-behaved seeds
- **Fix:** Updated threshold references from 20 to 30, updated `make_oversized_seed` to generate 35 operations (above the new 30 cap), updated test assertions and doc comment
- **Files modified:** src-tauri/src/commands/export_seed.rs
- **Verification:** `cargo check` passes, test logic now uses correct 30 cap
- **Committed in:** 8efe5d9 (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both auto-fixes necessary for compilation and test correctness. No scope creep.

## Issues Encountered
- Pre-commit hook (`lint-staged`) flagged formatting issues in seed_v3.rs on first commit attempt. Resolved by running `cargo fmt` before re-committing.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Migration infrastructure complete. Seeds with `AudioTweak` or old `FrameDrop` operations will be automatically updated on next app startup.
- Imported seeds from Phase 6 or earlier exports will be migrated during import.
- Ready for plan 07-05 (which may depend on migrated seed format).

---
*Phase: 07-audio-crop-meta*
*Plan: 06*
*Completed: 2026-05-18*
