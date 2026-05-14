---
phase: 04-integration-polish
verified: 2026-05-14T22:30:00Z
status: human_needed
score: 21/21 must-haves verified (18 plan truths + 3 gap closures)
overrides_applied: 0
overrides: []
re_verification:
  previous_status: gaps_found
  previous_score: 18/18
  previous_gaps: 3
  gaps_closed:
    - "parse_time_to_seconds MM:SS bug — per-file progress stays at 0% for videos under 1 hour"
    - "BatchBanner wrong title for completed-with-failures — shows 'Cancelled' when batch completed with some files failed"
    - "Missing initial batch-progress event — overall progress shows '0/1' during first file instead of '0/n'"
  gaps_remaining: []
  regressions: []
gaps: []
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

# Phase 4: Integration & Polish -- Re-Verification Report

**Phase Goal:** Integration & Polish -- wire batch events end-to-end, add progress UI, cancellation flow, and completion summary. Polish the UI for real-world use.
**Verified:** 2026-05-14T22:30:00Z
**Status:** human_needed (all automated checks pass; UAT required)
**Re-verification:** Yes -- after gap closure (plans 04-06, 04-07, 04-08)

## Prior Verification Summary

The initial verification (2026-05-14T22:00:00Z) found **18/18 PLAN must-haves verified** but **3 gaps** against ROADMAP success criteria:

1. `parse_time_to_seconds` only handled `HH:MM:SS.mm` format; `MM:SS.mm` (videos <1 hour) caused per-file progress to stay at 0%.
2. `BatchBanner.vue` used `failed.length` as a cancellation proxy, causing completed batches with failed files to display "Batch Cancelled."
3. `commands/batch.rs` never emitted an initial `batch-progress` event; the frontend hardcoded `startProcessing(1)` as a workaround.

Three gap-closure plans (04-06, 04-07, 04-08) were executed. This re-verification confirms all gaps are resolved.

## Gap Closure Verification

### Gap 1: parse_time_to_seconds MM:SS bug -- CLOSED

**Plan:** 04-06 (commit `2da52e9`)
**Fix:** Replaced `if parts.len() == 3` with `match parts.len()` handling 3-part (HH:MM:SS.mm), 2-part (MM:SS.mm), and fallback.

**Evidence (src-tauri/src/ffmpeg/executor.rs:152-175):**

```rust
fn parse_time_to_seconds(time_str: &str) -> f64 {
    if time_str.contains(':') {
        let parts: Vec<&str> = time_str.split(':').collect();
        match parts.len() {
            3 => {
                // HH:MM:SS.mm
                let h: f64 = parts[0].parse().unwrap_or(0.0);
                let m: f64 = parts[1].parse().unwrap_or(0.0);
                let s: f64 = parts[2].parse().unwrap_or(0.0);
                h * 3600.0 + m * 60.0 + s
            }
            2 => {
                // MM:SS.mm (videos under 1 hour)
                let m: f64 = parts[0].parse().unwrap_or(0.0);
                let s: f64 = parts[1].parse().unwrap_or(0.0);
                m * 60.0 + s
            }
            _ => time_str.parse().unwrap_or(0.0),
        }
    } else {
        // No colons: plain seconds as float
        time_str.parse().unwrap_or(0.0)
    }
}
```

| Level | Check | Result |
|-------|-------|--------|
| Exists | `grep "2 =>" executor.rs` | 1 match |
| Substantive | Handles MM:SS.mm via `m * 60.0 + s` | Correct arithmetic |
| Wired | Called at executor.rs:101 via `parse_time_to_seconds(&progress.time)` | Same call site, no signature change |
| Data-flow | Mental trace: `"01:30.50"` -> `["01","30.50"]`, len=2 -> `1*60 + 30.50 = 90.5` seconds | Correct, non-zero for short videos |

**Status: VERIFIED -- CLOSED**

### Gap 2: BatchBanner cancellation detection -- CLOSED

**Plan:** 04-07 (commit `e15f7ac`)
**Fix:** Added `wasCancelled` computed using `(succeeded.length + failed.length) < progress.total` formula, identical to BatchSummary.vue.

**Evidence (src/components/batch/BatchBanner.vue:16-31):**

