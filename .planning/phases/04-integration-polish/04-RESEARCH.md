# Phase 4: Integration & Polish - Research

**Researched:** 2026-05-13
**Domain:** Tauri Event Streaming, FFmpeg Progress Parsing, Batch Cancel Flow, Vue UI State Wiring
**Confidence:** HIGH

## Summary

Phase 4 integrates the Vue 3 frontend (Phase 3) with the Rust FFmpeg executor (Phase 2), replacing placeholder UI scaffolding with live data from Tauri event streams. The primary technical work is: (1) resolving an event naming collision where both the Rust executor and batch command emit `batch-progress` with different payloads, (2) enriching the executor to emit a new `batch-file-progress` event carrying frame-level data (frame, fps, speed) extracted from ffmpeg-sidecar's `FfmpegProgress` struct, (3) extending the `useBatchStore` and `useBatch` composable with per-file progress tracking and a "cancelling" transitional state, (4) modifying four existing Vue components and creating one new `BatchSummary` component, and (5) adding 13 new i18n keys across both locales.

The cancel flow is structurally complete end-to-end (UI button -> composable -> `cancel_batch` command -> global `AtomicBool` -> executor/batch loop checks) but lacks a confirmation dialog and the transitional "Cancelling..." UI state. The rich per-file progress data required by BATCH-02 (frame, FPS, ETA) is available in the ffmpeg-sidecar `FfmpegProgress` struct but is not currently extracted by the executor -- only `time` is used to compute a percentage. The event naming collision between executor and batch command must be resolved by introducing the `batch-file-progress` event as defined in the UI-SPEC.

**Primary recommendation:** Add a `batch-file-progress` event from the Rust executor carrying frame/FPS/speed data; rename the executor's current `batch-progress` emission to this new event name; extend the frontend store/composable to consume it for per-file progress bars in QueueList.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Per-file progress streaming | API / Backend (Rust) | Browser / Client (Vue) | Rust executor parses FFmpeg stderr via ffmpeg-sidecar `iter()`, extracts `FfmpegProgress` fields, emits Tauri events. Vue listens and renders. |
| Aggregate batch progress | API / Backend (Rust) | — | Batch command loop computes totals from completed file results and emits `batch-progress` aggregate event. |
| Cancel signal propagation | API / Backend (Rust) | Browser / Client (Vue) | Global `AtomicBool` is the authority; UI sends `cancel_batch` command, Rust sets flag, executor checks in iteration loop. |
| Completion summary rendering | Browser / Client (Vue) | — | `BatchResult` is received via `batch-complete`/`batch-cancelled` events; Vue renders `BatchSummary` from store state. No backend involvement after event arrival. |
| Cancel confirmation dialog | Browser / Client (Vue) | — | Pure UI concern: Naive UI `dialog.warning()` before calling `cancelBatch()`. |
| i18n message formatting | Browser / Client (Vue) | — | vue-i18n handles localization; new keys defined in locale JSON files. No backend changes needed. |
| Cancelling transitional state | API / Backend (Rust) | Browser / Client (Vue) | Rust emits `batch-cancelling` event; Vue listens and shows frozen progress bar + "Cancelling..." label. |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| @tauri-apps/api | 2.11.0 | Tauri IPC bridge (`invoke`, `listen`, `emit`) | Official Tauri JS binding; `listen()` with typed generic for event subscription. |
| ffmpeg-sidecar | 2.5.1 | FFmpeg process management, progress parsing | Already in Cargo.toml. `FfmpegProgress` struct provides `frame`, `fps`, `speed`, `time` -- all needed for BATCH-02 per-file progress. |
| Naive UI | 2.44.1 | Component library (NProgress, NModal, NButton, NScrollbar, NTag) | Already in project. `dialog.warning()` for cancel confirmation, `NProgress` for per-file bars, `NScrollbar` for completion summary lists. |
| vue-i18n | 11.4.2 | Bilingual i18n (zh-CN, en) | Already in project. Phase 4 adds ~13 keys under `batch.*` namespace. |
| Pinia | 3.0.4 | State management (useBatchStore) | Already in project. Phase 4 extends the existing Composition API store with `perFileProgress` map and `cancelling` state. |
| lucide-vue-next | 1.0.0 | Icons (CheckCircle, AlertCircle, XCircle) | Already in project. Completion summary icons. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Tauri event streaming for per-file progress | Polling `get_batch_status` on interval | Polling adds latency and CPU overhead. Event streaming is the Tauri-recommended pattern and already wired for aggregate progress. |
| Inline BatchSummary panel | NModal overlay | Inline panel (UI-SPEC choice) keeps the dual-panel layout intact and avoids covering controls. Modal would obscure the queue during result review. |
| Single `batch-progress` event for both aggregate and per-file | Separate `batch-progress` + `batch-file-progress` events | Separate events (UI-SPEC choice) avoids payload shape ambiguity and type conflicts. Single event with a discriminant would require union types on both sides. |

