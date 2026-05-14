---
phase: 04-integration-polish
plan: 04
type: execute
subsystem: batch-ui
tags: [batch-banner, cancel-confirmation, import-zone]
requires:
  - 04-03 (batch store + composable extensions)
provides:
  - BATCH-05 (batch banner multi-state + cancel confirmation + import-zone guard)
affects:
  - src/components/batch/BatchBanner.vue
  - src/components/batch/BatchControls.vue
  - src/components/queue/ImportZone.vue
tech-stack:
  added: []
  patterns:
    - "Naive UI dialog.warning() for destructive action confirmation"
    - "Vue computed for derived UI state (bannerState, labelText, barColor, barPercent)"
    - "v-if bindings on Pinia store refs for component visibility during processing"
key-files:
  created: []
  modified:
    - src/components/batch/BatchBanner.vue
    - src/components/batch/BatchControls.vue
    - src/components/queue/ImportZone.vue
decisions:
  - "BatchBanner uses computed bannerState to derive processing/cancelling/complete from batchStore.cancelling and batchStore.isComplete"
  - "Cancel confirmation uses Naive UI dialog.warning() with onPositiveClick callback (matches existing clear-all pattern in QueueList)"
  - "ImportZone hidden via v-if on batchStore.isProcessing (prevents queue mutations during processing per RESEARCH Pitfall 5)"
metrics:
  duration: "3 minutes"
  completed_date: "2026-05-14"
---

# Phase 4 Plan 4: Batch UI Polish Summary

**One-liner:** Added multi-state BatchBanner (processing/cancelling/complete), dialog.warning() cancel confirmation in BatchControls, and ImportZone hide during batch processing.

## Tasks Executed

| # | Task | Commit | Status |
|---|------|--------|--------|
| 1 | Modify BatchBanner.vue for processing, cancelling, and complete states | d163321 | Done |
| 2 | Modify BatchControls.vue to add cancel confirmation dialog and no-output-dir validation | 28b645a | Done |
| 3 | Disable ImportZone during batch processing | ca65bef | Done |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed redundant `/* global console */` ESLint declaration in BatchControls.vue**
- **Found during:** Task 2 commit
- **Issue:** Pre-existing `/* global console */` comment on line 2 caused ESLint `no-redeclare` error, blocking commit via lint-staged hook
- **Fix:** Removed the redundant declaration (console is already a built-in global in the ESLint config)
- **Files modified:** src/components/batch/BatchControls.vue
- **Commit:** 28b645a

## Verification Results

All verification checks passed:

- **BatchBanner.vue:** All 9 acceptance criteria met (bannerState, cancelling, complete, i18n keys, colors). vue-tsc clean.
- **BatchControls.vue:** All 10 acceptance criteria met (useDialog, dialog.warning, i18n keys, cancelling state, button states). vue-tsc clean.
- **ImportZone.vue:** All 3 acceptance criteria met (useBatchStore import, batchStore usage, v-if binding). vue-tsc clean.
- **Cross-file:** No type errors across all 3 files.

## Threat Mitigations Implemented

| Threat | Status | Implementation |
|--------|--------|----------------|
| T-04-08 (Repudiation - Cancel without confirmation) | Mitigated | dialog.warning() provides explicit confirmation step before calling cancelBatch() |
| T-04-09 (DoS - Queue mutation during processing) | Mitigated | ImportZone hidden via v-if="!batchStore.isProcessing", preventing drag-drop and file dialog during batch |

## Self-Check: PASSED

- [x] BatchBanner.vue exists with all 3 states
- [x] BatchControls.vue has dialog.warning() and no-output-dir validation
- [x] ImportZone.vue hidden during processing
- [x] All 3 commits verified in git log
- [x] vue-tsc reports no errors in any modified file
