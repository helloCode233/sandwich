---
phase: 04-integration-polish
verified: 2026-05-14T22:00:00Z
status: gaps_found
score: 18/18 must-haves verified
overrides_applied: 0
gaps:
  - truth: "During batch processing, per-file progress percentage accurately reflects encoding progress for all video durations"
    status: failed
    reason: "parse_time_to_seconds() in executor.rs:153-166 only handles HH:MM:SS.mm format. FFmpeg emits 'MM:SS.mm' format for videos under 1 hour. When time_str contains a colon but only has 2 parts (e.g. '01:30.50'), parts.len() == 3 is false, so the function falls through to time_str.parse() which returns 0.0. This causes per-file progress percent to stay at 0% for the entire duration of short videos, breaking the 'percentage complete' requirement of ROADMAP SC-1."
    artifacts:
      - path: "src-tauri/src/ffmpeg/executor.rs"
        issue: "parse_time_to_seconds (lines 153-166) does not handle 2-part colon-separated time format (MM:SS.mm)"
    missing:
      - "Add a match arm for parts.len() == 2 to handle MM:SS.mm format in parse_time_to_seconds"

  - truth: "After processing finishes, the banner correctly distinguishes between batch completion and batch cancellation"
    status: failed
    reason: "BatchBanner.vue:19-20 uses lastResult?.failed.length as a proxy for cancellation. A batch that completes with some files failed (e.g. 3 succeeded, 2 failed) will display 'Batch Cancelled' even though it fully completed. The BatchSummary component (lines 13-16) correctly distinguishes completion vs. cancellation using (succeeded + failed) < progress.total, but BatchBanner uses a different, incorrect heuristic. Breaks ROADMAP SC-2's requirement for accurate completion display."
    artifacts:
      - path: "src/components/batch/BatchBanner.vue"
        issue: "labelText for complete state uses failed.length as cancellation proxy at line 19"
    missing:
      - "Replace the failed.length check in BatchBanner.vue with the same cancellation detection logic used by BatchSummary (or share a wasCancelled computed)"

  - truth: "The overall batch progress bar shows an accurate total file count throughout processing"
    status: failed
    reason: "Key link 'Rust start_batch init -> Frontend batch-progress listener' is NOT_WIRED: commands/batch.rs:124-136 initializes BatchProgress with the correct total but never emits a batch-progress event. The frontend workaround in useBatch.ts:59 calls startProcessing(1) with a hardcoded total of 1. Result: the progress bar displays '0/1' during the first file instead of '0/n'. The correct total only arrives with the first batch-progress event after the first file completes (commands/batch.rs:206-207)."
    artifacts:
      - path: "src-tauri/src/commands/batch.rs"
        issue: "No app.emit('batch-progress', ...) after batch state initialization at line 136"
      - path: "src/composables/useBatch.ts"
        issue: "startProcessing(1) uses hardcoded total=1; BatchControls.vue does not pass queue entry count"
    missing:
      - "Emit an initial batch-progress event in commands/batch.rs right after initializing batch state (after line 136)"
      - "Pass the actual queue entry count through startBatch() -> startProcessing() in useBatch.ts and BatchControls.vue"
deferred: []
human_verification:
  - test: "Test the cancel flow with an actual FFmpeg process running"
    expected: "Clicking Cancel shows the dialog. On confirm, the banner shows 'Cancelling...', the cancel button becomes a disabled loading state, FFmpeg terminates, and the app returns to idle with appropriate batch-cancelled summary."
    why_human: "Requires a running FFmpeg process that can be terminated -- infrastructure setup required."

  - test: "Test per-file progress bars during real FFmpeg processing"
    expected: "During batch processing, the currently-processing file in the queue shows an NProgress bar with percentage, current frame / total frames, and estimated remaining minutes. The bar progresses smoothly. The overall banner shows completed/total."
    why_human: "Requires actual FFmpeg processing with real video files to observe real-time progress events and verify UI rendering timing."

  - test: "Test the E2E workflow: generate seed -> drag videos -> select seed -> process -> review summary"
    expected: "All steps work without error. Progress visible throughout. Summary shows correct succeeded/failed counts with output paths. Clear Results returns to idle."
    why_human: "Full integration test spanning seed generation, video import, batch processing, and result display -- requires application to be runnable with all systems operational."

  - test: "Verify completion summary with mixed succeeded/failed results"
    expected: "After a batch where some files succeed and some fail, the summary shows correct counts, per-file output paths for succeeded files, per-file error messages for failed files. Banner title correctly indicates completion (not cancellation)."
    why_human: "Requires input files that will produce both success and failure outcomes during processing."
