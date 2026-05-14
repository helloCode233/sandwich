---
phase: 04-integration-polish
reviewed: 2026-05-14T20:00:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - src-tauri/src/ffmpeg/executor.rs
  - src-tauri/src/models/batch.rs
  - src-tauri/src/commands/batch.rs
  - src-tauri/src/state.rs
  - src-tauri/src/lib.rs
  - src/types/batch.ts
  - src/locales/en.json
  - src/locales/zh-CN.json
  - src/stores/batch.ts
  - src/composables/useBatch.ts
  - src/components/batch/BatchBanner.vue
  - src/components/batch/BatchControls.vue
  - src/components/batch/BatchSummary.vue
  - src/components/queue/ImportZone.vue
  - src/components/queue/QueueList.vue
  - src/components/MainLayout.vue
findings:
  critical: 1
  warning: 5
  info: 6
  total: 12
status: issues_found
---

# Phase 4: Code Review Report

**Reviewed:** 2026-05-14T20:00:00Z
**Depth:** standard
**Files Reviewed:** 16 (13 from scope + 3 cross-referenced)
**Status:** issues_found

## Summary

Phase 4 (Integration & Polish) wires real-time progress streaming from Rust to Vue, adds completion summary, cancel confirmation, and per-file progress bars. The architecture is sound: Rust emits Tauri events for progress/error/completion/cancellation, the Pinia store tracks state reactively, and Vue components render accordingly. The cancel flow uses a global `AtomicBool` checked both between files and mid-FFmpeg iteration.

However, one **blocker** was found: the batch processing UI never activates until the first file completes. No `batch-progress` event is emitted at batch start, and the frontend never calls `startProcessing()`. This means the processing banner, cancel button, and queue-lock guard UI are all invisible during the first file -- which could be minutes of confusion for long videos.

Five warnings and six informational items are also identified, covering dead code, misleading UI labels, fragile event subscription patterns, and unused i18n keys.

---

## Critical Issues

### CR-01: Missing Initial Processing State Activation -- UI Invisible During First File

**File:** `src-tauri/src/commands/batch.rs:124-136`, `src/composables/useBatch.ts:53-63`
**Issue:** When a batch starts, neither the Rust backend nor the Vue frontend signals that processing has begun. The Rust `start_batch` initializes `BatchProgress` (total=n, completed=0) in managed state at lines 124-136 but never emits a `batch-progress` event with this initial state. Instead, the first `batch-progress` emission occurs at lines 200-207, after the first file completes. Meanwhile, the Vue composable `useBatch.ts` `startBatch()` function (lines 53-63) only calls `invoke('start_batch', ...)` without calling `store.startProcessing()`. The Pinia store's `setProgress()` at `stores/batch.ts:32-34` sets `isProcessing = p.total > 0 && p.completed < p.total`, which remains `false` until the first `batch-progress` event arrives.

**Consequences:**
- `batchStore.isProcessing` stays `false` during the first file
- `BatchBanner` (processing progress bar) never renders
- Cancel button never appears (gated on `isProcessing || cancelling`)
- `ImportZone` remains active (gated on `!isProcessing`) -- user can add files mid-batch
- Queue Clear All / Remove buttons remain enabled (gated on `!isProcessing`) -- user can mutate queue mid-batch
- The `startDisabled` guard only prevents re-click but the Rust double-check is the real guard

**Fix:** Two complementary changes are needed:

1. In `src-tauri/src/commands/batch.rs`, emit an initial `batch-progress` event right after initializing batch state (after line 136):

```rust
// After initializing batch state, emit initial progress to frontend
{
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let batch_state = app_state.batch_state.lock()
        .map_err(|e| format!("Batch state lock error: {}", e))?;
    let _ = app.emit("batch-progress", batch_state.progress.clone());
}
```

2. In `src/composables/useBatch.ts`, call `store.startProcessing(total)` before invoking the command:

```typescript
async function startBatch(seedId: string, outputDir: string, totalFiles: number): Promise<boolean> {
  try {
    store.startProcessing(totalFiles);
    await invoke('start_batch', { seedId, outputDir });
    return true;
  } catch (err) {
    store.resetBatch(); // rollback UI state on error
    console.error('Failed to start batch:', err);
    return false;
  }
}
```

And update the caller in `BatchControls.vue` to pass the queue size:

```typescript
const ok = await startBatch(
  seedStore.selectedSeedId,
  outputDir.value,
  queueStore.entryCount
);
```

---

## Warnings

### WR-01: BatchBanner Shows "Cancelled" for Completed Batches with Failures

**File:** `src/components/batch/BatchBanner.vue:19-21`
**Issue:** The `labelText` computed property uses `lastResult?.failed.length` as a proxy for whether the batch was cancelled:
```typescript
return batchStore.lastResult?.failed.length
  ? t('batch.summary.cancelledTitle')
  : t('batch.summary.completeTitle');
```
A batch that completes with some failed files (e.g., 3 succeeded, 2 failed) will display "Batch Cancelled" even though it fully completed. The `wasCancelled` logic in `BatchSummary.vue` correctly distinguishes completion vs. cancellation, but `BatchBanner` uses the wrong heuristic.

