---
phase: 03-vue-frontend
reviewed: 2026-05-13T00:00:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - vitest.config.ts
  - eslint.config.mjs
  - src/types/seed.ts
  - src/types/video.ts
  - src/types/batch.ts
  - src/locales/en.json
  - src/locales/zh-CN.json
  - src/stores/seed.ts
  - src/stores/queue.ts
  - src/stores/batch.ts
  - src/stores/__tests__/seed.test.ts
  - src/stores/__tests__/queue.test.ts
  - src/stores/__tests__/batch.test.ts
  - src/composables/useSeed.ts
  - src/composables/useQueue.ts
  - src/composables/useBatch.ts
  - src/App.vue
  - src/components/MainLayout.vue
  - src/components/seed/SeedCard.vue
  - src/components/seed/SeedList.vue
  - src/components/queue/ImportZone.vue
  - src/components/queue/QueueList.vue
  - src/components/batch/BatchBanner.vue
  - src/components/batch/BatchControls.vue
findings:
  critical: 0
  warning: 5
  info: 7
  total: 12
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-05-13T00:00:00Z
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

Reviewed all 22 source files in the Vue frontend phase. The codebase is well-structured: TypeScript types match Rust IPC contracts (`camelCase` field names), Pinia stores use Composition API patterns, composables wrap Tauri `invoke()` calls cleanly, Vue components implement the dual-panel layout with dark theme, and unit tests cover all store functions.

No critical (BLOCKER) issues were found. Five warnings and seven informational items were identified, primarily around error handling gaps, a potential `~` path resolution mismatch with Rust, composable state leakage risk, and minor UX/data-loss concerns.

## Warnings

### WR-01: Default output directory with `~` may not be resolved by Rust backend

**File:** `src/components/batch/BatchControls.vue:50,110-112`
**Issue:** The default output directory is `~/Videos/sandwich-output/` (from i18n key `batch.defaultOutputDir`). The code comment on line 111 claims "Rust resolves ~ automatically", but Rust's `std::path::PathBuf` and `std::fs` do NOT perform tilde expansion. Only shell expanders (`shellexpand` crate, `dirs::home_dir()`) handle this. If the Rust backend does not explicitly resolve `~` to the user's home directory before calling `std::fs::create_dir_all` or passing the path to FFmpeg, the batch processing will attempt to write to a literal directory named `~` in the current working directory. This will either fail with a permission error (if the CWD is read-only) or create files in a confusing location.

**Fix:** Either resolve `~` on the frontend side before sending to Rust, or confirm that the Rust backend uses `shellexpand::full()` or `dirs::home_dir()` to expand the path. On the frontend, a pragmatic fix:

```typescript
// Before passing to startBatch:
let resolvedDir = outputDir.value;
if (resolvedDir.startsWith('~/')) {
  // Use Tauri's path API or a home-dir resolution
  // Workaround: defer to Rust, but ensure Rust explicitly expands ~
  // Best: resolve here using @tauri-apps/api/path if available
}
```

### WR-02: Error swallowing in composable `loadSeeds()` and `loadQueue()` leave UI in unknown state

**File:** `src/composables/useSeed.ts:16-18`, `src/composables/useQueue.ts:16-18`
**Issue:** Both `loadSeeds()` and `loadQueue()` catch errors silently with `console.error` but return `void`. Callers (the `subscribe()` functions and event listeners) cannot detect failure. If the IPC call fails (e.g., Rust backend crash, network issue with remote state, or serialization error), the stores retain their previous state (or remain empty), and the user sees no indication of failure.

**Fix:** Return a boolean or throw a structured error so callers can surface a user-visible error.

```typescript
async function loadSeeds(): Promise<boolean> {
  try {
    const list = await invoke<Seed[]>('list_seeds');
    store.setSeeds(list);
    return true;
  } catch (err) {
    console.error('Failed to load seeds:', err);
    return false;
  }
}
```

Then in the caller (`subscribe`):
```typescript
unlisten = await listen('seeds-updated', async () => {
  const ok = await loadSeeds();
  if (!ok) {
    // Show toast or fallback UI
  }
});
```