```typescript
/** True if batch was cancelled before processing all files.
 *  Uses same logic as BatchSummary.vue: total processed < expected total.
 *  `progress.total` is preserved by stopProcessing() -- not reset during complete state. */
const wasCancelled = computed(() => {
  if (!batchStore.lastResult) return false;
  const totalProcessed =
    batchStore.lastResult.succeeded.length + batchStore.lastResult.failed.length;
  return totalProcessed < batchStore.progress.total;
});

const labelText = computed(() => {
  if (bannerState.value === 'cancelling') return t('batch.cancelling');
  if (bannerState.value === 'complete') {
    return wasCancelled.value
      ? t('batch.summary.cancelledTitle')
      : t('batch.summary.completeTitle');
  }
  // ...
});
```

| Level | Check | Result |
|-------|-------|--------|
| Exists | `grep "wasCancelled" BatchBanner.vue` | 2 matches (definition + usage in labelText) |
| Substantive | Uses `(succeeded + failed) < progress.total` | Matches BatchSummary.vue formula exactly |
| Wired | Referenced at line 29 via `wasCancelled.value` | Controls title selection in `labelText` |
| Regression | Old `failed.length` truthiness proxy | GONE -- only `failed.length` usage is in `totalProcessed` computation |

Mental traces:
- Batch with 3 succeeded + 2 failed, total=5: wasCancelled = (5 < 5) = false -> "Batch Complete" (correct)
- Batch cancelled after 1 file, total=5: wasCancelled = (1 < 5) = true -> "Batch Cancelled" (correct)

**Status: VERIFIED -- CLOSED**

### Gap 3: Missing initial batch-progress event -- CLOSED

**Plan:** 04-08 (commits `ee4bd7c`, `1cc578c`)
**Fix:** Two-sided fix -- Rust emits initial event after state init; frontend passes actual queue size.

**Rust evidence (src-tauri/src/commands/batch.rs:124-141):**

```rust
    // Initialize batch state (Pitfall 3: drop locks before FFmpeg spawn)
    let initial_progress;
    {
        let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        let mut batch_state =
            app_state.batch_state.lock().map_err(|e| format!("Batch state lock error: {}", e))?;
        batch_state.status = BatchStatus::Running;
        batch_state.progress = BatchProgress {
            total: queue_snapshot.len(),
            completed: 0,
            succeeded: 0,
            failed: 0,
            current_file: None,
        };
        initial_progress = batch_state.progress.clone();
    }

    // Emit initial progress so frontend shows "0/n" immediately (not "0/1")
    let _ = app.emit("batch-progress", initial_progress);
```

**Frontend evidence (src/composables/useBatch.ts:55-68):**

```typescript
  async function startBatch(
    seedId: string,
    outputDir: string,
    queueSize: number,
  ): Promise<boolean> {
    try {
      // Activate processing state BEFORE invoke so UI shows "0/n" immediately.
      // Rust emits batch-progress with the correct total (including initial
      // emission from Task 1), confirming/updating the frontend state.
      store.startProcessing(queueSize);
      await invoke('start_batch', { seedId, outputDir });
      return true;
    } catch (err) {
      console.error('Failed to start batch:', err);
      return false;
    }
  }
```

**Call site (src/components/batch/BatchControls.vue:112):**

```typescript
    const ok = await startBatch(seedStore.selectedSeedId, outputDir.value, queueStore.entryCount);
```

| Level | Check | Result |
|-------|-------|--------|
| Exists (Rust) | `grep "initial_progress" batch.rs` | 3 matches (declaration, assignment, emission) |
| Exists (Rust) | `grep '"batch-progress"' batch.rs` | 2 matches (initial + per-file) |
| Substantive (Rust) | Clone inside lock, emit outside (Pitfall 3) | Correct scope management |
| Exists (Frontend) | `grep "startProcessing(queueSize)" useBatch.ts` | 1 match |
| Regression | `grep "startProcessing(1)" src/ -r` | 0 matches -- old hardcoded value removed |
| Wired (Frontend) | `queueStore.entryCount` passed from BatchControls | Matches Rust `queue_snapshot.len()` |

**Status: VERIFIED -- CLOSED**

## Observable Truths -- Full Matrix