**Fix:** Use the same `wasCancelled` logic as `BatchSummary`, or pass a `wasCancelled` flag from the store:
```typescript
// In the store, add a derived wasCancelled or a boolean flag  
// Then in BatchBanner:
if (bannerState.value === 'complete') {
  return batchStore.wasCancelled
    ? t('batch.summary.cancelledTitle')
    : t('batch.summary.completeTitle');
}
```

### WR-02: Dead `BatchState.cancel_flag` Field

**File:** `src-tauri/src/state.rs:33`, `src-tauri/src/commands/batch.rs:97-101`
**Issue:** `BatchState` defines `pub cancel_flag: AtomicBool` (line 33) that is initialized in `Default::default()` (line 43) but is never read or written anywhere in the codebase. The actual cancel mechanism uses a separate global static `BATCH_CANCEL: OnceLock<TokioMutex<Option<Arc<AtomicBool>>>>` in `batch.rs` (lines 21-25). The `start_batch` command creates an `Arc<AtomicBool>`, stores it in the global static (line 100), and passes it to `execute_single_file()`. The `cancel_batch` command reads from the same global static to set the flag.

Having two cancel flags -- one unused in `BatchState` and one actually functional in the global static -- creates confusion. Future maintainers checking `BatchState.cancel_flag` would observe incorrect state.

**Fix:** Remove `cancel_flag` from `BatchState` (lines 33, 43 in state.rs) and update the `Default` impl accordingly. The single cancel mechanism in the global static is sufficient.

### WR-03: `getBatchStatus` Exposed but Never Consumed by Any Component

**File:** `src/composables/useBatch.ts:77-86`
**Issue:** The `getBatchStatus()` function invokes the Tauri `get_batch_status` command and updates the store, but no Vue component ever calls it. The JSDoc comment says "useful on app re-focus to sync state" but there is no `onFocus` listener in `MainLayout.vue` or any other component. Other composables (`useSeed`, `useQueue`) load initial state in their `subscribe()` methods, but `useBatch` does not -- meaning batch state is never re-synced if the app loses focus and returns.

**Fix:** Either:
1. Call `getBatchStatus()` in `MainLayout.vue` when receiving a window focus event, or
2. Call it once in `subscribe()` to sync initial state on app startup, or
3. Remove the function if status sync is genuinely not needed (and remove the `get_batch_status` Rust command to avoid dead code in both tiers).

### WR-04: Concurrency Preference Read but Intentionally Ignored

**File:** `src-tauri/src/commands/batch.rs:94`
**Issue:** The `start_batch` command reads the concurrency preference from the store:
```rust
let _concurrency = get_concurrency_preference(&app);
```
The underscore prefix signals intentional non-use. The batch loop processes files sequentially with no concurrency implementation. The UI allows the user to select concurrency levels 1-4 (per D-08), and the preference is persisted to the store, but the backend ignores it entirely. This is an incomplete implementation of a documented requirement.

**Fix:** If sequential processing is intentional for Phase 4, add a `// TODO: D-08 -- implement concurrent processing` comment. Otherwise, implement concurrent processing using `tokio::task::spawn` or `futures::stream::iter` with a semaphore.

### WR-05: Fragile Event Subscription -- Partial Failure Drops Remaining Listeners

**File:** `src/composables/useBatch.ts:21-50`
**Issue:** The `subscribe()` function registers six event listeners sequentially with `await`:
```typescript
progressUnlisten = await listen<BatchProgress>('batch-progress', ...);
fileErrorUnlisten = await listen<FileResult>('batch-file-error', ...);
completeUnlisten   = await listen<BatchResult>('batch-complete', ...);
cancelledUnlisten  = await listen<BatchResult>('batch-cancelled', ...);
fileProgressUnlisten  = await listen<PerFileProgress>('batch-file-progress', ...);
cancellingUnlisten = await listen<void>('batch-cancelling', ...);
```
If any `listen()` call throws (e.g., network error, IPC channel failure), the function propagates the error and remaining listeners are never registered. For example, if `batch-file-error` registration fails, the `batch-complete`, `batch-cancelled`, `batch-file-progress`, and `batch-cancelling` listeners are silently absent. The already-registered listeners (e.g., `batch-progress`) remain active with no way to clean them up since `unsubscribe()` uses the partially-set module-level variables but won't be called (the error propagates out of `onMounted`).

**Fix:** Wrap each `listen()` in a try/catch that collects errors but continues registration, or use `Promise.allSettled()` to register all listeners independently:
```typescript
const results = await Promise.allSettled([
  listen<BatchProgress>('batch-progress', ...),
  listen<FileResult>('batch-file-error', ...),
  // ... etc
]);
// Collect unlisten fns from successful results, log failures from rejected ones
```

---

## Info

### IN-01: Unused `serde::Serialize` Import

