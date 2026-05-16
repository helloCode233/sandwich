---
phase: 06
plan: 05
subsystem: rust-backend
type: execute
wave: 2
tags: [thumbnail, batch-log, reorder-queue, migration, command-registration]
depends_on: ["06-01"]
requires: ["06-01 (data model extensions)"]
provides:
  - D-14 (reorder_queue Tauri command)
  - D-15 (thumbnail_base64 extraction during import)
  - D-16 (batch-log events with real per-file duration_ms)
  - D-19 (legacy seed auto-migration on startup)
  - D-20 (all new Tauri commands registered in lib.rs)
affects:
  - src-tauri/src/commands/import.rs
  - src-tauri/src/commands/batch.rs
  - src-tauri/src/commands/queue.rs
  - src-tauri/src/commands/export_seed.rs
  - src-tauri/src/commands/mod.rs
  - src-tauri/src/migrations/seed_v2.rs
  - src-tauri/src/migrations/mod.rs
  - src-tauri/src/lib.rs
  - src-tauri/src/ffmpeg/executor.rs
  - src-tauri/Cargo.toml
tech-stack:
  added:
    - base64 0.22 (JPEG encoding for thumbnail)
    - md5 0.8.0 (file hashing for batch integrity)
  patterns:
    - FfmpegCommand::new_with_path + take_stdout() for stdout capture
    - Instant::now().elapsed() for per-file duration timing
    - ProcessingLogEntry construction from FileSuccess/FileResult with zip
    - tauri-plugin-store marker key for idempotent migration
    - generate_handler! macro for command registration
key-files:
  created:
    - src-tauri/src/migrations/mod.rs
    - src-tauri/src/migrations/seed_v2.rs
    - src-tauri/src/commands/export_seed.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/commands/import.rs
    - src-tauri/src/commands/batch.rs
    - src-tauri/src/commands/queue.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/ffmpeg/executor.rs
decisions:
  - Used extract_thumbnail with FfmpegChild::take_stdout() (not wait_with_output which is unavailable in ffmpeg-sidecar 2.5.x)
  - Per-file timing uses Instant::now() captured before execute_single_file, elapsed calculated after result
  - ProcessingLogEntry emitted one per result via app.emit("batch-log", entry) loop
  - export_seed.rs created as stub (Rule 3) for compilation; full implementation from plan 06-04
  - md5 crate added to Cargo.toml (missing dependency from 06-01 models/batch.rs)
  - executor.rs seed_alias field added preemptively (missing from 06-01 PerFileProgress extension)
metrics:
  duration: ~8 minutes
  tasks: 2
  files: 10
  completed: "2026-05-16T12:41:51Z"
---

# Phase 06 Plan 05: Rust Backend Infrastructure Wire-Up

**One-liner:** Threads thumbnail extraction into video import, emits batch-log events with real per-file duration_ms, adds reorder_queue command for drag-and-drop persistence, and runs idempotent legacy seed migration on startup — all registered in Tauri invoke_handler.

## Task Summary

### Task 1: Thumbnail Extraction + Batch-Log Events

- **Added** `extract_thumbnail()` function using ffmpeg-sidecar `take_stdout()` to capture 120px-wide JPEG frame as base64
- **Modified** `import_video` to extract thumbnail before constructing `VideoEntry`; emits `thumbnail-extraction-warning` on failure without blocking import
- **Rewrote** `start_batch` from single-seed (`seed_id: String`) to multi-seed (`seed_ids: Vec<String>`) with:
  - MD5 pre-hash map for all input files via `tokio::task::spawn_blocking`
  - Post-processing MD5 comparison with `FileSuccess` output (modified flag)
  - GPU encode failure auto-retry with CPU fallback (D-05)
  - Per-file timing via `Instant::now()` + `elapsed_ms` for D-16 duration tracking
  - `ProcessingLogEntry` construction from `FileSuccess`/`FileResult` with zipped durations
  - `batch-log` event emission loop for all results (frontend accumulates into history panel)
- **Added** `base64 = "0.22"` and `md5 = "0.8.0"` crate dependencies to Cargo.toml
- **Fixed** pre-existing missing `seed_alias` field in `executor.rs` `PerFileProgress`

### Task 2: reorder_queue + Seed Migration + Command Registration

