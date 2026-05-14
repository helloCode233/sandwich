---
phase: 04-integration-polish
plan: "01"
subsystem: backend
tags: [tauri, rust, ffmpeg, event-streaming, progress, batch-processing]

# Dependency graph
requires: []
provides:
  - PerFileProgress struct with 6 rich progress fields (file, percent, currentFrame, totalFrames, fps, remainingSeconds)
  - batch-file-progress event from Rust executor carrying full frame-level progress data
  - Resolved event naming collision between executor.rs and batch.rs (formerly both used "batch-progress")
affects: [03-vue-frontend, batch-progress-ui, useBatchStore]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Dedicated event name for per-file progress (batch-file-progress) distinct from aggregate batch progress (batch-progress)
    - Rust event-only structs use Serialize (not Deserialize) for one-way IPC emission
    - Negative remaining_seconds clamped to 0.0 via max(0.0) for edge safety

key-files:
  created: []
  modified:
    - src-tauri/src/models/batch.rs
    - src-tauri/src/ffmpeg/executor.rs

key-decisions:
  - "Use separate batch-file-progress event (carrying PerFileProgress) instead of overloading batch-progress with union payload"
  - "Compute remaining_seconds from ffmpeg-sidecar speed field: (total_duration - seconds) / speed"
  - "Clamp remaining_seconds to non-negative via max(0.0) for edge safety when speed is 0"
  - "PerFileProgress uses Serialize-only (no Deserialize) since it is emitted from Rust to frontend only"

patterns-established:
  - "Event naming convention: aggregate progress uses batch-progress, per-file progress uses batch-file-progress"
  - "Progress structs scoped to models/batch.rs, consumed by executor.rs via import"

requirements-completed: [BATCH-02]

# Metrics
duration: 5min
completed: 2026-05-14
---

# Phase 4 Plan 1: Fix Event Naming Collision and Enrich Per-File Progress

**Introduced batch-file-progress event carrying PerFileProgress struct with frame, FPS, and ETA data to resolve executor.rs/batch.rs event collision**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-14T01:25:23Z
- **Completed:** 2026-05-14T01:30:19Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `PerFileProgress` struct to `models/batch.rs` with 6 fields: file, percent, currentFrame, totalFrames, fps, remainingSeconds
- Removed `ExecutorProgress` struct from executor.rs (replaced by PerFileProgress)
- Replaced executor's `"batch-progress"` event emission with `"batch-file-progress"` carrying full `PerFileProgress` data
- Extract frame, FPS, and speed from ffmpeg-sidecar's `FfmpegProgress` (previously only `time` was used)
- Compute ETA as `remaining_seconds = (total_duration - current_time) / speed`, clamped to non-negative
- Resolved event naming collision: executor.rs no longer emits under `"batch-progress"` (batch.rs owns that event for aggregate progress)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add PerFileProgress struct** - `5d5c89b` (feat)
2. **Task 2: Modify executor to emit batch-file-progress** - `8f60c0f` (feat)

## Files Created/Modified
- `src-tauri/src/models/batch.rs` - Added PerFileProgress struct (6 fields, camelCase serde rename, Serialize-only)
- `src-tauri/src/ffmpeg/executor.rs` - Removed ExecutorProgress, removed "starting" emission, replaced FfmpegEvent::Progress match arm with batch-file-progress emission carrying PerFileProgress

## Decisions Made
- Followed plan's approach of separate event name (`batch-file-progress`) with dedicated `PerFileProgress` struct rather than overloading the existing `batch-progress` event
- ETA calculation uses ffmpeg-sidecar's `speed` field: `(total_duration - seconds) / speed` with a 0.01 guard against division by zero
- remaining_seconds clamped to non-negative via `.max(0.0)` per the plan's Pitfall 3 guidance

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed cargo fmt import ordering**
- **Found during:** Task 2 (commit hook)
- **Issue:** `use crate::models::batch::PerFileProgress` was placed after `use crate::models::seed::Seed` but rustfmt required alphabetical ordering (batch < seed < video)
- **Fix:** Moved PerFileProgress import before Seed import in the crate::models block
- **Files modified:** `src-tauri/src/ffmpeg/executor.rs`
- **Verification:** `cargo fmt --check` passes, `cargo check` exits 0
- **Committed in:** `8f60c0f` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minimal -- purely formatting, no behavioral change.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Rust backend now emits rich per-file progress data via `batch-file-progress` event
- Frontend (Phase 3 / Phase 4-02) can now `listen("batch-file-progress", ...)` to receive frame, FPS, and ETA data
- Event naming collision resolved -- executor.rs no longer conflicts with batch.rs on `batch-progress`

---
*Phase: 04-integration-polish*
*Completed: 2026-05-14*