**Installation:**
No new packages needed. All dependencies are already installed from Phase 1-3.

**Version verification:**
```
@tauri-apps/api@2.11.0 [VERIFIED: npm registry]
ffmpeg-sidecar@2.5.1 [VERIFIED: Cargo.toml]
naive-ui@2.44.1 [VERIFIED: package.json]
vue-i18n@11.4.2 [VERIFIED: package.json]
pinia@3.0.4 [VERIFIED: package.json]
lucide-vue-next@1.0.0 [VERIFIED: package.json]
vue@3.5.34 [VERIFIED: package.json]
```

## Architecture Patterns

### System Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                        VUE FRONTEND                              │
│                                                                  │
│  BatchControls.vue ──[click]──> useBatch.cancelBatch()           │
│       │                         │ invoke('cancel_batch')         │
│       │                         ▼                                │
│       │              useBatch.cancelBatch()                      │
│       │                         │                                │
│       ▼                         ▼                                │
│  NModal confirm          invoke('cancel_batch')                  │
│  dialog.warning()              │                                 │
│                                │                                 │
│  useBatchStore ◄── listen() ───┤── batch-progress (aggregate)   │
│    .progress                   │── batch-file-progress (per-file)│
│    .perFileProgress            │── batch-file-error (toast)      │
│    .cancelling                 │── batch-cancelling (transition) │
│    .lastResult                 │── batch-complete (final)        │
│    .isComplete                 │── batch-cancelled (final)       │
│       │                        │                                 │
│       ▼                        │                                 │
│  BatchBanner.vue               │                                 │
│  QueueList.vue (per-file bars) │                                 │
│  BatchSummary.vue (completion) │                                 │
└────────────────────────────────┼─────────────────────────────────┘
                                 │  Tauri IPC (JSON over WebSocket)
                                 ▼
┌──────────────────────────────────────────────────────────────────┐
│                        RUST BACKEND                              │
│                                                                  │
│  cancel_batch command                                            │
│       │                                                          │
│       ├── Sets BatchState.status = Cancelling                    │
│       ├── Sets global AtomicBool = true                          │
│       └── Emits "batch-cancelling" event                         │
│                                                                  │
│  start_batch command                                             │
│       │                                                          │
│       ├── For each entry in queue:                               │
│       │     ├── Check cancel_flag (SeqCst)                       │
│       │     ├── Update BatchState.progress.current_file          │
│       │     ├── Emit "batch-progress" (aggregate)                │
│       │     │                                                   │
│       │     └── execute_single_file() ─────────────┐             │
│       │                                             │             │
│       └── After loop: emit "batch-complete" or     │             │
│           "batch-cancelled" with BatchResult        │             │
│                                                     ▼             │
│  execute_single_file()                               │             │
│       │                                             │             │
│       ├── Spawn FFmpeg via ffmpeg-sidecar           │             │
│       ├── For each FfmpegEvent::Progress:           │             │
│       │     ├── Check cancel_flag (SeqCst)          │             │
│       │     ├── Extract: frame, fps, speed, time    │             │
│       │     ├── Compute: percent, remainingSeconds  │             │
│       │     └── Emit "batch-file-progress" ◄────────┘             │
│       │                                                           │
│       └── Return Ok(output_path) or Err(message)                  │
└──────────────────────────────────────────────────────────────────┘
```

### Event Flow (Complete Contract)

| Event Name | Emitted By | Payload Structure | Frontend Listener | Store Update |
|------------|-----------|-------------------|-------------------|--------------|
| `batch-progress` | `start_batch` loop (line 207) | `BatchProgress { total, completed, succeeded, failed, current_file }` | `useBatch.subscribe()` | `store.setProgress()` |
| `batch-file-progress` | `execute_single_file` loop | `PerFileProgress { file, percent, currentFrame, totalFrames, fps, remainingSeconds }` | NEW: `useBatch.subscribe()` | NEW: `store.setPerFileProgress()` |
| `batch-file-error` | `start_batch` loop (line 176) | `FileResult { file, seed, error }` | `useBatch.subscribe()` | (toast only, no store change) |
| `batch-cancelling` | `cancel_batch` command (line 267) | `()` (unit) | NEW: `useBatch.subscribe()` | NEW: `store.setCancelling(true)` |
| `batch-complete` | `start_batch` after loop (line 233) | `BatchResult { succeeded[], failed[] }` | `useBatch.subscribe()` | `store.stopProcessing()` |
| `batch-cancelled` | `start_batch` after loop (line 231) | `BatchResult { succeeded[], failed[] }` | `useBatch.subscribe()` | `store.stopProcessing()` |

### Critical Bug: Event Naming Collision

**What's broken:** The Rust `execute_single_file()` in `executor.rs` (lines 65-72, 130-137) emits `"batch-progress"` with payload `ExecutorProgress { file, stage, percent }`. The `start_batch` loop in `batch.rs` (line 207) also emits `"batch-progress"` with payload `BatchProgress { total, completed, succeeded, failed, current_file }`. These are different payload shapes under the same event name.

**Impact:** The frontend listener in `useBatch.ts` types the event as `BatchProgress`. When the executor's payload arrives, `store.setProgress()` does `progress.value = { ...p }`, spreading `{ file, stage, percent }` over the `BatchProgress` shape -- fields like `total`, `completed`, `succeeded`, `failed` become `undefined`.

**Fix:** Rename executor's emission to `"batch-file-progress"` with a new `PerFileProgress` Rust struct. This is exactly what the UI-SPEC prescribes.

### Recommended Project Structure
```
src/
├── types/
│   └── batch.ts                  # ADD: PerFileProgress interface
├── stores/
│   └── batch.ts                  # MODIFY: add perFileProgress map, cancelling state, setPerFileProgress, setCancelling
├── composables/
│   └── useBatch.ts               # MODIFY: add batch-file-progress + batch-cancelling listeners
├── components/
│   ├── batch/
│   │   ├── BatchBanner.vue       # MODIFY: cancelling/completion variants, per-file label
│   │   ├── BatchControls.vue     # MODIFY: cancel confirmation dialog
│   │   └── BatchSummary.vue      # NEW: completion summary panel
│   ├── queue/
│   │   └── QueueList.vue         # MODIFY: per-file progress bars, disable controls during processing
│   └── MainLayout.vue            # MODIFY: conditional BatchBanner/BatchSummary rendering
├── locales/
│   ├── en.json                   # MODIFY: add ~13 batch.* keys
│   └── zh-CN.json                # MODIFY: add ~13 batch.* keys
src-tauri/src/
├── ffmpeg/
│   └── executor.rs               # MODIFY: emit batch-file-progress with PerFileProgress, extract fps/speed/frame from FfmpegProgress
├── models/
│   └── batch.rs                  # MODIFY: add PerFileProgress Rust struct
└── commands/
    └── batch.rs                  # MODIFY: (optional) minor adjustments if needed