---

# Phase 4: Integration & Polish Verification Report

**Phase Goal:** Users experience live per-video progress feedback during processing, a clear batch completion summary, responsive cancellation from the UI, and a reliable end-to-end workflow with no broken states.
**Verified:** 2026-05-14T22:00:00Z
**Status:** gaps_found
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths (from PLAN Must-Haves)

All 18 must-have truths from the 5 execution plans were verified against the actual codebase. Every truth passes.

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Per-file progress events carry frame, FPS, and ETA data | ✓ VERIFIED | executor.rs:100-124 constructs PerFileProgress with currentFrame (progress.frame), fps (progress.fps), remainingSeconds from speed field |
| 2 | Executor no longer emits under "batch-progress" event name (collision resolved) | ✓ VERIFIED | executor.rs emits only "batch-file-progress" (line 115) and "batch-log" (line 129); "batch-progress" exists only in commands/batch.rs:206 |
| 3 | PerFileProgress TypeScript interface exists with all 6 fields matching Rust struct | ✓ VERIFIED | types/batch.ts:24-37 has file, percent, currentFrame, totalFrames, fps, remainingSeconds -- camelCase matching serde rename_all |
| 4 | All 18 new i18n keys exist in both en.json and zh-CN.json under batch.* and batch.summary.* namespaces | ✓ VERIFIED | All 18 keys confirmed in both locale files; cancelConfirm replaced by cancelConfirmTitle + cancelConfirmBody; both files valid JSON |
| 5 | The deprecated batch.cancelConfirm key is replaced by batch.cancelConfirmTitle and batch.cancelConfirmBody | ✓ VERIFIED | cancelConfirm removed from both locales; cancelConfirmTitle + cancelConfirmBody present |
| 6 | useBatchStore tracks per-file progress via Map<string, PerFileProgress> keyed by filename | ✓ VERIFIED | stores/batch.ts:15 -- perFileProgress ref<Map>, setPerFileProgress at line 61-65 with reactivity trigger (new Map) |
| 7 | useBatchStore exposes cancelling state that transitions to true when batch-cancelling event arrives | ✓ VERIFIED | stores/batch.ts:16 -- cancelling ref, setCancelling at line 68-70; useBatch.ts:47-49 calls store.setCancelling(true) |
| 8 | useBatch composable listens for batch-file-progress and batch-cancelling events | ✓ VERIFIED | useBatch.ts:43-49 -- two listeners with typed generics, both cleaned up in unsubscribe() |
| 9 | useBatch composable shows user-facing toast on batch-file-error events | ✓ VERIFIED | useBatch.ts:26-32 -- message.error(t('batch.fileFailed', ...)) with filename and error interpolation |
| 10 | BatchBanner shows "Cancelling..." label with frozen progress bar when cancelling is true | ✓ VERIFIED | BatchBanner.vue:11,17 -- bannerState returns 'cancelling' when batchStore.cancelling; labelText returns t('batch.cancelling') |
| 11 | BatchBanner shows processing file label when currentFile is set | ✓ VERIFIED | BatchBanner.vue:23-24 -- t('batch.processingFile', { filename }) when currentFile !== null |
| 12 | BatchBanner is visible during processing, cancelling, AND complete states | ✓ VERIFIED | MainLayout.vue:90 -- v-if="isProcessing || cancelling || isComplete" |
| 13 | BatchControls cancel button triggers NModal dialog.warning() confirmation before calling cancelBatch() | ✓ VERIFIED | BatchControls.vue:119-131 -- dialog.warning() with cancelConfirmTitle/Body, onPositiveClick calls cancelBatch() |
| 14 | ImportZone is hidden during batch processing | ✓ VERIFIED | ImportZone.vue:99 -- v-if="!batchStore.isProcessing" on root div |
| 15 | QueueList shows per-file NProgress bar with frame counter and ETA for the currently processing file | ✓ VERIFIED | QueueList.vue:196-224 -- NProgress with percentage, fileProgress i18n (frame counter), fileEta i18n (estimated time) |
| 16 | QueueList disables remove/clear/add buttons during batch processing | ✓ VERIFIED | QueueList.vue:130,153,228,232 -- all mutation controls gated on !batchStore.isProcessing |
| 17 | BatchSummary shows succeeded/failed file lists with per-file output paths after batch completes | ✓ VERIFIED | BatchSummary.vue:52-104 -- succeeded section with outputPath i18n, failed section with fileError i18n, NScrollbar at 200px |
| 18 | MainLayout conditionally renders BatchBanner during processing/cancelling/complete AND BatchSummary when isComplete | ✓ VERIFIED | MainLayout.vue:90-93 -- BatchBanner for all non-idle states, BatchSummary when isComplete && lastResult |

