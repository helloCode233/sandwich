---
phase: 04-integration-polish
reviewed: 2026-05-14T00:00:00Z
depth: quick
files_reviewed: 14
files_reviewed_list:
  - src-tauri/src/commands/batch.rs
  - src-tauri/src/ffmpeg/executor.rs
  - src-tauri/src/models/batch.rs
  - src/components/batch/BatchBanner.vue
  - src/components/batch/BatchControls.vue
  - src/components/batch/BatchSummary.vue
  - src/components/MainLayout.vue
  - src/components/queue/ImportZone.vue
  - src/components/queue/QueueList.vue
  - src/composables/useBatch.ts
  - src/stores/batch.ts
  - src/types/batch.ts
  - src/locales/en.json
  - src/locales/zh-CN.json
findings:
  critical: 2
  warning: 1
  info: 0
  total: 3
status: issues_found
---

# Phase 04: Code Review Report

**Reviewed:** 2026-05-14
**Depth:** quick
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Quick-depth review of 14 files (3 Rust backend modules, 6 Vue components, 2 TypeScript composables/stores, 1 TypeScript types file, 2 locale JSON files). Pattern-matching scans for hardcoded secrets, dangerous functions (`eval`, `innerHTML`, `exec`, `system`), debug artifacts (`console.log`, `debugger`, `TODO`, `FIXME`), empty catch blocks, and commented-out code returned clean across all files. However, full file reads revealed two BLOCKER-level functional bugs and one WARNING-level quality issue related to concurrency implementation, batch-state recovery, and progress-parsing robustness.

## Critical Issues

### CR-01: Concurrency preference is read from config but never applied to processing

**File:** `src-tauri/src/commands/batch.rs:94`
**Issue:** The `start_batch` command reads the user's concurrency preference on line 94:
```rust
let _concurrency = get_concurrency_preference(&app);
```
The underscore prefix suppresses the compiler's unused-variable warning, confirming the value is intentionally discarded. The processing loop at line 150 iterates files one-at-a-time:
```rust
for entry in &queue_snapshot { ... }
```
No `tokio::spawn`, `Semaphore`, or any other concurrency mechanism is present. The concurrency selector in the UI (values 1-4, persisted to `sandwich-config.json`), the `get_concurrency_preference()` function with its range validation (`(1..=4).contains(&n)`), and the `onConcurrencyChange()` handler that persists the user's choice -- all of this infrastructure exists but is disconnected from the actual processing pipeline. Every batch runs with effective concurrency of 1 regardless of the user's setting. This contradicts design requirements D-08 and D-09.

**Fix:** Either implement concurrent processing using `tokio::spawn` with a `Semaphore` bounded by the concurrency value, or remove the concurrency UI and label it as a future feature. If implementing, the loop should look like:
```rust
use tokio::sync::Semaphore;

let concurrency = get_concurrency_preference(&app);
let semaphore = Arc::new(Semaphore::new(concurrency as usize));

for entry in &queue_snapshot {
    if cancel_flag.load(Ordering::SeqCst) { break; }
    let permit = semaphore.clone().acquire_owned().await
        .map_err(|_| "Semaphore closed".to_string())?;
    let app = app.clone();
    let entry = entry.clone();
    let seed = seed.clone();
    let ffmpeg_dir = ffmpeg_dir.clone();
    let output_dir = output_dir.clone();
    let cancel_flag = cancel_flag.clone();
    tokio::spawn(async move {
        let _permit = permit;
        execute_single_file(&app, &entry, &seed, &ffmpeg_dir, &output_dir, &cancel_flag)
    });
}
```

### CR-02: Batch start failure leaves Pinia store permanently stuck in `isProcessing = true`

**File:** `src/composables/useBatch.ts:62-68`
**Issue:** The `startBatch()` function calls `store.startProcessing(queueSize)` on line 62 *before* `invoke('start_batch', ...)` on line 63. If the backend rejects the batch start for any reason (queue empty, FFmpeg not configured, another batch already running), the `catch` block only logs the error and returns `false`:
```typescript
async function startBatch(seedId: string, outputDir: string, queueSize: number): Promise<boolean> {
  try {
    store.startProcessing(queueSize);  // Sets isProcessing = true FIRST
    await invoke('start_batch', { seedId, outputDir }); // May throw
    return true;
  } catch (err) {
    console.error('Failed to start batch:', err);
    return false; // Store IS NEVER RESET to idle
  }
}
```
Consequences:
- `isProcessing` stays `true` permanently (until page reload)
- The start button is replaced by the cancel button (gated on `isProcessing || cancelling`)
- The cancel button sends `cancel_batch` to the backend, which returns "No batch is currently running"
- `ImportZone` is hidden (gated on `!batchStore.isProcessing`), preventing the user from adding more files
- Queue remove/clear buttons are disabled (gated on `!batchStore.isProcessing`)
- The `startDisabled` computed prevents re-clicking even if conditions change
- The user has no recovery path short of a full page reload

**Fix:** Reset the store on failure:
```typescript
async function startBatch(seedId: string, outputDir: string, queueSize: number): Promise<boolean> {
  try {
    await invoke('start_batch', { seedId, outputDir });
    store.startProcessing(queueSize); // Only set after backend confirms start
    return true;
  } catch (err) {
    console.error('Failed to start batch:', err);
    store.resetBatch(); // Restore to idle so UI unlocks
    return false;
  }
}
```
Note: This also fixes the ordering -- the backend should confirm the batch can start before the frontend enters processing state.

## Warnings

### WR-01: Silent swallow of malformed FFmpeg progress time strings

**File:** `src-tauri/src/ffmpeg/executor.rs:159-174`
**Issue:** The `parse_time_to_seconds()` function uses `unwrap_or(0.0)` on all `str::parse::<f64>()` calls. If FFmpeg emits an unexpected time format (locale variations, edge-case timestamp representations, or future ffmpeg-sidecar changes), the progress percentage is silently clamped to 0.0 with no indication of a parse failure. Since `total_duration > 0.0` is the normal case (line 102), a parse failure returns `seconds = 0.0`, causing `percent = 0.0` throughout the entire encode. The per-file progress bar would show 0% until completion, confusing users who may think the process is stuck.

**Fix:** At minimum, log a warning and carry on with 0.0 rather than silently ignoring the failure:
```rust
let h: f64 = parts[0].parse().unwrap_or_else(|_| {
    eprintln!("WARN: failed to parse hours from time_str='{}'", time_str);
    0.0
});
```
Or, more robustly, use `Result` return types so the caller can decide how to handle the parse failure:
```rust
fn parse_time_to_seconds(time_str: &str) -> Option<f64> { ... }
```

---

_Reviewed: 2026-05-14_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: quick_