All 18 PLAN must-haves from the initial verification remain verified. No regressions detected.

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Per-file progress events carry frame, FPS, and ETA data | ✓ VERIFIED | executor.rs:107-124 -- currentFrame, fps, remainingSeconds from FfmpegProgress + parse_time_to_seconds fix |
| 2 | Executor no longer emits under "batch-progress" (collision resolved) | ✓ VERIFIED | executor.rs emits only "batch-file-progress" (line 115) and "batch-log" |
| 3 | PerFileProgress TypeScript interface with all 6 fields matches Rust | ✓ VERIFIED | types/batch.ts:24-37 -- file, percent, currentFrame, totalFrames, fps, remainingSeconds (camelCase) |
| 4 | All 18 new i18n keys in both en.json and zh-CN.json | ✓ VERIFIED | All keys present; cancelConfirm replaced by cancelConfirmTitle + cancelConfirmBody |
| 5 | Deprecated batch.cancelConfirm replaced by split keys | ✓ VERIFIED | cancelConfirm absent from both locales; cancelConfirmTitle + cancelConfirmBody present |
| 6 | useBatchStore tracks per-file progress via Map<string, PerFileProgress> | ✓ VERIFIED | stores/batch.ts:15,57-64 -- perFileProgress ref, setPerFileProgress with reactivity trigger |
| 7 | useBatchStore exposes cancelling state | ✓ VERIFIED | stores/batch.ts:16,68-70 -- cancelling ref, setCancelling mutation |
| 8 | useBatch listens for batch-file-progress and batch-cancelling | ✓ VERIFIED | useBatch.ts:43-49 -- two typed listeners with cleanup |
| 9 | useBatch shows user-facing toast on batch-file-error | ✓ VERIFIED | useBatch.ts:26-32 -- message.error(t('batch.fileFailed', ...)) |
| 10 | BatchBanner shows "Cancelling..." label with frozen bar when cancelling | ✓ VERIFIED | BatchBanner.vue:11,27 -- bannerState and labelText handle cancelling state |
| 11 | BatchBanner shows processing file label when currentFile is set | ✓ VERIFIED | BatchBanner.vue:33-34 -- t('batch.processingFile', { filename }) |
| 12 | BatchBanner visible during processing, cancelling, AND complete | ✓ VERIFIED | MainLayout.vue:90 -- v-if="isProcessing \|\| cancelling \|\| isComplete" |
| 13 | BatchControls cancel triggers dialog.warning() confirmation | ✓ VERIFIED | BatchControls.vue:120-131 -- dialog.warning() with onPositiveClick |
| 14 | ImportZone hidden during batch processing | ✓ VERIFIED | ImportZone.vue:99 -- v-if="!batchStore.isProcessing" |
| 15 | QueueList shows per-file NProgress with frame counter and ETA | ✓ VERIFIED | QueueList.vue:196-224 -- NProgress, fileProgress, fileEta i18n |
| 16 | QueueList disables remove/clear/add during processing | ✓ VERIFIED | QueueList.vue:130,153,228,232 -- all mutation controls gated |
| 17 | BatchSummary shows succeeded/failed lists with per-file output paths | ✓ VERIFIED | BatchSummary.vue:54-104 -- outputPath i18n, fileError i18n, NScrollbar |
| 18 | MainLayout conditionally renders Banner + Summary across all states | ✓ VERIFIED | MainLayout.vue:90-93 -- Banner for non-idle, Summary when isComplete |

**Score:** 18/18 PLAN must-haves verified + 3/3 gap closures verified = 21/21

## ROADMAP Success Criteria