**File:** `src-tauri/src/ffmpeg/executor.rs:11`
**Issue:** `use serde::Serialize;` is imported but never used. The `PerFileProgress` struct (used in the emit call at line 114) derives `Serialize` in `batch.rs`, not here. The `serde_json::json!` macro does not require a `Serialize` import. This import should be removed to keep the module clean.

**Fix:** Remove line 11: `use serde::Serialize;`

### IN-02: Console Logging Left in Production Code

**File:** `src/composables/useBatch.ts:60,71,84`, `src/components/batch/BatchControls.vue:53,67,88`, `src/components/queue/ImportZone.vue:59`
**Issue:** Multiple `console.error()` and `console.warn()` calls are present in production code paths. These are appropriate for development but should be routed through a proper logging mechanism or suppressed in production builds.

**Fix:** Replace with a structured logger (e.g., `tauri-plugin-log`) or guard with `import.meta.env.DEV`:
```typescript
if (import.meta.env.DEV) console.error('Failed to start batch:', err);
```

### IN-03: Unused i18n Keys

**File:** `src/locales/en.json:90,91`, `src/locales/zh-CN.json:90,91`
**Issue:** The i18n keys `batch.completed` ("Batch Complete") and `batch.cancelled` ("Batch Cancelled") are defined in both locale files but never referenced by any Vue component or composable. The completion and cancellation titles come from `batch.summary.completeTitle` and `batch.summary.cancelledTitle` instead.

**Fix:** Either:
1. Remove the unused keys to keep locale files lean, or
2. Use `batch.completed`/`batch.cancelled` where appropriate (e.g., in `BatchBanner.vue` label text).

### IN-04: Module-Level Mutable State in useBatch.ts

**File:** `src/composables/useBatch.ts:8-13`
**Issue:** The six `UnlistenFn | null` variables (`progressUnlisten`, `fileErrorUnlisten`, etc.) are declared at module scope, not within the `useBatch()` function. This creates a pseudo-singleton: if two components both call `useBatch()`, they share the same unlisten references. The current code avoids issues because `subscribe()` is only called once (in `MainLayout.vue`), and `unsubscribe()` is only called once (in the same component's `onUnmounted`). However, this pattern silently breaks if another component ever calls `useBatch()` and subscribes.

**Fix:** Move the unlisten variables inside `useBatch()` as `ref()` so each composable instance owns its own listeners, or convert the composable to a true singleton by guarding initialization.

### IN-05: `parse_time_to_seconds` Does Not Handle M:SS.mm Format

**File:** `src-tauri/src/ffmpeg/executor.rs:153-166`
**Issue:** The function parses FFmpeg time strings in HH:MM:SS.mm format, but FFmpeg may output "MM:SS.mm" for videos under one hour:
```rust
if time_str.contains(':') {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {   // Only handles HH:MM:SS.mm
        ...
    }
}
// Falls through to this for 2-part time strings:
time_str.parse().unwrap_or(0.0)  // Returns 0.0 for "01:30.5"
```
For short videos (<1 hour), the parsed time would be 0.0 seconds, causing the progress bar to stay at 0%.

**Fix:** Handle both 2-part and 3-part colon-separated formats:
```rust
fn parse_time_to_seconds(time_str: &str) -> f64 {
    if time_str.contains(':') {
        let parts: Vec<&str> = time_str.split(':').collect();
        match parts.len() {
            3 => {
                let h: f64 = parts[0].parse().unwrap_or(0.0);
                let m: f64 = parts[1].parse().unwrap_or(0.0);
                let s: f64 = parts[2].parse().unwrap_or(0.0);
                return h * 3600.0 + m * 60.0 + s;
            }
            2 => {
                let m: f64 = parts[0].parse().unwrap_or(0.0);
                let s: f64 = parts[1].parse().unwrap_or(0.0);
                return m * 60.0 + s;
            }
            _ => {}
        }
    }
    time_str.parse().unwrap_or(0.0)
}
```

### IN-06: Misleading `ffmpeg_path` Parameter Name in execute_single_file

**File:** `src-tauri/src/ffmpeg/executor.rs:40`
**Issue:** The function signature accepts `ffmpeg_path: &str` but the JSDoc (line 25) and usage at line 61 show it is a *directory* containing the ffmpeg binary, not a path to the binary itself:
```rust
let ffmpeg_bin = Path::new(ffmpeg_path).join(if cfg!(target_os = "windows") {
    "ffmpeg.exe"
} else {
    "ffmpeg"
});
```
This naming conflicts with the store key `ffmpeg_path` in `ffmpeg-config.json` (which also stores a directory path). The caller in `batch.rs` correctly reads the directory from the store, but the parameter name suggests a path to an executable rather than a directory.

**Fix:** Rename the parameter to `ffmpeg_dir` in both the function signature and the JSDoc comment:
```rust
pub fn execute_single_file(
    app: &AppHandle,
    entry: &VideoEntry,
    seed: &Seed,
    ffmpeg_dir: &str,  // Directory containing ffmpeg binary
    output_dir: &str,
    cancel_flag: &AtomicBool,
) -> Result<String, String> {
```

---

_Reviewed: 2026-05-14T20:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
