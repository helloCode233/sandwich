---
phase: 02-rust-backend
plan: 03
subsystem: commands
tags: [seed, queue, crud, persistence, weighted-random]
requires: [02-01]
provides: [8 Tauri IPC commands, store persistence]
affects: [src-tauri/src/commands/seed.rs, src-tauri/src/commands/queue.rs, src-tauri/src/state.rs]
tech-stack:
  added: []
  patterns: [tauri::command, Mutex<AppState>, tauri-plugin-store write-through, event emission, weighted random selection]
key-files:
  created:
    - src-tauri/src/commands/seed.rs
    - src-tauri/src/commands/queue.rs
  modified:
    - src-tauri/src/state.rs
decisions:
  - "Replaced rand::WeightedIndex with manual cumulative-probability selection due to rand 0.9 API change (WeightedIndex moved/removed from distributions module)"
  - "Simplified AppState per-field Mutex to plain Vec fields (removed redundant double-Mutex wrapping from Plan 01)"
metrics:
  duration: ~5min
  completed_date: "2026-05-13"
  task_count: 2
  file_count: 3
---

# Phase 2 Plan 3: Seed and Queue IPC Commands Summary

Seed management (5 Tauri commands with weighted random generation) and video queue management (3 Tauri commands with D-06 path validity checking). All 8 commands use tauri-plugin-store write-through persistence and emit frontend events.

## Tasks Executed

### Task 1: Seed generation and CRUD commands (seed.rs)
**Commit:** `9280255` (`feat(02-rust-backend-03): implement seed generation and CRUD commands`)

Created 5 Tauri commands:
- `generate_seed` -- D-02 weighted random (MathOverlay ~30%, others 11-12%), D-03 3-7 steps, D-04 timestamp alias
- `rename_seed` -- In-place alias update with non-empty validation
- `delete_seed` -- Remove by ID with not-found error
- `copy_seed` -- Copy with re-randomized params per D-01
- `list_seeds` -- Return all seeds

All mutations persist to `seeds.json` via tauri-plugin-store and emit `seeds-updated`.

### Task 2: Video queue management commands (queue.rs)
**Commit:** `f5b0b56` (`feat(02-rust-backend-03): implement video queue management commands`)

Created 3 Tauri commands:
- `get_queue` -- D-06 path validity check, marks Invalid entries preserving metadata
- `remove_from_queue` -- Remove by index with bounds check and descriptive error
- `clear_queue` -- Clear entire queue

All mutations persist to `queue.json` via tauri-plugin-store and emit `queue-updated`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] AppState double-Mutex prevented field access**
- **Found during:** Task 1 compilation
- **Issue:** `AppState.seeds` and `AppState.queue` were `Mutex<Vec<T>>` inside a `Mutex<AppState>`, creating double-locking that prevented direct method calls like `.push()`, `.iter()`, `.len()`, etc.
- **Fix:** Changed `seeds: Mutex<Vec<Seed>>` to `seeds: Vec<Seed>` and `queue: Mutex<Vec<VideoEntry>>` to `queue: Vec<VideoEntry>` in state.rs. The outer `Mutex<AppState>` (via `tauri::State`) already serializes all access.
- **Files modified:** `src-tauri/src/state.rs`
- **Commit:** `9280255` (included in Task 1 commit)

**2. [Rule 1 - Bug] rand 0.9 WeightedIndex unavailable**
- **Found during:** Task 1 compilation
- **Issue:** `rand::distributions::WeightedIndex` does not exist in rand 0.9 (module renamed from `distributions` to `distr`, and `WeightedIndex` API changed).
- **Fix:** Replaced with manual cumulative-probability selection using `rng.random_range(1..=100)` with match arms: 1-30 MathOverlay, 31-42 PixelShift, 43-54 FrameDrop, 55-66 GopModify, 67-78 MetadataErase, 79-89 AudioTweak, 90-100 Remux. Functionally equivalent to the original weighted distribution.
- **Files modified:** `src-tauri/src/commands/seed.rs`
- **Commit:** `9280255` (included in Task 1 commit)

**3. [Rule 3 - Blocking] Disk space exhaustion prevented file writes**
- **Found during:** Task 1 state.rs rewrite
- **Issue:** Filesystem full (3.3GB in target dir)
- **Fix:** Removed `src-tauri/target` build artifacts to free space
- **Files modified:** n/a (build artifact cleanup)

## Success Criteria Verification

- [x] `cargo check` in src-tauri exits 0
- [x] 5 Tauri commands in seed.rs: generate_seed, rename_seed, delete_seed, copy_seed, list_seeds
- [x] 3 Tauri commands in queue.rs: get_queue, remove_from_queue, clear_queue
- [x] Seed generation uses weighted random MathOverlay ~30% (D-02, manual cumulative-probability)
- [x] Seed generation produces 3-7 operations (D-03)
- [x] Seed generation auto-alias uses timestamp format (D-04)
- [x] SEED-04 constraints enforced: opacity <= 0.15, pixel shift [-3,3], frame interval >= 15
- [x] All seed mutations persist to seeds.json and emit "seeds-updated"
- [x] All queue mutations persist to queue.json and emit "queue-updated"
- [x] get_queue checks path validity and marks invalid files (D-06)
- [x] remove_from_queue validates index bounds
- [x] No unwrap() calls in any command function; all errors propagated with Result<T, String>

## Known Stubs

None. All 8 commands are fully implemented with real persistence and event emission.

## Threat Flags

None. All threat surface introduced (8 new IPC commands, 2 new store files) is covered by the plan's threat model (T-02-10 through T-02-16).

## Self-Check: PASSED

- SUMMARY.md exists at `.planning/phases/02-rust-backend/02-03-SUMMARY.md`
- Commit `9280255` (Task 1: seed commands) found in git log
- Commit `f5b0b56` (Task 2: queue commands) found in git log
- All created/modified files verified present on disk