```

### Pattern 1: Tauri Event Streaming (Progress)
**What:** Rust emittor -> Tauri event bus -> Vue listener -> Pinia store update -> reactive component render
**When to use:** Any long-running backend operation that needs real-time UI updates
**Key constraint:** Event payloads must be `Serialize` on Rust side and match the TypeScript interface exactly (field names transformed by `#[serde(rename_all = "camelCase")]`).
**Example:**
```rust
// Rust side (executor.rs — NEW pattern)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerFileProgress {
    pub file: String,
    pub percent: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub fps: f32,
    pub remaining_seconds: f64,
}

app.emit("batch-file-progress", PerFileProgress { ... })?;
```
```typescript
// Vue side (useBatch.ts — NEW listener)
const fileProgressUnlisten = await listen<PerFileProgress>(
  'batch-file-progress',
  (event) => { store.setPerFileProgress(event.payload); }
);
```

### Pattern 2: Cancel Flag Propagation
**What:** `AtomicBool` (global static) -> checked in both batch loop and executor iteration -> FFmpeg process killed -> partial output cleaned -> `batch-cancelled` event emitted
**When to use:** Any cancelable long-running operation that spans multiple files and involves external process management
**Key insight:** The cancel flag uses `Ordering::SeqCst` for ARM visibility (Pitfall 5 documented in executor.rs line 112). The flag is stored in a `OnceLock<TokioMutex<Option<Arc<AtomicBool>>>>` so it's accessible from both `start_batch` (producer) and `cancel_batch` (consumer) without passing through managed state.
**Example:** See existing `batch.rs` lines 21-25, 96-101, 147, 259-264. No change needed in Phase 4 -- only the UI confirmation dialog is added.

### Pattern 3: Store-Driven Conditional Rendering
**What:** Pinia store refs (`isProcessing`, `isComplete`, `cancelling`) drive `v-if` in layout components
**When to use:** Any UI state transition triggered by async backend events
**Example:**
```vue
<!-- MainLayout.vue — MODIFIED pattern -->
<BatchBanner v-if="batchStore.isProcessing || batchStore.cancelling || batchStore.isComplete" />
<BatchSummary v-if="batchStore.isComplete && batchStore.lastResult" />
<QueueList />
```