- **Added** `reorder_queue` Tauri command in `queue.rs` with server-side `order_index` assignment and `persist_queue` call (D-14)
- **Created** `migrations/seed_v2.rs` with `migrate_seeds()` function: idempotent via `migration_v2_applied` marker key, safe for empty seed lists (D-19)
- **Created** `export_seed.rs` stub with `export_seed` and `import_seed` Tauri commands (placeholder implementations for parallel-plan compatibility)
- **Updated** `commands/mod.rs` with `pub mod export_seed`
- **Updated** `lib.rs`:
  - Added `mod migrations;` declaration
  - Imported `reorder_queue`, `export_seed`, `import_seed`
  - Registered all 3 new commands in `invoke_handler!` macro (D-20)
  - Spawned `seed_v2::migrate_seeds` on startup with `seeds-migrated` / `seeds-migration-error` event emission

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Missing Dependency] Added md5 crate to Cargo.toml**
- **Found during:** Task 1 GREEN
- **Issue:** `models/batch.rs` `file_md5()` function uses `md5::Context` but Cargo.toml did not declare the `md5` dependency
- **Fix:** Added `md5 = "0.8.0"` to Cargo.toml `[dependencies]`
- **Files modified:** `src-tauri/Cargo.toml`
- **Commit:** `743cd04`

**2. [Rule 3 - Missing Module] Created export_seed.rs stub for lib.rs compilation**
- **Found during:** Task 2 GREEN
- **Issue:** `lib.rs` imports `commands::export_seed::{export_seed, import_seed}` but `export_seed.rs` does not exist (created by parallel plan 06-04)
- **Fix:** Created `src-tauri/src/commands/export_seed.rs` with stub `export_seed`/`import_seed` commands returning "Not yet implemented" error; also added `pub mod export_seed` to `commands/mod.rs`
- **Files modified:** `src-tauri/src/commands/export_seed.rs` (created), `src-tauri/src/commands/mod.rs`
- **Commit:** `b5af561`

**3. [Rule 3 - Pre-existing Error] Fixed missing seed_alias field in executor.rs**
- **Found during:** Task 1 GREEN
- **Issue:** `PerFileProgress` struct gained `seed_alias` field in plan 06-01 models but `executor.rs` construction did not set it (pre-existing compilation error not caused by 06-05)
- **Fix:** Added `seed_alias: seed.alias.clone()` to `PerFileProgress` initialization
- **Files modified:** `src-tauri/src/ffmpeg/executor.rs`
- **Commit:** `743cd04`

**4. [Rule 1 - API Mismatch] Fixed ffmpeg-sidecar API usage for thumbnail extraction**
- **Found during:** Task 1 GREEN
- **Issue:** Plan's research doc used `wait_with_output()` which does not exist on `FfmpegChild` in ffmpeg-sidecar 2.5.x
- **Fix:** Used `FfmpegChild::take_stdout()` + `Read::read_to_end()` for stdout capture, then `wait()` for process completion (avoids deadlock by reading stdout before waiting)
- **Files modified:** `src-tauri/src/commands/import.rs`
- **Commit:** `743cd04`

## Known Stubs

| File | Line | Reason |
|------|------|--------|
| `src-tauri/src/commands/export_seed.rs` | 16 | `export_seed` returns "Not yet implemented" error — full implementation provided by plan 06-04 in same wave |
| `src-tauri/src/commands/export_seed.rs` | 22 | `import_seed` returns "Not yet implemented" error — full implementation provided by plan 06-04 in same wave |

Both stubs are intentional and will be replaced when plan 06-04's worktree is merged by the orchestrator.

## Commits

| # | Commit | Type | Message |
|---|--------|------|---------|
| 1 | `8710b8e` | test | add failing tests for thumbnail extraction and batch-log events (RED) |
| 2 | `743cd04` | feat | implement thumbnail extraction, batch-log events, per-file duration timing (GREEN) |
| 3 | `bc4f49f` | test | add failing tests for reorder_queue, migrate_seeds, and lib.rs (RED) |
| 4 | `b5af561` | feat | implement reorder_queue, seed migration, export_seed stub, lib.rs registration (GREEN) |

## Verification

- `cargo check -p sandwich` exits 0 (warnings only, no errors)
- All 18 acceptance criteria met (grep counts verified)
- Thumbnail extraction uses `scale=120:-1` with JPEG `image2pipe` output to stdout
- Batch-log events emit `ProcessingLogEntry` with real `duration_ms` from `Instant::now().elapsed()`
- `reorder_queue` assigns `order_index = i as u32` server-side
- `migrate_seeds` checks `migration_v2_applied` marker for idempotency
- All new commands registered in `invoke_handler!` macro

## Self-Check: PASSED

All 4 commits verified present in git log. All 10 created/modified files confirmed in repository.
