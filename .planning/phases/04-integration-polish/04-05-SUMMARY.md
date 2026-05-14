---
phase: 04-integration-polish
plan: "05"
subsystem: batch-ui
tags:
  - progress-bar
  - completion-summary
  - conditional-rendering
  - queue-disable
requires:
  - 04-03 (batch store with perFileProgress, cancelling, currentFileProgress)
  - 04-04 (BatchBanner multi-state rendering, i18n batch.summary.* keys)
provides:
  - BATCH-02 (per-file frame-level progress with ETA in QueueList)
  - BATCH-05 (completion summary with succeeded/failed file breakdown)
affects:
  - src/components/queue/QueueList.vue
  - src/components/batch/BatchSummary.vue
  - src/components/MainLayout.vue
tech-stack:
  added: []
  patterns:
    - NProgress inside queue items with conditional color (accent #2080f0 / success #18a058 at 100%)
    - Inline completion panel (not modal) with NScrollbar-bounded sections at 200px/60vh
    - Cancellation detection via (succeeded + failed) < progress.total
    - BatchBanner in all non-idle states for consistent top-of-panel visual anchor
key-files:
  created:
    - src/components/batch/BatchSummary.vue (inline completion summary with succeeded/failed lists)
  modified:
    - src/components/queue/QueueList.vue (per-file NProgress bars, disabled controls during processing)
    - src/components/MainLayout.vue (conditional BatchSummary rendering, BatchBanner for all non-idle states)
decisions:
  - "NProgress per-file color transitions from #2080f0 (active) to #18a058 (100%) to match overall batch bar behavior"
  - "BatchSummary is inline panel beneath BatchBanner, not a modal overlay — keeps queue area usable post-completion"
  - "BatchBanner v-if expanded to isProcessing || cancelling || isComplete so the banner serves as persistent top-of-panel status anchor across all non-idle states"
  - "Cancellation detection in BatchSummary derived from (succeeded + failed) < batchStore.progress.total rather than a dedicated wasCancelled flag"
metrics:
  duration: 5m
  completed_date: 2026-05-14
---

# Phase 04 Plan 05: Per-File Progress Bars, BatchSummary, and MainLayout State Rendering

Wires Phase 3's placeholder UI structure to the real store state added in 04-03: per-file NProgress bars in QueueList with frame counter and ETA, a new BatchSummary completion summary component with succeeded/failed file lists, and updated MainLayout conditional rendering across all batch states.

## Summary

3 tasks completed. QueueList now shows per-file NProgress bars (with frame count and ETA) for the currently processing file and disables all mutation controls during batch processing. BatchSummary is a new inline completion panel showing succeeded/failed file lists with output paths and error messages, triggered when `isComplete && lastResult`. MainLayout now conditionally renders BatchBanner in all non-idle states (processing, cancelling, complete) and BatchSummary when the batch is complete with results.

### Key Changes

- **QueueList.vue**: Imported NProgress and useBatchStore. Added isCurrentFile/fileProgressFor helpers. Per-file NProgress bar renders below metadata line in each queue item when `batchStore.progress.currentFile === entry.filename`. Frame counter and ETA displayed below bar. Clear All button hidden during processing. Add Video button hidden during processing (empty state). Per-item Remove buttons disabled during processing.
- **BatchSummary.vue** (new): Inline completion summary panel. Shows CheckCircle (green, all success) or AlertCircle (warning, partial/cancelled). Displays succeeded section with per-file output paths and failed section with per-file error messages. Each section max 200px height with NScrollbar. Overall panel max 60vh. "Clear Results" button calls `batchStore.resetBatch()`. Cancellation detected by comparing completed file count against `progress.total`.
- **MainLayout.vue**: Imported BatchSummary. BatchBanner now renders for `isProcessing || cancelling || isComplete` (all non-idle states). BatchSummary renders for `isComplete && lastResult`. Produces four visual states: idle (nothing), processing (banner only), cancelling (banner only, frozen), complete (banner green 100% + summary file breakdown).

## Deviations from Plan

None -- plan executed exactly as written.

## Verification

- `npx vue-tsc -b --noEmit`: No errors in QueueList.vue, BatchSummary.vue, or MainLayout.vue
- All acceptance grep criteria met for all three tasks
- Per-file progress: NProgress with indicator-placement="inside", height 18, dynamic color
- Disabled controls: 4 locations guarded by `batchStore.isProcessing` in QueueList
- BatchSummary: 9 i18n summary keys used, 2 scroll bounds (200px + 60vh), resetBatch wired
- MainLayout: BatchBanner visibility expanded to 3 states, BatchSummary conditional with lastResult

## Threat Flags

None — all threat surfaces covered by plan threat model (T-04-10 overflow, T-04-11 disabled controls, T-04-12 output paths).