### Anti-Patterns to Avoid
- **Payload type mismatch:** Never emit different payload shapes under the same event name. The current executor/command collision on `batch-progress` is exactly this anti-pattern. Use distinct event names for distinct payloads.
- **Polling for progress:** Do not use `setInterval` with `get_batch_status` for per-file progress updates. Event streaming is the Tauri-recommended pattern and provides sub-second latency without CPU overhead.
- **Storing FFmpeg progress in managed state:** Do not put `PerFileProgress` into the `Mutex<AppState>`. The executor runs outside the Mutex lock (Pitfall 3 in batch.rs line 123: "drop locks before FFmpeg spawn"). Emit events directly via `AppHandle` instead.
- **Computing ETA from file size:** Do not estimate remaining time from `size_kb / total_size`. Use FFmpeg's `speed` field (already available in `FfmpegProgress`) -- `remaining_seconds = (total_duration - current_seconds) / max(speed, 0.01)`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FFmpeg progress parsing | Custom regex on stderr | `ffmpeg-sidecar`'s `child.iter()` returning `FfmpegEvent::Progress` | Already integrated. Edge cases: multi-line progress output, carriage-return updates, locale-dependent number formatting -- all handled by ffmpeg-sidecar's parser. |
| Frame count calculation | Counting `OutputFrame` events | Compute `totalFrames = metadata.duration_secs * metadata.fps` from ffprobe data | VideoEntry.metadata already has `duration_secs` and `fps` from ffprobe. Counting frames would require enabling rawvideo output. |
| ETA computation | Custom time-series prediction | `(total_duration - current_time) / max(ffmpeg_speed, 0.01)` | FFmpeg's `speed` field already encodes the processing rate. Simple arithmetic is sufficient; complex prediction adds no accuracy for batch video encoding. |
| Cancel confirmation dialog | Custom modal component | Naive UI `dialog.warning()` | Already used in QueueList for clear-all confirmation. Consistent look, keyboard accessible, i18n-friendly. |
| Event bus | Custom pub/sub | Tauri `app.emit()` + `listen()` | Built into Tauri 2.x. Type-safe via Rust `Serialize` + TS generics. No additional dependency. |

**Key insight:** The only "new code" in Phase 4 is wiring existing capabilities together. The Rust executor already has the data (ffmpeg-sidecar provides it); the Vue components already have the UI structure (Naive UI provides it). The work is extracting the right fields, choosing the right event names, and connecting the right store refs to the right `v-if` conditions.

## Common Pitfalls

### Pitfall 1: Event Payload Type Mismatch
**What goes wrong:** The executor emits `{ file, stage, percent }` under `"batch-progress"` but the frontend listener destructures it as `BatchProgress` (expecting `total`, `completed`, etc.). Undefined fields propagate through the store and break computed properties.
**Why it happens:** Same event name reused for different payloads. The executor was written to emit per-file status, the batch loop was written to emit aggregate status, and both used the same event name.
**How to avoid:** Create a new `"batch-file-progress"` event with a dedicated `PerFileProgress` Rust struct. Update the executor to emit this event. Update the frontend to listen for it separately.
**Warning signs:** `batchStore.progress.total` showing `NaN`, `overallPercent` flickering, TypeScript no longer catching shape errors because `listen<T>` uses type assertion not runtime validation.

### Pitfall 2: Missing Cancelling Transitional State
**What goes wrong:** User clicks Cancel, the button becomes unresponsive for 1-3 seconds (while FFmpeg process terminates), then the UI jumps directly to the completion summary. No feedback during the wind-down period.
**Why it happens:** The `batch-cancelling` event IS emitted by Rust (`batch.rs` line 267) but no frontend listener exists for it. The store has no `cancelling` state ref.
**How to avoid:** Add `const cancelling = ref(false)` to `useBatchStore`. Add `listen('batch-cancelling', ...)` to `useBatch.subscribe()`. Show "Cancelling..." label + frozen progress bar in `BatchBanner.vue` when `cancelling` is true.
**Warning signs:** UI freeze during cancel; user clicks cancel multiple times; "still processing" in console after cancel returns.