### WR-03: Composable module-level state creates listener leak risk if called multiple times

**File:** `src/composables/useSeed.ts:6`, `src/composables/useQueue.ts:6-7`, `src/composables/useBatch.ts:6-9`
**Issue:** Unlisten functions (`unlisten`, `queueUpdatedUnlisten`, `videoImportedUnlisten`, `progressUnlisten`, `fileErrorUnlisten`, `completeUnlisten`, `cancelledUnlisten`) are stored in module-level variables. If the composable function is ever called from more than one component instance (e.g., if the app later adds a secondary panel or a diagnostic view), the second call to `subscribe()` will overwrite the module-level variables, leaking the first set of listeners. The `unsubscribe()` call from the first component will then be a no-op (it calls `null?.()` after the second component overwrites the variable), and the first component's listeners remain active after its unmount.

Currently safe because each composable is instantiated exactly once in `MainLayout.vue`, but this fragility should be documented or fixed.

**Fix:** Move the unlisten variables inside the `useSeed()` / `useQueue()` / `useBatch()` function scope so each caller gets its own state:

```typescript
export function useSeed() {
  const store = useSeedStore();
  let unlisten: UnlistenFn | null = null;
  // ... rest of implementation
}
```

### WR-04: Rust error details lost in import failure path

**File:** `src/components/queue/ImportZone.vue:84-91`
**Issue:** When `importVideo(filepath)` fails, the composable's `importVideo` function logs the error to `console.error` and returns `null`. The ImportZone component then shows a generic toast: "Operation failed: Import failed". Specific error information from the Rust backend (e.g., "Unsupported format .xyz", "File not found", "No video stream detected") is only available in the console log, not surfaced to the user. Users clicking "Add Videos" with mixed valid/invalid files will see identical failure messages with no hint about which file failed or why.

**Fix:** Return richer error information from the composable, or emit structured errors. Minimal fix — capture the error message:

```typescript
// In useQueue composable:
async function importVideo(filepath: string): Promise<{ entry: VideoEntry } | { error: string }> {
  try {
    const entry = await invoke<VideoEntry>('import_video', { filepath });
    return { entry };
  } catch (err) {
    console.error('Failed to import video:', err);
    return { error: String(err) };
  }
}
```

Then in ImportZone:
```typescript
const result = await importVideo(filepath);
if ('entry' in result) {
  message.success(t('queue.imported', { filename: result.entry.filename }));
} else {
  message.error(t('import.error', { error: result.error }));
}
```

### WR-05: QueueList `removeFromQueue` uses index-based removal, which can target wrong entry after list mutation

**File:** `src/components/queue/QueueList.vue:59-67`, `src/stores/queue.ts:23-27`
**Issue:** `removeFromQueue` takes a numeric index and calls `invoke('remove_from_queue', { index })`. If the queue list changes between rendering and the user clicking remove (e.g., another video is imported via the `video-imported` event), the index in the rendered DOM may no longer correspond to the same entry. The Rust backend also uses the index, so it would remove the wrong entry from the Rust-side queue, and the frontend store would similarly remove the wrong entry.

This is a race condition that is unlikely in normal single-user usage but could occur during rapid import operations concurrent with user removal clicks.

**Fix:** Use a stable identifier (e.g., `filepath`) instead of index for removal, or disable the remove button when the queue is being modified. Longer-term, switching to ID-based removal (`remove_from_queue_by_path` or a UUID per entry) would eliminate this class of bug.

## Info

### IN-01: Rename input cancels on blur, silently discarding user input

**File:** `src/components/seed/SeedCard.vue:110`
**Issue:** The rename `<NInput>` has `@blur="cancelRename"`, which means clicking anywhere outside the input discards the user's typed alias without confirmation. The user must press Enter to confirm. While this is a valid UX choice (click-away = cancel), it is lossy: a user who types a long alias and accidentally clicks the card border loses their work. Many desktop applications (macOS Finder, Windows Explorer) confirm on blur.

**Fix:** Consider confirming on blur instead, or add a small save/cancel button pair inside the card during rename mode.