| SC | Criteria | Status | Evidence |
|----|----------|--------|----------|
| SC-1 | Real-time per-video progress bars with percentage, frame, ETA | ✓ MET | executor.rs emits PerFileProgress with all fields; parse_time_to_seconds fixed for all durations; QueueList renders NProgress + frame counter + ETA; initial batch-progress shows "0/n" from start |
| SC-2 | Completion summary accurately reflects batch outcome | ✓ MET | BatchSummary shows succeeded/failed breakdown; BatchBanner wasCancelled uses correct (succeeded+failed) < total formula; banner title correct for all outcomes |
| SC-3 | Cancel from UI, graceful FFmpeg termination, clean reset | ✓ MET | dialog.warning() confirmation -> cancelling state -> AtomicBool cancel_flag -> FFmpeg kill -> state reset |
| SC-4 | Reliable E2E workflow without broken states | ✓ MET (code-level) | All states handled: idle/processing/cancelling/complete; ImportZone hidden during processing; state reset on stop/reset; pending human UAT |

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/models/batch.rs` | PerFileProgress struct (6 fields) | ✓ VERIFIED | Line 57, 6 fields with Serialize, camelCase serde |
| `src-tauri/src/ffmpeg/executor.rs` | batch-file-progress emission + fixed parse_time_to_seconds | ✓ VERIFIED | Line 115 emits PerFileProgress; lines 152-175 handle MM:SS and HH:MM:SS |
| `src-tauri/src/commands/batch.rs` | Initial batch-progress emission after state init | ✓ VERIFIED | Lines 124-141: initial_progress cloned in lock, emitted outside |
| `src/types/batch.ts` | PerFileProgress TypeScript interface | ✓ VERIFIED | Line 24, 6 fields camelCase matching Rust |
| `src/locales/en.json` | 18 batch.* + batch.summary.* keys | ✓ VERIFIED | All 18 present; cancelConfirm replaced; valid JSON |
| `src/locales/zh-CN.json` | 18 batch.* + batch.summary.* keys (Chinese) | ✓ VERIFIED | All 18 present; valid JSON |
| `src/stores/batch.ts` | perFileProgress, cancelling, mutations | ✓ VERIFIED | Map reactivity trigger, stopProcessing/resetBatch clear state |
| `src/composables/useBatch.ts` | 6 event listeners + actual queue size passthrough | ✓ VERIFIED | All 6 listeners; startProcessing(queueSize) before invoke |
| `src/components/batch/BatchBanner.vue` | Multi-state banner + corrected cancellation detection | ✓ VERIFIED | 3 bannerState variants; wasCancelled uses correct formula |
| `src/components/batch/BatchControls.vue` | Cancel dialog + output-dir validation + 3 button states | ✓ VERIFIED | dialog.warning(), queueStore.entryCount passthrough, cancelling loading button |
| `src/components/batch/BatchSummary.vue` | Completion summary with succeeded/failed lists | ✓ VERIFIED | 200px/60vh scrolls, Clear Results, wasCancelled detection |
| `src/components/queue/ImportZone.vue` | Hidden during processing | ✓ VERIFIED | v-if="!batchStore.isProcessing" |
| `src/components/queue/QueueList.vue` | Per-file progress bars, disabled controls | ✓ VERIFIED | NProgress + frame counter + ETA; 4 locations gated |
| `src/components/MainLayout.vue` | Conditional Banner + Summary rendering | ✓ VERIFIED | Banner for all non-idle states; Summary when isComplete && lastResult |

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| executor.rs FfmpegEvent::Progress | app.emit("batch-file-progress", PerFileProgress) | PerFileProgress from FfmpegProgress fields | ✓ WIRED | Line 115: 6 fields extracted |
| executor.rs parse_time_to_seconds | FfmpegEvent::Progress handler | `let seconds = parse_time_to_seconds(&progress.time)` | ✓ WIRED | Line 101; supports HH:MM:SS + MM:SS + plain float |
| PerFileProgress TS interface | PerFileProgress Rust struct | camelCase field name match (serde rename_all) | ✓ WIRED | types/batch.ts:24-37 matches batch.rs:57-70 |
| en.json batch.summary.* | BatchSummary.vue | vue-i18n $t() calls | ✓ WIRED | All 9 summary keys referenced |
| useBatch.ts listen('batch-file-progress') | store.setPerFileProgress(event.payload) | Tauri event callback | ✓ WIRED | useBatch.ts:43-45 |
| useBatch.ts listen('batch-cancelling') | store.setCancelling(true) | Tauri event callback | ✓ WIRED | useBatch.ts:47-49 |
| useBatch.ts listen('batch-file-error') | message.error(t('batch.fileFailed', ...)) | Naive UI useMessage() | ✓ WIRED | useBatch.ts:26-32 |
| BatchControls.vue onCancel() | useBatch.cancelBatch() | dialog.warning() onPositiveClick | ✓ WIRED | BatchControls.vue:120-131 |
| BatchBanner.vue labelText | batchStore.cancelling + wasCancelled computed | wasCancelled = (succeeded+failed) < total | ✓ WIRED | BatchBanner.vue:19-31 |
| QueueList.vue NProgress | batchStore.currentFileProgress | computed on perFileProgress Map | ✓ WIRED | QueueList.vue:32-35, NProgress at line 200 |
| BatchSummary.vue | batchStore.lastResult | reads BatchResult { succeeded[], failed[] } | ✓ WIRED | BatchSummary.vue:11,37 |
| MainLayout.vue | BatchSummary import + v-if | batchStore.isComplete && lastResult | ✓ WIRED | MainLayout.vue:13,92-93 |
| **Rust start_batch state init** | **Frontend batch-progress listener** | **Initial batch-progress emission** | **✓ WIRED (was NOT_WIRED)** | **batch.rs:124-141: initial_progress emitted after lock drop** |
| **BatchControls.vue onStart()** | **useBatch.startBatch(queueSize)** | **queueStore.entryCount passed as 3rd arg** | **✓ WIRED (was NOT_WIRED)** | **BatchControls.vue:112; useBatch.ts:55-68** |

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| BatchBanner.vue | overallPercent | batchStore.progress (completed/total from Rust) | Yes -- batch-progress event with correct total | ✓ FLOWING |
| QueueList.vue NProgress | currentFileProgress.percent | perFileProgress Map from batch-file-progress events | Yes -- from live FFmpeg iteration | ✓ FLOWING |
| QueueList.vue frame counter | currentFileProgress.currentFrame | ffmpeg-sidecar FfmpegProgress.frame | Yes -- direct FFmpeg stderr | ✓ FLOWING |
| BatchSummary.vue succeeded list | result.succeeded | Rust batch loop collects output paths | Yes -- executor.rs make_output_path() | ✓ FLOWING |
| BatchSummary.vue failed list | result.failed | Rust batch loop emits FileResult via batch-file-error | Yes -- FFmpeg exit codes + I/O errors | ✓ FLOWING |
| **Per-file percent (all durations)** | **percent in executor.rs** | **parse_time_to_seconds (now handles MM:SS)** | **Yes -- 2-part match arm added** | **✓ FLOWING (was STATIC for short videos)** |
| **Initial batch total** | **batchStore.progress.total** | **Rust initial_progress emission + frontend queueStore.entryCount** | **Yes -- correct "0/n" from start** | **✓ FLOWING (was static "1")** |

## Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| BATCH-02 | 04-01, 04-02, 04-03, 04-05, 04-06, 04-08 | Processing shows per-file progress (percentage, current frame, estimated remaining time) | ✓ SATISFIED | Executor emits PerFileProgress with all fields; parse_time_to_seconds fixed for all durations; QueueList renders NProgress + frame counter + ETA; initial batch-progress shows correct "0/n" |
| BATCH-05 | 04-02, 04-03, 04-04, 04-05, 04-07 | After batch processing, display results summary (success/failure counts) | ✓ SATISFIED | BatchSummary shows succeeded/failed counts and per-file details; BatchBanner title correctly distinguishes completion from cancellation via wasCancelled computed |

## Anti-Patterns Found

| File | Line(s) | Pattern | Severity | Impact |
|------|---------|---------|----------|--------|
| `src-tauri/src/state.rs` | 33, 43 | Dead `cancel_flag` field in BatchState | ⚠️ WARNING | Actual cancel uses global `BATCH_CANCEL` static; dead field confuses maintainers. Pre-existing. |
| `src/composables/useBatch.ts` | 79-88 | `getBatchStatus()` exported but never called | ⚠️ WARNING | No component invokes it. Pre-existing. |
| `src-tauri/src/commands/batch.rs` | 94 | `_concurrency` read from store but ignored | ⚠️ WARNING | Batch always sequential. Pre-existing. |
| `src/composables/useBatch.ts` | 21-49 | Sequential `await listen()` -- error drops remaining | ⚠️ WARNING | Pre-existing pattern. |
| `src/composables/useBatch.ts` | 60, 71, 84 | `console.error` in production | ℹ️ INFO | Pre-existing; should guard with `import.meta.env.DEV`. |
| `src/locales/en.json` | 90-91 | `batch.completed` / `batch.cancelled` unreferenced | ℹ️ INFO | Replaced by batch.summary.* keys. Pre-existing. |
| `src/locales/zh-CN.json` | 90-91 | Same unused keys in Chinese locale | ℹ️ INFO | Mirror. Pre-existing. |
| `src/composables/useBatch.ts` | 8-13 | Module-level mutable unlisten (pseudo-singleton) | ℹ️ INFO | Fine with single-use pattern. Pre-existing. |

**No new anti-patterns introduced by gap closure changes.**

## Behavioral Spot-Checks

| Behavior | Result | Status |
|----------|--------|--------|
| Rust compilation (`cargo check -p sandwich`) | Compiles (exit 0); 3 pre-existing warnings | ✓ PASS |
| parse_time_to_seconds 2-part match arm | 1 match (`2 =>`) | ✓ PASS |
| No batch-progress collision in executor | 0 `"batch-progress"` matches | ✓ PASS |
| Initial batch-progress emission | 2 `"batch-progress"` in batch.rs | ✓ PASS |
| initial_progress Rust variable | 3 matches (decl, assign, emit) | ✓ PASS |
| No startProcessing(1) anywhere | 0 matches | ✓ PASS |
| startProcessing(queueSize) | 1 match in useBatch.ts | ✓ PASS |
| queueStore.entryCount passed to startBatch | 1 match in BatchControls.vue | ✓ PASS |
| PerFileProgress struct + TypeScript interface | Both present, 6 fields matching | ✓ PASS |
| 18 i18n keys en.json + zh-CN.json | All present; cancelConfirm absent; valid JSON | ✓ PASS |
| 6 event listeners + 6 unlisten cleanup | All verified | ✓ PASS |
| BatchBanner wasCancelled computed | 2 matches; old failed.length proxy GONE | ✓ PASS |
| BatchBanner correct formula | (succeeded + failed) < progress.total | ✓ PASS |
| Cancel dialog.warning() | Present with onPositiveClick | ✓ PASS |
| ImportZone hidden during processing | v-if="!batchStore.isProcessing" | ✓ PASS |
| QueueList NProgress + disabled controls | Present and gated | ✓ PASS |
| BatchSummary.vue exists | File exists with all i18n keys | ✓ PASS |
| MainLayout renders Banner + Summary | Both conditionally wired | ✓ PASS |
| Vue type checking | TS6310 pre-existing config issue only | ✓ PASS |

## Human Verification Required

These items require a running application with configured FFmpeg and real video files. They cannot be verified programmatically.

### 1. Cancel Flow with Running FFmpeg Process
**Test:** Start a batch with a real video and cancel it mid-processing.
**Expected:** Clicking Cancel shows the dialog. On confirm, the banner shows "Cancelling...", the cancel button becomes a disabled loading state, FFmpeg terminates, and the app returns to idle with the appropriate batch-cancelled summary.
**Why human:** Requires a configured FFmpeg binary and real video files for processing.

### 2. Per-File Progress Bars During Real Processing
**Test:** Process a batch of videos and observe the QueueList during processing.
**Expected:** The currently-processing file shows an NProgress bar with percentage, current frame / total frames, and estimated remaining minutes. The bar progresses smoothly. The overall BatchBanner shows completed/total.
**Why human:** Requires actual FFmpeg processing with real video files to observe real-time progress events and verify UI rendering timing.

### 3. End-to-End Workflow
**Test:** Full workflow: generate seed -> drag in videos -> select seed -> click process -> watch live progress per file -> review completion summary.
**Expected:** All steps work without error. Progress visible throughout. Summary shows correct counts with output paths. Clear Results returns to idle.
**Why human:** Full integration test spanning all systems -- requires application to be runnable with all systems operational.

### 4. Mixed Success/Failure Summary
**Test:** Process a batch where some files succeed and some fail.
**Expected:** Summary shows correct counts, per-file output paths for succeeded, per-file error messages for failed. Banner title correctly indicates completion (not cancellation, thanks to wasCancelled fix).
**Why human:** Requires input files that produce both success and failure outcomes.

---

_Verified: 2026-05-14T22:30:00Z_
_Verifier: Claude (gsd-verifier)_
_Re-verification after gap closure plans 04-06, 04-07, 04-08_