### Pitfall 3: Per-File Progress Bar Lag
**What goes wrong:** The per-file NProgress bar jumps from 0% to 80% then stalls. The frame counter and ETA flicker between updates.
**Why it happens:** FFmpeg's progress output is irregular -- it emits every ~1 second but encoding speed varies by frame complexity. The `percent` computed from `time/total_duration` can be non-linear (FFmpeg may report a time that's ahead of actual encoded frames).
**How to avoid:** Use `progress.frame` (monotonically increasing frame count) for the progress bar, not `percent` from time. Clamp `remainingSeconds` to non-negative. Rate-limit UI updates with `requestAnimationFrame` if the frontend feels janky (unlikely with ffmpeg-sidecar's ~1Hz output).
**Warning signs:** Progress bar goes backwards; ETA shows negative numbers; bar shows 100% while FFmpeg is still running.

### Pitfall 4: BatchSummary Overflow
**What goes wrong:** A batch of 100+ files generates a completion summary that overflows the viewport, hiding the "Clear Results" button.
**Why it happens:** The BatchSummary renders a non-scrollable list of all succeeded + failed files. Without height constraints, it pushes all other content off-screen.
**How to avoid:** Per UI-SPEC: max height 200px per section (succeeded/failed lists), use `NScrollbar` for overflow. The summary panel itself should be `max-height: 60vh` with `overflow-y: auto` as a safety net.
**Warning signs:** Summary content clips at bottom; scroll on body instead of within panel; cannot reach dismiss button.

### Pitfall 5: QueueList Interaction During Processing
**What goes wrong:** User removes a video from the queue while it's being processed (or about to be processed). The Rust batch loop is iterating a snapshot of the queue, but the frontend queue store and the Rust managed state can diverge.
**Why it happens:** The batch `start_batch` command clones the queue (line 113: `let queue = app_state.queue.clone()`) before processing. But the frontend queue store can still be modified by the user.
**How to avoid:** Disable all queue mutation controls during processing: remove buttons, clear-all button, import zone, add-video button. Bind `:disabled="batchStore.isProcessing"` on all interactive elements in QueueList.vue and ImportZone.vue.
**Warning signs:** "Index out of bounds" error when removing during processing; video removed from UI but still being processed; processed file count doesn't match visible queue.

## Code Examples

Verified patterns from official sources:

### Emitting Per-File Progress from Rust Executor
```rust
// Source: ffmpeg-sidecar 2.5.1 event.rs (FfmpegProgress struct — VERIFIED in cargo registry)
// Current executor.rs only extracts `time`. Extend to extract frame, fps, speed:

use crate::models::batch::PerFileProgress;

for event in child.iter()? {
    if cancel_flag.load(Ordering::SeqCst) { /* ... kill logic ... */ }
    match event {
        FfmpegEvent::Progress(progress) => {
            let seconds = parse_time_to_seconds(&progress.time);
            let percent = if total_duration > 0.0 {
                (seconds / total_duration * 100.0).clamp(0.0, 100.0)
            } else { 0.0 };
            let remaining = if progress.speed > 0.01 {
                (total_duration - seconds) / progress.speed as f64
            } else { 0.0 };
            let total_frames = (total_duration * entry.metadata.fps as f64) as u32;

            let _ = app.emit("batch-file-progress", PerFileProgress {
                file: filename.clone(),
                percent,
                current_frame: progress.frame,
                total_frames,
                fps: progress.fps,
                remaining_seconds: remaining.max(0.0),
            });
        }
        // ... existing Log handling ...
        _ => {}
    }
}
```

### Store Extension: Per-File Progress Map
```typescript
// Source: existing useBatchStore pattern (VERIFIED in src/stores/batch.ts)
// ADD these to the existing store:

import type { PerFileProgress } from '@/types/batch';

const perFileProgress = ref<Map<string, PerFileProgress>>(new Map());
const cancelling = ref(false);

const currentFileProgress = computed(() => {
  if (!progress.value.currentFile) return null;
  return perFileProgress.value.get(progress.value.currentFile) ?? null;
});

function setPerFileProgress(p: PerFileProgress) {
  perFileProgress.value.set(p.file, { ...p });
  // Trigger reactivity: Map.set doesn't trigger by itself
  perFileProgress.value = new Map(perFileProgress.value);
}

function setCancelling(value: boolean) {
  cancelling.value = value;
}
```

### Cancel Confirmation Dialog in BatchControls
```vue
<!-- Source: existing dialog.warning() pattern in QueueList.vue (VERIFIED) -->
<script setup lang="ts">
import { useDialog } from 'naive-ui';
const dialog = useDialog();

async function onCancel() {
  dialog.warning({
    title: t('batch.cancelConfirmTitle'),
    content: t('batch.cancelConfirmBody'),
    positiveText: t('batch.cancel'),
    negativeText: t('batch.keepProcessing'),
    onPositiveClick: async () => {
      const ok = await cancelBatch();
      if (!ok) {
        message.error(t('notification.operationFailed', { error: 'Cancel failed' }));
      }
    },
  });
}
</script>
```

### Per-File Progress Bar in QueueList
```vue
<!-- Source: UI-SPEC data contract (section "Per-File Progress Bar Contract") -->
<div v-if="isCurrentFile" class="mt-2">
  <NProgress
    type="line"
    :percentage="fileProgress.percent"
    indicator-placement="inside"
    :height="18"
    :color="fileProgress.percent === 100 ? '#18a058' : '#2080f0'"
  />
  <div class="flex justify-between mt-0.5">
    <NText depth="3" class="text-xs">
      {{ t('batch.fileProgress', { current: fileProgress.currentFrame, total: fileProgress.totalFrames }) }}
    </NText>
    <NText depth="3" class="text-xs">
      {{ t('batch.fileEta', { minutes: Math.ceil(fileProgress.remainingSeconds / 60) }) }}
    </NText>
  </div>
</div>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Executor emits `batch-progress` with `ExecutorProgress` | Executor emits `batch-file-progress` with `PerFileProgress` | Phase 4 | Resolves naming collision; enables per-file frame/FPS/ETA display |
| Single `batch-progress` for aggregate only | Separate `batch-progress` (aggregate) + `batch-file-progress` (per-file) | Phase 4 | Clean separation of concerns; type-safe on both sides |
| BatchControls cancel without confirmation | NModal dialog.warning() confirmation | Phase 4 | Prevents accidental cancel; matches clear-all pattern |
| BatchBanner shows during processing only | Shows during processing, cancelling, AND complete (with appropriate variants) | Phase 4 | Consistent position for processing status across all states |

**Deprecated/outdated:**
- `ExecutorProgress` Rust struct in `executor.rs`: Replaced by `PerFileProgress`. The `stage: String` field ("starting", "processing") is not used by the frontend and has no UI representation.
- Executor `batch-progress` emission at line 65-72 (executor.rs): The "starting" emission provides no actionable data. Replace with a single `batch-file-progress` on first progress event.
- `batch.cancelConfirm` i18n key (single string): Split into `batch.cancelConfirmTitle` + `batch.cancelConfirmBody` per UI-SPEC dialog format.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| BATCH-02 | 处理时显示逐文件进度（百分比、当前帧、预估剩余时间） | Per-file progress data available from ffmpeg-sidecar `FfmpegProgress { frame, fps, speed, time }`. Requires new `batch-file-progress` event (not yet emitted). Total frames computable from `VideoEntry.metadata.duration_secs * fps`. ETA computable from `(total_duration - current_time) / speed`. Frontend: new `PerFileProgress` TS type + `perFileProgress` store map + per-file NProgress bars in QueueList.vue. |
| BATCH-05 | 批处理完成后展示结果摘要（成功/失败数） | BatchResult already emitted via `batch-complete` / `batch-cancelled` events and stored in `useBatchStore.lastResult`. New `BatchSummary.vue` component reads from store and renders inline panel with succeeded/failed lists. Per-file output paths available in `BatchResult.succeeded: string[]`. Error messages available in `BatchResult.failed: FileResult[]`. |

## i18n Keys Inventory

### Existing Keys (No Change)
| Key | EN Value | Status |
|-----|----------|--------|
| `batch.title` | "Processing Controls" | Unchanged |
| `batch.selectSeed` | "Select Seed" | Unchanged |
| `batch.concurrency` | "Concurrency" | Unchanged |
| `batch.outputDir` | "Output Directory" | Unchanged |
| `batch.changeDir` | "Change Directory" | Unchanged |
| `batch.start` | "Start Processing" | Unchanged |
| `batch.cancel` | "Cancel Processing" | Unchanged |
| `batch.noSeedSelected` | "Please select a seed first" | Unchanged |
| `batch.queueEmpty` | "Queue is empty. Import videos first" | Unchanged |
| `batch.alreadyRunning` | "A batch is already in progress" | Unchanged |
| `batch.processing` | "Processing" | Unchanged |
| `batch.progress` | "{completed}/{total}" | Unchanged |
| `batch.defaultOutputDir` | "~/Videos/sandwich-output/" | Unchanged |

### Keys to Modify
| Key | Current EN | New EN | Reason |
|-----|-----------|--------|--------|
| `batch.cancelConfirm` | "Cancel batch processing? Completed files will be preserved." | Split into two keys | See below |

### New Keys Required (per UI-SPEC copywriting contract)
| Key | EN Value | ZH Value | Used In |
|-----|----------|----------|---------|
| `batch.cancelConfirmTitle` | "Cancel Batch Processing?" | "取消批处理？" | BatchControls NModal dialog |
| `batch.cancelConfirmBody` | "Completed files will be preserved. In-progress files will be discarded." | "已完成处理的文件将保留，正在处理的文件将被丢弃。" | BatchControls NModal dialog |
| `batch.keepProcessing` | "Keep Processing" | "继续处理" | BatchControls NModal negative button |
| `batch.cancelling` | "Cancelling..." | "取消中..." | BatchBanner cancelling state label |
| `batch.processingFile` | "Processing: {filename}" | "正在处理：{filename}" | BatchBanner per-file label |
| `batch.fileProgress` | "Frame {current} / {total}" | "帧 {current} / {total}" | QueueList per-file bar frame counter |
| `batch.fileEta` | "~{minutes} min remaining" | "预计剩余 {minutes} 分钟" | QueueList per-file bar ETA |
| `batch.fileFailed` | "{filename} failed: {error}" | "{filename} 处理失败：{error}" | useBatch toast on batch-file-error |
| `batch.noOutputDir` | "Please select an output directory first" | "请先选择输出目录" | BatchControls start validation |
| `batch.summary.completeTitle` | "Batch Complete" | "批处理完成" | BatchSummary title |
| `batch.summary.cancelledTitle` | "Batch Cancelled" | "批处理已取消" | BatchSummary title |
| `batch.summary.succeededSection` | "Succeeded ({count})" | "成功（{count}）" | BatchSummary section heading |
| `batch.summary.failedSection` | "Failed ({count})" | "失败（{count}）" | BatchSummary section heading |
| `batch.summary.outputPath` | "{filename} -> {outputPath}" | "{filename} -> {outputPath}" | BatchSummary succeeded file row |
| `batch.summary.fileError` | "{filename}: {errorMessage}" | "{filename}：{errorMessage}" | BatchSummary failed file row |
| `batch.summary.clearResults` | "Clear Results" | "清除结果" | BatchSummary dismiss button |
| `batch.summary.completeBody` | "{succeeded} succeeded, {failed} failed out of {total} files" | "共 {total} 个文件，成功 {succeeded} 个，失败 {failed} 个" | BatchSummary summary line |
| `batch.summary.cancelledBody` | "Cancelled after {succeeded} succeeded, {failed} failed out of {total} files" | "已取消，共 {total} 个文件，成功 {succeeded} 个，失败 {failed} 个" | BatchSummary summary line |

**Total: 18 new keys, 1 key to deprecate** (`batch.cancelConfirm` replaced by `batch.cancelConfirmTitle` + `batch.cancelConfirmBody`).

## Environment Availability

Step 2.6: SKIPPED (no external dependencies identified). Phase 4 is a pure code integration phase -- all dependencies (Tauri, ffmpeg-sidecar, Naive UI, vue-i18n) are already installed from Phases 1-3. No new tools, services, or CLIs are required.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `VideoEntry.metadata.fps` from ffprobe accurately reflects the total frame count when multiplied by duration | Standard Stack | If fps is variable frame rate (VFR), totalFrames computation will be approximate. Impact: per-file frame counter shows slightly inaccurate max value. Not critical -- percent and ETA are more important metrics for users. |
| A2 | FFmpeg's `speed` field in `FfmpegProgress` accurately represents encoding speed ratio | Code Examples | On very short files (< 3 seconds), speed may spike unrealistically causing ETA to show 0 minutes. Mitigation: clamp `speed` minimum to 0.01 in ETA formula. |
| A3 | QueueList per-file progress bar does not need to handle concurrent processing (concurrency > 1) | Architecture Patterns | Current Rust `start_batch` processes files sequentially (for-each loop, not parallel). If concurrency > 1 is implemented later, `currentFile` in `BatchProgress` would need to become `currentFiles: Vec<String>` and the per-file bar logic would need to handle multiple active files. |
| A4 | The `batch.cancelConfirm` key has no other consumers beyond BatchControls | i18n Keys Inventory | If other components reference `batch.cancelConfirm`, they would break when the key is split. [ASSUMED] based on grep of codebase -- only BatchControls.vue references it. |
| A5 | `NModal` `dialog.warning()` supports `positiveText` and `negativeText` as documented in Naive UI 2.44.x | Code Examples | If the API changed, fallback is to use `NModal` directly with custom slot content. [ASSUMED] based on QueueList.vue existing `dialog.warning()` usage. |

## Open Questions

1. **Should `batch.cancelConfirm` be removed or kept for backward compat?**
   - What we know: Only used in BatchControls.vue. The UI-SPEC prescribes splitting it into title + body.
   - What's unclear: Whether other phases added references that grep didn't find (e.g., dynamic key construction).
   - Recommendation: Replace with two keys, remove old key. If any component references it, it will fail at build time (TypeScript catches missing i18n keys at compile time).

2. **Should the `batch.completed` / `batch.cancelled` existing keys be deprecated in favor of `batch.summary.completeTitle` / `batch.summary.cancelledTitle`?**
   - What we know: `batch.completed` and `batch.cancelled` exist in `en.json` (lines 82-83) but are not referenced in any Vue component (verified by grep). The UI-SPEC defines new keys under `batch.summary.*`.
   - What's unclear: Whether Phase 3 planned to use these keys but never did.
   - Recommendation: Add the new `batch.summary.*` keys. Keep the old `batch.completed` / `batch.cancelled` keys in the JSON files but do not reference them. Remove in a later cleanup phase.

3. **Should the executor's "starting" stage emission be removed?**
   - What we know: The executor emits `batch-progress` with `stage: "starting", percent: 0.0` at line 65-72 (executor.rs). The frontend does nothing with `stage`. After Phase 4, the executor will emit `batch-file-progress` instead.
   - Recommendation: Remove the starting emission. The first `batch-file-progress` event (from the first `FfmpegEvent::Progress`) provides more useful data (actual percent, frame count) than a synthetic "starting" event.

4. **How should QueueList determine which file is the "current file"?**
   - What we know: `batchStore.progress.currentFile` is set by the Rust batch loop. `batchStore.perFileProgress` is keyed by filename.
   - What's unclear: Whether `currentFile` is always set BEFORE the first `batch-file-progress` event for that file arrives (race condition between aggregate and per-file events).
   - Recommendation: The batch loop sets `currentFile` before calling `execute_single_file()`. The executor's first progress event arrives after FFmpeg spawns. This ordering (set name -> spawn -> first event) guarantees `currentFile` is set before the first per-file event. No race condition.

## Security Domain

> `security_enforcement` is not explicitly set to `false` in `.planning/config.json`, so the default (enabled) applies. However, this phase adds no new attack surface -- it wires existing IPC events, adds UI components, and extends i18n keys. All IPC commands (`start_batch`, `cancel_batch`, `get_batch_status`) were already registered in Phase 2 and are not modified.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | No auth layer -- desktop app |
| V3 Session Management | No | No sessions -- single-user desktop |
| V4 Access Control | No | No multi-user access control |
| V5 Input Validation | No (no new inputs) | Existing Phase 2 validation on `start_batch` params (seed_id, output_dir) unchanged |
| V6 Cryptography | No | No crypto operations |

### Known Threat Patterns for Tauri Event Streaming

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Event listener leak (unsubscribed listeners accumulating) | Denial of Service | `useBatch.unsubscribe()` already calls `unlistenFn?.()` for all 4 listeners. Phase 4 adds 2 more listeners -- verify they are added to the unsubscribe array. |
| Large event payloads causing UI jank | Denial of Service | `PerFileProgress` is fixed-size (6 primitive fields). `BatchResult.succeeded[]` could be large for 100+ files but is emitted once at completion. Client-side mitigation: virtual scrolling in `BatchSummary` via `NScrollbar` (per UI-SPEC max 200px per section). |

## Sources

### Primary (HIGH confidence)
- [ffmpeg-sidecar 2.5.1 event.rs — VERIFIED from cargo registry] `/Users/ghost/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ffmpeg-sidecar-2.5.1/src/event.rs` — `FfmpegProgress` struct fields: `frame: u32`, `fps: f32`, `speed: f32`, `time: String`, `size_kb: u32`, `q: f32`, `bitrate_kbps: f32`
- [src-tauri/src/ffmpeg/executor.rs] — Current executor implementation: `ExecutorProgress` struct, `execute_single_file()` function, event emission at lines 65-72 and 130-137
- [src-tauri/src/commands/batch.rs] — Batch command implementation: `start_batch`, `cancel_batch`, `get_batch_status`, cancel flag pattern, event emissions
- [src-tauri/src/models/batch.rs] — Rust model types: `BatchProgress`, `BatchResult`, `FileResult`, `BatchConfig`
- [src/types/batch.ts] — TypeScript types: `BatchProgress`, `BatchResult`, `FileResult`
- [src/stores/batch.ts] — `useBatchStore`: all existing refs, computed properties, and actions
- [src/composables/useBatch.ts] — `useBatch`: event subscriptions, `startBatch`, `cancelBatch`, `getBatchStatus`
- [src/components/batch/BatchControls.vue] — Current batch controls with start/cancel buttons, no confirmation dialog
- [src/components/batch/BatchBanner.vue] — Current batch banner with NProgress, no cancelling/completion variants
- [src/components/queue/QueueList.vue] — Current queue list, no per-file progress bars, no processing locks
- [src/components/MainLayout.vue] — Current layout, conditional BatchBanner only for `isProcessing`
- [src/locales/en.json] and [src/locales/zh-CN.json] — All existing i18n keys, verified `batch.*` namespace
- [UI-SPEC: .planning/phases/04-integration-polish/04-UI-SPEC.md] — Complete data contract, component inventory, interaction states, copywriting, color, spacing, typography
- [package.json] — Verified all version numbers: vue@3.5.34, naive-ui@2.44.1, pinia@3.0.4, @tauri-apps/api@2.11.0, vue-i18n@11.4.2
- [Cargo.toml] — Verified: ffmpeg-sidecar@2.5.1

### Secondary (MEDIUM confidence)
- [Tauri 2.x event system] — CLUADE.md Context7 citations confirm `app.emit()` and `listen()` APIs are the standard Tauri IPC pattern. Pattern confirmed by existing codebase usage of `batch-progress`, `batch-file-error`, `batch-complete`, `batch-cancelled` events.

### Tertiary (LOW confidence)
- None. All claims are verified against source code or official crate source.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All libraries already installed and verified in package.json/Cargo.toml. No new dependencies needed.
- Architecture: HIGH — Event flow, cancel flow, and store structure verified against actual source code in 16 files.
- Pitfalls: HIGH — Event naming collision confirmed by source code inspection (executor.rs line 65 vs batch.rs line 207). Missing cancelling listener confirmed by grepping useBatch.ts for 'batch-cancelling'.
- i18n: HIGH — All existing keys read from locale JSON files. New keys derived from UI-SPEC copywriting contract.

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (30 days — stable tech, no expected breaking changes in Tauri or ffmpeg-sidecar)
