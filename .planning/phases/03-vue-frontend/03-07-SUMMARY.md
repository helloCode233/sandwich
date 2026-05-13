---
phase: 03-vue-frontend
plan: 07
subsystem: batch-controls
tags: [vue, batch, controls, progress, ui]
requires: ["03-03 (useBatch composable, batch/seed/queue stores)"]
provides: ["BatchControls.vue", "BatchBanner.vue"]
affects: ["sticky bottom batch action hub", "batch progress banner"]
tech-stack: { added: [], patterns: ["Naive UI NSelect/NProgress/NButton/NSpace", "tauri-plugin-store Store.load()", "tauri-plugin-dialog open()", "vue-i18n useI18n()"] }
key-files: { created: ["src/components/batch/BatchControls.vue", "src/components/batch/BatchBanner.vue"], modified: [] }
decisions:
  - "Use Store.load() static method (tauri-plugin-store v2.4) instead of new Store() (private constructor)"
  - "Load store reference once on mount and reuse; persist concurrency via store.set('concurrency', n)"
  - "Output directory persisted to plugin-store key 'output_dir' for Rust persist_output_dir() to read"
  - "NProgress color transitions from accent blue (#2080f0) to success green (#18a058) at 100%"
duration: ""
---

# Phase 3 Plan 7: Batch Controls and Progress Banner Summary

Plan 03-07 implemented two batch-processing UI components: BatchControls.vue (the sticky bottom action hub with seed selector, concurrency, output directory picker, and start/cancel toggle) and BatchBanner.vue (the top progress banner with NProgress bar visible during batch execution).

## Tasks Completed

| # | Task | Commit | Files |
|---|------|--------|-------|
| 1 | Create BatchControls.vue (seed selector, concurrency, output dir, start/cancel) | `f4abab3` | `src/components/batch/BatchControls.vue` |
| 2 | Create BatchBanner.vue (progress banner with NProgress) | `dff0984` | `src/components/batch/BatchBanner.vue` |

## What Was Built

**BatchControls.vue** (216 lines) — The action hub component implementing D-11 (concurrency 1-4, persisted to plugin-store) and D-12 (output directory display + native directory picker). Features:

- NSelect for seed selection from `useSeedStore().seeds` (value=id, label=alias), filterable and clearable
- NSelect for concurrency with options [1, 2, 3, 4], default 1, writes to plugin-store key `concurrency` on change
- Output directory display (truncated NText) + "Change" NButton using `@tauri-apps/plugin-dialog` `open({ directory: true })`
- Output directory persisted to plugin-store key `output_dir` matching Rust's `persist_output_dir()`
- Start button (type="primary", block, Play icon) when idle — disabled when no seed selected or `queueStore.validCount === 0`
- Cancel button (type="error", block, Square icon) when `batchStore.isProcessing`
- All controls disabled during processing (seed selector, concurrency, output dir) except cancel
- Preferences loaded from plugin-store on mount; falls back to i18n `batch.defaultOutputDir`

**BatchBanner.vue** (42 lines) — The progress display banner implementing D-13 structure. Features:

- Horizontal banner strip with Processing label, NProgress bar, and completed/total counter
- NProgress with `indicator-placement="inside"` and `:height="24"`
- Color: `#2080f0` during processing, `#18a058` (success green) at 100%
- Counter text with `tabular-nums` to prevent layout shift on digit changes
- Dark strip background (`#1a1a24`) with bottom border (`#2a2a36`)
- Pure display component — reads reactive state from `useBatchStore()`, no mutations

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] tauri-plugin-store v2.4 requires Store.load() static API, not new Store()**

- **Found during:** Task 1 implementation
- **Issue:** The plan used `new Store('sandwich-config.json')` pattern, but `@tauri-apps/plugin-store` v2.4.3 has a private `Store` constructor. The correct API is `await Store.load('sandwich-config.json')`.
- **Fix:** Replaced all `new Store(...)` calls with `await Store.load(...)`. Refactored to load the store once in `loadPreferences()` and reuse the reference. Added null-guards for store operations before the store is loaded.
- **Files modified:** `src/components/batch/BatchControls.vue`

**2. [Rule 3 - Blocking] ESLint no-undef: console not recognized as global**

- **Found during:** Task 1 commit (lint-staged hook)
- **Issue:** ESLint config lacks `env: browser`, so `console` (used via `console.warn`/`console.error` for error logging) triggered `no-undef` errors.
- **Fix:** Added `/* global console */` directive at the top of the `<script setup>` block.
- **Files modified:** `src/components/batch/BatchControls.vue`

## Known Stubs

None. Both components are fully wired:
- BatchControls reads from seed/queue/batch stores and calls `useBatch().startBatch()`/`cancelBatch()` with real arguments
- BatchBanner reads reactive `batchStore.progress` and `batchStore.overallPercent` — its display updates via the store's reactivity system, which is populated by `useBatch` composable event listeners

## Threat Flags

None. No new threat surface introduced beyond what is captured in the plan's threat model (T-03-16 concurrency tampering mitigated by NSelect value constraints + Rust server-side clamp; T-03-17 rapid start/cancel mitigated by Rust batch state checks; T-03-18 output path accepted for local desktop app).

## Verification Results

All plan-specified verification checks passed:

```
BatchControls.vue: startBatch(2), cancelBatch(2), NSelect(4), concurrency(11), outputDir(8), Store import(1), isProcessing(5), startDisabled(2), onConcurrencyChange(2), Store.load(1)
BatchBanner.vue: NProgress(2), overallPercent(2), progress(1), batch.processing(1), batch.progress(1), indicator-placement(1), batch-banner(2)
TypeScript: vue-tsc --noEmit passed with zero errors
ESLint: passed after /* global console */ fix
```

## Self-Check: PASSED

- [x] `src/components/batch/BatchControls.vue` exists
- [x] `src/components/batch/BatchBanner.vue` exists
- [x] Commit `f4abab3` exists in git log
- [x] Commit `dff0984` exists in git log
- [x] No file deletions in commit range
- [x] TypeScript compilation passes
