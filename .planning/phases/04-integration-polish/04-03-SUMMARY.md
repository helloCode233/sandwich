---
phase: 04-integration-polish
plan: "03"
subsystem: batch-state
tags:
  - store
  - composable
  - event-listener
  - progress-tracking
requires:
  - 04-01 (batch-file-progress event, PerFileProgress Rust struct)
  - 04-02 (PerFileProgress TypeScript interface, batch i18n keys)
provides:
  - 04-04 (per-file progress bars, cancelling UI state, error toasts)
affects:
  - src/stores/batch.ts
  - src/composables/useBatch.ts
tech-stack:
  added: []
  patterns:
    - Map reactivity trigger via new Map(oldMap) pattern
    - Tauri event listener with typed generic listen<PerFileProgress>
    - Naive UI message.error toast driven by i18n key with interpolation
key-files:
  created: []
  modified:
    - src/stores/batch.ts (added perFileProgress map, cancelling ref, currentFileProgress computed, setPerFileProgress/setCancelling mutations)
    - src/composables/useBatch.ts (added batch-file-progress/cancelling listeners, file-error toast, 6-listener cleanup)
decisions:
  - "Map reactivity trigger: perFileProgress.value = new Map(perFileProgress.value) after Map.set() because Vue 3 reactivity does not track Map mutations"
  - "Cancelling state resets on both stopProcessing (completion) and resetBatch (manual reset) to prevent stale cancelling UI"
  - "fileProgressUnlisten and cancellingUnlisten added between fileErrorUnlisten and completeUnlisten in unsubscribe() for ordered cleanup"
metrics:
  duration: 2m
  completed_date: 2026-05-14
---

# Phase 04 Plan 03: Batch Store and Composable Extension for Per-File Progress

Extends the Pinia batch store with per-file progress tracking (Map<string, PerFileProgress>) and cancelling transitional state. Extends the useBatch composable with batch-file-progress and batch-cancelling event listeners plus user-facing error toasts on batch-file-error.

## Summary

2 tasks completed. The `useBatchStore` now tracks per-file progress via a reactive `Map<string, PerFileProgress>` keyed by filename, exposes a `cancelling` transitional state ref, and provides `setPerFileProgress`/`setCancelling` mutations. The `useBatch` composable now subscribes to 6 batch events (batch-progress, batch-file-error with toast, batch-complete, batch-cancelled, batch-file-progress, batch-cancelling), routing data to store mutations and showing localized error messages via Naive UI `message.error()`.

### Key Changes

- **stores/batch.ts**: Added `perFileProgress` (Map), `cancelling` (ref), `currentFileProgress` (computed), `setPerFileProgress` (with reactivity trigger), `setCancelling`. Updated `stopProcessing` and `resetBatch` to clear new state.
- **composables/useBatch.ts**: Added `useMessage` and `useI18n` for toast localization. Added `batch-file-progress` and `batch-cancelling` listeners. Replaced `console.warn` with `message.error(t('batch.fileFailed', ...))` on file errors. Added 2 new unlisten vars with full cleanup in `unsubscribe()`.

## Deviations from Plan

None -- plan executed exactly as written.

## Verification

- `npx vue-tsc -b --noEmit`: No errors in `stores/batch.ts` or `composables/useBatch.ts`
- 6 event listeners registered: batch-progress, batch-file-error, batch-complete, batch-cancelled, batch-file-progress, batch-cancelling
- 6 unlisten calls in `unsubscribe()`: all 6 listeners properly cleaned up
- Map reactivity trigger: `new Map(perFileProgress.value)` reassignment ensures Vue reactivity on `Map.set()`

## Threat Flags

None. No new threat surface beyond what the plan's threat model covers. All event listeners are properly cleaned up (T-04-05 mitigated). Error toast displays backend-controlled strings only (T-04-06 accepted risk).

## Self-Check: PASSED

- `src/stores/batch.ts`: FOUND
- `src/composables/useBatch.ts`: FOUND
- Commit `30a19ad`: FOUND
- Commit `8d17f84`: FOUND