### IN-02: `visibleOps()` and `overflowCount()` are functions, not computed properties

**File:** `src/components/seed/SeedCard.vue:27-32`
**Issue:** Both `visibleOps()` and `overflowCount()` are plain functions called in the template. Since they are called within the render function, they re-execute on every component re-render. The computation is trivial (`.slice(0,3)` and `Math.max`), so performance impact is negligible, but computed properties would be more idiomatic and avoid confusion about whether they are cached.

**Fix:**
```typescript
const visibleOps = computed(() => props.seed.operations.slice(0, 3).map((o) => o.opType));
const overflowCount = computed(() => Math.max(0, props.seed.operations.length - 3));
```

### IN-03: Local `TauriFile` interface extension for `File.path`

**File:** `src/components/queue/ImportZone.vue:20-22`
**Issue:** The interface `TauriFile extends File { path?: string }` is defined locally and duplicated in logic. If Tauri v2's `File` type in the webview changes (e.g., `path` is renamed to `filepath` or moved to a different API), the local cast `files[i] as TauriFile` will silently produce `undefined` for `path`, falling through to the warning message. The file would be silently skipped instead of triggering a clear error.

**Fix:** Consider using Tauri's official drag-drop event API (`onDragDropEvent` from `@tauri-apps/api/window`) instead of raw HTML5 drag-and-drop, which provides typed paths directly:
```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';
const unlisten = await getCurrentWindow().onDragDropEvent((event) => {
  if (event.payload.type === 'drop') {
    for (const path of event.payload.paths) {
      await importFile(path);
    }
  }
});
```

### IN-04: `formatBytes` does not guard against negative values

**File:** `src/components/queue/QueueList.vue:32-41`
**Issue:** The function only guards `bytes === 0`. If `bytes` is negative (e.g., from corrupted metadata or an edge case in the Rust backend), the function would return something like `-123.0 B`. While not a crash, it produces nonsensical output that could confuse users.

**Fix:**
```typescript
function formatBytes(bytes: number): string {
  if (bytes < 0 || !isFinite(bytes)) return '--';
  if (bytes === 0) return '0 B';
  // ... rest
}
```

### IN-05: Hardcoded English text "more" in operation overflow tag

**File:** `src/components/seed/SeedCard.vue:123`
**Issue:** The overflow tag displays `+${overflowCount()} more` with the word "more" hardcoded in English. This appears even when the locale is set to `zh-CN`.

**Fix:** Add an i18n key and use `t()`:
```typescript
// en.json: "seed.moreOperations": "+{count} more"
// zh-CN.json: "seed.moreOperations": "+{count} 项"
{{ t('seed.moreOperations', { count: overflowCount() }) }}
```

### IN-06: `pointermove` listener always attached, fires even when not resizing

**File:** `src/components/MainLayout.vue:44`
**Issue:** The `pointermove` event listener is attached on mount and never removed. On every pointer movement, the handler executes a check `if (!isResizing.value) return;`. While the early return makes the cost negligible, it's slightly wasteful to fire this function on every mouse move across the entire window. A more efficient pattern would be to attach the listener only when `isResizing.value` transitions to `true` and remove it on `false`.

**Fix:** Move `addEventListener`/`removeEventListener` for `pointermove` and `pointerup` into the `onResizePointerdown` / `onResizePointerup` functions, so listeners exist only during an active drag.

### IN-07: `NIcon` receives `undefined` as color prop

**File:** `src/components/queue/ImportZone.vue:103`
**Issue:** `<NIcon :size="48" :color="isDragging ? '#2080f0' : undefined">` passes `undefined` as the color prop when not dragging. While Vue 3 treats `undefined` as "no value" (equivalent to omitting the prop), it is unconventional and could trigger prop validation warnings in strict mode.

**Fix:** Use a conditional binding or provide a computed:
```typescript
const iconColor = computed(() => isDragging.value ? '#2080f0' : undefined);
```
Or use a conditional prop:
```html
<NIcon :size="48" v-bind="isDragging ? { color: '#2080f0' } : {}" />
```

---

_Reviewed: 2026-05-13T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
