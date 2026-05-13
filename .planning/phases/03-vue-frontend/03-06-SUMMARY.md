---
phase: 03-vue-frontend
plan: 06
subsystem: queue-import
tags: [ui, import, drag-drop, queue-management, confirmation-dialog]
depends_on: ["03-03"]
requires:
  - useQueue composable (importVideo, removeFromQueue, clearQueue)
  - useQueueStore (entries, entryCount)
  - vue-i18n (queue, import, notification keys)
  - @tauri-apps/plugin-dialog (open)
provides:
  - ImportZone.vue (HTML5 drag-drop import zone)
  - QueueList.vue (video queue display with metadata and actions)
affects: []
tech-stack:
  added: []
  patterns:
    - "vue-i18n t() for all user-facing text"
    - "HTML5 DragEvent + TauriFile interface for cross-platform drop"
    - "Naive UI useDialog().warning() for irreversible destructive actions"
    - "Manual v-for cards (not NDataTable) for full style control"
key-files:
  created:
    - src/components/queue/ImportZone.vue
    - src/components/queue/QueueList.vue
  modified:
    - eslint.config.mjs (added browser globals for .vue files)
decisions:
  - "Used TauriFile interface (extends File with path?: string) instead of (file as any) to satisfy @typescript-eslint/no-explicit-any"
  - "onDragLeave checks relatedTarget to avoid flicker when dragging over child elements inside the drop zone"
  - "Metadata line format: duration | resolution | size | codec (FPS omitted from display line per plan specification)"
duration: ~13 min
completed_date: "2026-05-13"
---

# Phase 3 Plan 6: Queue Import & Management UI Summary

ImportZone.vue (HTML5 drag-drop video import zone with native file dialog fallback) and QueueList.vue (video queue display with metadata table, per-item remove, clear-all with NModal confirmation, and guided empty state).

## Purpose

Created two Vue 3 queue management components for the video import and queue management UI (upper section of the right panel). Users drag videos into the hot zone or use the file dialog to import. QueueList shows all imported videos with metadata, status indicators, and queue management actions.

## Tasks Executed

### Task 1: Create ImportZone.vue (drag-drop zone + file dialog button)

**Commit:** `225dc33`

- HTML5 drag-drop zone with `@dragover`, `@dragleave`, `@drop` event handlers
- Visual feedback on dragover: solid `#2080f0` border + accent-tinted background + icon color change
- Text changes from `t('import.dropHere')` to `t('import.dropActive')` on drag-over
- `onDragLeave` checks `relatedTarget` to avoid flickering when dragging over child elements
- `onDrop` extracts file paths via `TauriFile` interface (extends `File` with `path?: string`) for cross-platform compatibility
- macOS fallback: warns user when dropped file path is undefined
- "Add Videos" button opens native file dialog via `@tauri-apps/plugin-dialog` `open()` with video extension filter and `multiple: true`
- Each imported file shows success toast (filename) or error toast via Naive UI `useMessage()`
- All text uses `vue-i18n` `t()` function
- Minimum height 120px per UI-SPEC

### Task 2: Create QueueList.vue (video queue table, empty state, clear-all confirmation)

**Commit:** `9391629`

- Header row: `t('queue.title')` with entry count badge + "Clear All" button (visible only when entries > 0)
- Empty state (D-07): Clapperboard icon 48px, descriptive text, "Add Videos" CTA button
- Empty state CTA (D-08): opens native file dialog and imports videos on confirmation
- Queue items rendered via `v-for="(entry, index) in store.entries"` with `:key="entry.filepath"`
- Each item: filename (bold, truncated), status NTag (success=valid with CheckCircle, warning=invalid with AlertCircle), metadata line, remove Trash2 button
- Metadata formatting functions: `formatDuration()` (MM:SS or HH:MM:SS), `formatBytes()` (B/KB/MB/GB), `formatCodec()` (uppercase)
- Remove button calls `onRemove(index)` which invokes `removeFromQueue(index)` and shows toast with filename
- Clear All uses `useDialog().warning()` (NModal confirmation) per D-09
- NScrollbar wraps queue list for consistent dark theme scrollbar
- All text uses `vue-i18n` `t()` function

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed missing browser globals in ESLint config for .vue files**
- **Found during:** Task 1 pre-commit hook
- **Issue:** ESLint flat config was missing `globals.browser` in the `.vue` files block, causing `no-undef` errors for DOM types (`DragEvent`, `HTMLElement`, `Node`, `console`)
- **Fix:** Added `import globals from 'globals'` and `globals: globals.browser` to the Vue file language options in `eslint.config.mjs`
- **Files modified:** `eslint.config.mjs`

**2. [Rule 1 - Bug] Fixed `(file as any).path` explicit any violation**
- **Found during:** Task 1 pre-commit hook
- **Issue:** Using `(file as any).path` triggered `@typescript-eslint/no-explicit-any`
- **Fix:** Defined `TauriFile` interface (`extends File { path?: string }`) and cast dropped files as `TauriFile`
- **Files modified:** `src/components/queue/ImportZone.vue`

**3. [Rule 1 - Bug] Fixed `queue.removed` i18n key missing filename parameter**
- **Found during:** Task 2 implementation
- **Issue:** Plan code called `message.success(t('queue.removed'))` without `{filename}` parameter, but i18n keys expect interpolation (zh-CN: "已移除「{filename}」", en: "'{filename}' removed")
- **Fix:** Pass `{ filename }` to i18n call, extracting filename from `store.entries[index]?.filename`
- **Files modified:** `src/components/queue/QueueList.vue`

**4. [Rule 1 - Bug] Removed unused imports and functions from QueueList.vue**
- **Found during:** Task 2 ESLint check
- **Issue:** `ref` import and `formatFps` function were declared but never used
- **Fix:** Removed both
- **Files modified:** `src/components/queue/QueueList.vue`

## Verification

### Plan-level checks
```
onDrop: 2           import-zone: 4
onDragOver: 2       removeFromQueue: 2
clearQueue: 2       formatDuration: 2
Queue components verified
```

### Acceptance criteria

All acceptance criteria from the plan met:

| Criterion | Status |
|-----------|--------|
| ImportZone.vue renders drag-drop zone with dashed border and visual feedback on dragover | PASS |
| ImportZone.vue drag-over changes border to solid accent, background to accent-tinted, text to "Drop to import" | PASS |
| ImportZone.vue "Add Videos" button opens native file dialog (multiple: true, video extension filter) | PASS |
| ImportZone.vue shows success/error toast per imported file | PASS |
| QueueList.vue renders guided empty state when queue is empty (icon + text + CTA) | PASS |
| QueueList.vue renders video entries with filename, status tag, metadata line, remove button | PASS |
| QueueList.vue Clear All uses NModal confirmation before calling clearQueue() | PASS |
| QueueList.vue metadata formatting: duration as MM:SS, size as human-readable, codec uppercase | PASS |
| Both components use useMessage() for operation feedback per D-10 | PASS |
| Both components use vue-i18n t() for all text | PASS |

## Commits

- `225dc33`: feat(03-06): create ImportZone.vue with HTML5 drag-drop zone and file dialog
- `9391629`: feat(03-06): create QueueList.vue with metadata table, empty state, and clear-all confirmation

## Self-Check: PASSED

- [x] `src/components/queue/ImportZone.vue` exists (144 lines, >70 min_lines)
- [x] `src/components/queue/QueueList.vue` exists (185 lines, >80 min_lines)
- [x] Commit `225dc33` exists in git log
- [x] Commit `9391629` exists in git log
- [x] ESLint passes with 0 errors and 0 warnings on both files
- [x] No untracked files remaining