**Score:** 18/18 must-have truths verified (all PLAN must_haves met).

### ROADMAP Success Criteria Gaps

While all PLAN must_haves are verified, the following ROADMAP success criteria gaps were identified:

| SC | Criteria | Status | Gap |
|----|----------|--------|-----|
| SC-1 | Real-time per-video progress bars with percentage, frame, ETA | ⚠️ PARTIAL | parse_time_to_seconds bug breaks percentage for short videos (<1 hour) -- progress stays at 0% |
| SC-1 | Overall progress shows accurate file count | ⚠️ PARTIAL | Initial total is hardcoded to 1 during first file (no initial batch-progress emission from Rust) |
| SC-2 | Completion summary accurately reflects batch outcome | ⚠️ PARTIAL | BatchBanner banner title shows "Cancelled" for completed batches with failed files |
| SC-3 | Cancel from UI, graceful FFmpeg termination, clean reset | ✓ MET | Fully wired: dialog confirmation -> cancelling state -> cancel flag -> FFmpeg kill -> state reset |
| SC-4 | Reliable E2E workflow without broken states | ✗ FAILED | Broken by the 3 gaps above: wrong total, broken short-video progress, misleading banner title |

### Deferred Items

None. All identified gaps are within Phase 4 scope.

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/models/batch.rs` | PerFileProgress struct (6 fields) | ✓ VERIFIED | Exists at line 57, 6 fields with Serialize, camelCase serde, compiles |
| `src-tauri/src/ffmpeg/executor.rs` | Emit batch-file-progress with PerFileProgress | ✓ VERIFIED | Emits at lines 114-124; frame, fps, ETA computed; no batch-progress collision |
| `src-tauri/src/commands/batch.rs` | Batch lifecycle commands | ⚠️ PARTIAL | Cancel/complete flow works; missing initial batch-progress emission after state init |
| `src/types/batch.ts` | PerFileProgress TypeScript interface (6 fields) | ✓ VERIFIED | Exists at line 24, all camelCase fields matching Rust struct |
| `src/locales/en.json` | 18 new batch.* + batch.summary.* keys | ✓ VERIFIED | All 18 keys present (9 top-level + 9 summary), valid JSON |
| `src/locales/zh-CN.json` | 18 new batch.* + batch.summary.* keys (Chinese) | ✓ VERIFIED | All 18 keys present, valid JSON |
| `src/stores/batch.ts` | perFileProgress Map, cancelling ref, setPerFileProgress, setCancelling | ✓ VERIFIED | Map reactivity trigger at line 64, all mutations exported, state reset in stopProcessing/resetBatch |
| `src/composables/useBatch.ts` | 6 event listeners with proper cleanup | ✓ VERIFIED | All 6 listeners registered (typed generics), all 6 cleaned up in unsubscribe() |
| `src/components/batch/BatchBanner.vue` | Multi-state banner (processing/cancelling/complete) | ✓ VERIFIED | 3 computed states via bannerState, 2 color modes (blue/green), per-file label |
| `src/components/batch/BatchControls.vue` | Cancel confirmation dialog, no-output-dir validation, 3 button states | ✓ VERIFIED | dialog.warning() confirmation, output dir check, cancelling loading state, seed/queue guards |
| `src/components/batch/BatchSummary.vue` | Completion summary with succeeded/failed lists | ✓ VERIFIED | Inline panel, 200px section scrolls, Clear Results button, wasCancelled detection |
| `src/components/queue/ImportZone.vue` | Hidden during batch processing | ✓ VERIFIED | v-if="!batchStore.isProcessing" on root div |
| `src/components/queue/QueueList.vue` | Per-file progress bars, disabled controls | ✓ VERIFIED | NProgress with frame/ETA, remove/clear/add all disabled during processing |
| `src/components/MainLayout.vue` | Conditional BatchBanner and BatchSummary rendering | ✓ VERIFIED | Banner for all non-idle states, Summary when isComplete && lastResult |

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| executor.rs FfmpegEvent::Progress | app.emit("batch-file-progress", PerFileProgress) | PerFileProgress from FfmpegProgress fields | ✓ WIRED | executor.rs:114-124 extracts frame, fps, speed and computes percent/remaining |
| executor.rs FfmpegProgress fields | PerFileProgress struct fields | Field extraction: progress.frame, .fps, .speed | ✓ WIRED | executor.rs:107-112 |
| PerFileProgress TS interface | PerFileProgress Rust struct | camelCase field name match (serde rename_all) | ✓ WIRED | types/batch.ts:24-37 matches models/batch.rs:57-70 field-for-field |
| en.json batch.summary.* keys | BatchSummary.vue | vue-i18n $t() calls | ✓ WIRED | All 9 summary keys referenced in template |
| useBatch.ts listen('batch-file-progress') | store.setPerFileProgress(event.payload) | Tauri event listener callback | ✓ WIRED | useBatch.ts:43-45 |
| useBatch.ts listen('batch-cancelling') | store.setCancelling(true) | Tauri event listener callback | ✓ WIRED | useBatch.ts:47-49 |
| useBatch.ts listen('batch-file-error') | message.error(t('batch.fileFailed', ...)) | Naive UI useMessage() | ✓ WIRED | useBatch.ts:26-32 |
| BatchControls.vue onCancel() | useBatch.cancelBatch() | dialog.warning() onPositiveClick | ✓ WIRED | BatchControls.vue:119-131 |
| BatchBanner.vue template | batchStore.cancelling | computed bannerState | ✓ WIRED | BatchBanner.vue:11 |
| QueueList.vue NProgress | batchStore.currentFileProgress | computed on perFileProgress Map | ✓ WIRED | QueueList.vue:32-35, NProgress at 200-205 |
| BatchSummary.vue | batchStore.lastResult | reads BatchResult { succeeded[], failed[] } | ✓ WIRED | BatchSummary.vue:11,37 |
| MainLayout.vue | BatchSummary import + v-if | batchStore.isComplete && lastResult | ✓ WIRED | MainLayout.vue:13,92-93 |
| Rust start_batch init (line 124-136) | Frontend batch-progress listener | Initial batch-progress emission | ✗ NOT_WIRED | BatchProgress initialized with correct total but never emitted; frontend uses hardcoded startProcessing(1) |
| Frontend startBatch() (line 53-65) | batchStore.startProcessing(n) | Actual queue entry count from caller | ✗ NOT_WIRED | Hardcoded to 1; BatchControls.vue call site does not pass queue size |

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| BatchBanner.vue | overallPercent | batchStore.progress (completed/total from Rust) | Yes -- set by Rust batch-progress event (commands/batch.rs:206) | ✓ FLOWING |
| QueueList.vue NProgress | currentFileProgress.percent | batchStore.perFileProgress Map from batch-file-progress events | Yes -- populated by Rust executor from live FFmpeg iteration | ✓ FLOWING |
| QueueList.vue frame counter | currentFileProgress.currentFrame | ffmpeg-sidecar FfmpegProgress.frame | Yes -- direct from FFmpeg stderr progress parsing | ✓ FLOWING |
| BatchSummary.vue succeeded list | result.succeeded | Rust batch loop collects output paths | Yes -- paths from executor.rs make_output_path() | ✓ FLOWING |
| BatchSummary.vue failed list | result.failed | Rust batch loop emits FileResult via batch-file-error | Yes -- errors from FFmpeg exit codes and I/O failures | ✓ FLOWING |
| Per-file progress percent for short videos | percent computed in executor.rs | parse_time_to_seconds(progress.time) | ⚠️ STATIC (0.0) for MM:SS format | 0% displayed for videos under 1 hour |

## Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| BATCH-02 | 04-01, 04-02, 04-03, 04-05 | Processing shows per-file progress (percentage, current frame, estimated remaining time) | ✓ SATISFIED (with gap) | Executor emits batch-file-progress with all fields; QueueList renders NProgress + frame counter + ETA; parse_time_to_seconds bug means progress shows 0% for short videos (<1 hour) |
| BATCH-05 | 04-02, 04-03, 04-04, 04-05 | After batch processing, display results summary (success/failure counts) | ✓ SATISFIED (with gap) | BatchSummary shows succeeded/failed counts and per-file details; BatchBanner title misleading for completed-with-failures (shows "Cancelled") |

## Anti-Patterns Found

| File | Line(s) | Pattern | Severity | Impact |
|------|---------|---------|----------|--------|
| `src-tauri/src/commands/batch.rs` | 124-136 | Missing initial batch-progress event emission after state init | 🛑 BLOCKER | Overall progress shows wrong total (1) during first file; root cause of Gap 3 |
| `src/composables/useBatch.ts` | 59 | Hardcoded `startProcessing(1)`; total parameter not threaded from caller | 🛑 BLOCKER | Complements the missing Rust emission gap; progress bar reads "0/1" |
| `src-tauri/src/ffmpeg/executor.rs` | 153-166 | parse_time_to_seconds only handles HH:MM:SS, not MM:SS | 🛑 BLOCKER | Per-file progress stays at 0% for videos <1 hour; root cause of Gap 1 |
| `src/components/batch/BatchBanner.vue` | 19-20 | `failed.length` used as cancellation proxy for labelText | 🛑 BLOCKER | Banner shows "Batch Cancelled" for completed batches with any failed files; root cause of Gap 2 |
| `src-tauri/src/state.rs` | 33, 43 | Dead `cancel_flag` field in BatchState -- never read/written | ⚠️ WARNING | Actual cancel uses global `BATCH_CANCEL` static in batch.rs; dead field confuses future maintainers |
| `src/composables/useBatch.ts` | 79-88 | `getBatchStatus()` defined and exported but never called | ⚠️ WARNING | No component invokes it; Rust `get_batch_status` command also unused |
| `src-tauri/src/commands/batch.rs` | 94 | `_concurrency` read from store but intentionally ignored | ⚠️ WARNING | Batch always processes sequentially regardless of user's concurrency setting |
| `src/composables/useBatch.ts` | 21-49 | Sequential `await listen()` -- error in one drops remaining listeners | ⚠️ WARNING | If any listen() throws, 5 remaining listeners never register |
| `src-tauri/src/ffmpeg/executor.rs` | 11 | Unused `use serde::Serialize` import | ℹ️ INFO | `PerFileProgress` derives Serialize in batch.rs; no serialize usage in executor.rs |
| `src/locales/en.json` | 90-91 | `batch.completed` and `batch.cancelled` defined but never referenced | ℹ️ INFO | Replaced by `batch.summary.completeTitle` / `cancelledTitle` |
| `src/locales/zh-CN.json` | 90-91 | Same unused keys in Chinese locale | ℹ️ INFO | Mirror of English unused keys |
| `src/composables/useBatch.ts` | 60, 71, 84 | `console.error` in production code paths | ℹ️ INFO | Should guard with `import.meta.env.DEV` or use structured logger |
| `src/composables/useBatch.ts` | 8-13 | Module-level mutable unlisten variables (pseudo-singleton pattern) | ℹ️ INFO | Would silently break if multiple components called useBatch(); not an issue with current single-use pattern |
| `src-tauri/src/ffmpeg/executor.rs` | 40 | `ffmpeg_path` parameter is actually a directory, not an executable path | ℹ️ INFO | Misleading parameter name; caller in batch.rs reads it correctly as directory |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Rust compilation | `cargo check -p sandwich 2>&1` | Compiles successfully (exit 0) | ✓ PASS |
| PerFileProgress struct exists | `grep "struct PerFileProgress" src-tauri/src/models/batch.rs` | 1 match at line 57 | ✓ PASS |
| batch-file-progress emission | `grep "batch-file-progress" src-tauri/src/ffmpeg/executor.rs` | 1 match at line 115 | ✓ PASS |
| No batch-progress in executor | `grep '"batch-progress"' src-tauri/src/ffmpeg/executor.rs` | 0 matches | ✓ PASS |
| TypeScript interface | `grep "interface PerFileProgress" src/types/batch.ts` | 1 match at line 24 | ✓ PASS |
| 18 i18n keys in en.json | Manual verification of all batch.* and batch.summary.* keys | All 18 present, valid JSON | ✓ PASS |
| 18 i18n keys in zh-CN.json | Manual verification of all batch.* and batch.summary.* keys | All 18 present, valid JSON | ✓ PASS |
| Vue type checking | `npx vue-tsc -b --noEmit 2>&1` | TS6310 tsconfig error (pre-existing project config issue, unrelated to Phase 4) | ? SKIP |
| perFileProgress in store | `grep "perFileProgress" src/stores/batch.ts` | 5 matches | ✓ PASS |
| 6 event listeners | `grep -c "listen<" src/composables/useBatch.ts` | 6 listeners | ✓ PASS |
| 6 unlisten cleanup | `grep "unlisten" src/composables/useBatch.ts` | All 6 in unsubscribe() | ✓ PASS |
| BatchBanner 3 states | `grep -E "cancelling|complete|processing" src/components/batch/BatchBanner.vue` | All handled in bannerState computed | ✓ PASS |
| Cancel dialog | `grep "dialog.warning" src/components/batch/BatchControls.vue` | 1 match at line 120 | ✓ PASS |
| ImportZone guarded | `grep "batchStore.isProcessing" src/components/queue/ImportZone.vue` | v-if="!batchStore.isProcessing" at line 99 | ✓ PASS |
| QueueList per-file bar | `grep "NProgress" src/components/queue/QueueList.vue` | 2 matches (import + usage) | ✓ PASS |
| BatchSummary exists | `test -f src/components/batch/BatchSummary.vue` | File exists | ✓ PASS |
| MainLayout renders Summary | `grep "BatchSummary" src/components/MainLayout.vue` | 2 matches (import + usage) | ✓ PASS |
| parse_time_to_seconds handles MM:SS | Code review IN-05 confirmed in code | Only handles 3-part colon format; 2-part unsupported | ✗ FAIL |
| Initial batch-progress after state init | Code review CR-01 confirmed in code | No emission between lines 136-137 | ✗ FAIL |
| Correct total in startProcessing | `grep "startProcessing" src/composables/useBatch.ts` | Hardcoded to 1 at line 59 | ✗ FAIL |

## Human Verification Required

### 1. Cancel Flow with Running FFmpeg Process

**Test:** Start a batch with a real video and cancel it mid-processing.
**Expected:** Clicking Cancel shows the dialog. On confirm, the banner shows "Cancelling...", the cancel button becomes a disabled loading state, FFmpeg terminates, and the app returns to idle with the appropriate batch-cancelled summary.
**Why human:** Requires a configured FFmpeg binary and real video files for processing.

### 2. Per-File Progress Bars During Real Processing

**Test:** Process a batch of videos and observe the QueueList during processing.
**Expected:** The currently-processing file shows an NProgress bar with percentage, current frame / total frames, and estimated remaining minutes. The bar progresses smoothly (subject to FFmpeg's ~1Hz progress output). The overall BatchBanner shows completed/total.
**Why human:** Requires actual FFmpeg processing with real video files to observe real-time progress events and verify UI rendering timing.

### 3. End-to-End Workflow

**Test:** The full workflow: generate seed -> drag in videos -> select seed -> click process -> watch live progress per file -> review completion summary.
**Expected:** All steps work without error. Progress visible throughout. Summary shows correct counts with output paths. Clear Results returns to idle.
**Why human:** Full integration test spanning seed generation, video import, batch processing, and result display -- requires application to be runnable with all systems operational.

### 4. Mixed Success/Failure Summary

**Test:** Process a batch where some files succeed and some fail (e.g., include intentionally corrupted files).
**Expected:** Summary shows correct succeeded/failed counts, per-file output paths for succeeded files, per-file error messages for failed files. Banner title correctly indicates completion (not cancellation).
**Why human:** Requires input files that will produce both success and failure outcomes during processing, plus verification that the UI presentation is accurate (especially the banner title gap).

## Gaps Summary

All 18 PLAN must-have truths are verified -- the Phase 4 architecture is implemented and wired: the Rust executor emits rich per-file progress, the Pinia store tracks it reactively, the useBatch composable routes 6 batch events with proper cleanup, and the Vue components render multi-state UI across processing/cancelling/complete states.

However, **3 gaps** prevent the Phase goal from being fully achieved against the ROADMAP success criteria:

1. **parse_time_to_seconds MM:SS bug (Gap 1):** `executor.rs:153-166` does not handle the 2-part colon-separated time format (`MM:SS.mm`) that FFmpeg emits for videos under 1 hour. Per-file progress percent stays at 0% for all short videos. This is a **functional correctness bug** -- the "percentage complete" requirement is broken for an entire class of videos.

2. **Wrong BatchBanner title for completed-with-failures (Gap 2):** `BatchBanner.vue:19-20` uses `failed.length > 0` as a proxy for cancellation detection. A batch that fully completes with some failed files displays "Batch Cancelled" instead of "Batch Complete." The BatchSummary component uses correct logic, but the banner does not.

3. **Missing initial batch-progress event (Gap 3):** `commands/batch.rs:124-136` initializes `BatchProgress` with the correct total but never emits it. The frontend `useBatch.ts:59` fills the gap with a hardcoded `startProcessing(1)`, fixing the `isProcessing` visibility issue (original CR-01) but leaving the total count wrong during the first file's processing.

The cancel flow (SC-3) is fully implemented and verified: dialog confirmation, cancelling transitional state, cancel flag checked both between files and mid-FFmpeg iteration, graceful process termination, and clean state reset.

---

_Verified: 2026-05-14T22:00:00Z_
_Verifier: Claude (gsd-verifier)_
