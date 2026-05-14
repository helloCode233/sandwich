---
phase: 04-integration-polish
plan: 08
subsystem: batch-progress
tags: [bug-fix, gap-closure, rust, vue]
depends_on: []
requires: []
provides: [initial-batch-progress-emission, accurate-queue-size-frontend]
affects: [batch-progress IPC, useBatch composable, BatchControls component]
requirements: [BATCH-02]
gaps_closed:
  - "Missing initial batch-progress event — overall progress shows '0/1' during first file instead of '0/n'"
tech-stack:
  added: []
  patterns: [event-emission-after-lock-drop, frontend-state-before-invoke]
key-files:
  created: []
  modified:
    - src-tauri/src/commands/batch.rs
    - src/composables/useBatch.ts
    - src/components/batch/BatchControls.vue
decisions:
  - "Initial batch-progress emission uses clone inside Mutex lock scope then emits outside, respecting Pitfall 3"
  - "Frontend startProcessing runs BEFORE Rust invoke so UI shows '0/n' immediately, with Rust emission as redundant confirmation"
metrics:
  duration_seconds: 0
  completed_date: "2026-05-14"
---

# Phase 4 Plan 8: Fix initial batch-progress event so progress bar shows correct total from start

Fixed the missing initial `batch-progress` event emission so the overall progress bar shows the correct file count ("0/n") from the start of batch processing, not "0/1" during the first file.

## What Was Done

### Task 1: Rust emits initial batch-progress event after state init

Added an initial `batch-progress` event emission in `commands/batch.rs` immediately after the batch state initialization block. The `BatchProgress` struct is cloned inside the Mutex lock scope and emitted outside (respects Pitfall 3: drop locks before FFmpeg spawn). Previously, the first emission only arrived after the first file completed at line 206, causing the frontend to show "0/1" until that point.

### Task 2: Frontend passes actual queue size to startProcessing

Two-file fix:
- `useBatch.ts`: Changed `startBatch` signature to accept `queueSize: number`, moved `store.startProcessing(queueSize)` to BEFORE `await invoke('start_batch', ...)` so the UI shows the correct progress total immediately.
- `BatchControls.vue`: Passes `queueStore.entryCount` to `startBatch`, matching the Rust `queue_snapshot.len()` (which clones the entire queue).

Removed the hardcoded `store.startProcessing(1)` workaround that was the previous frontend fix for the same issue.

## Changes

| File | Change |
|------|--------|
| `src-tauri/src/commands/batch.rs` | Extract `initial_progress` from lock scope, emit `batch-progress` event with correct total before processing |
| `src/composables/useBatch.ts` | Add `queueSize` param, call `startProcessing(queueSize)` before `invoke` |
| `src/components/batch/BatchControls.vue` | Pass `queueStore.entryCount` as third argument to `startBatch` |

## Verification

- `cargo check` compiles without errors
- Two `"batch-progress"` emissions: initial (line 141) and per-file (line 211)
- `startProcessing(queueSize)` before `await invoke` in useBatch.ts
- `queueStore.entryCount` passed from BatchControls.vue
- `startProcessing(1)` no longer exists anywhere in the codebase

## Commits

- `ee4bd7c`: fix(04-integration-polish): emit initial batch-progress event from Rust after state init
- `1cc578c`: fix(04-integration-polish): pass actual queue size to startProcessing instead of hardcoded 1

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None. Changes are internal to existing IPC channel (`batch-progress` event) and function signatures. No new trust boundaries, network endpoints, or file access patterns introduced.

## Self-Check

- [x] `src-tauri/src/commands/batch.rs` modified and committed
- [x] `src/composables/useBatch.ts` modified and committed
- [x] `src/components/batch/BatchControls.vue` modified and committed
- [x] Two commits on branch: `ee4bd7c` and `1cc578c`
