---
status: pending
phase: 03-vue-frontend
source: 03-VERIFICATION.md
started: 2026-05-13T19:48:00Z
updated: 2026-05-13T19:48:00Z
---

# Phase 3: Vue Frontend — Human UAT Checklist

All automated checks passed (23/23 must-haves, 19 tests, clean typecheck). 8 items need human testing.

## Tests

### 1. Dual-Panel Layout and Resizable Divider

expected: NLayout renders with NLayoutSider (SeedList) on left, NLayoutContent (ImportZone, QueueList, BatchBanner, BatchControls) on right. Resizable 4px divider between panels via drag, clamped 250px-70% viewport. Sticky BatchControls footer in right panel.
result: [pending]

### 2. Dark Theme Consistency

expected: All Naive UI components (NCard, NButton, NTag, NModal, NPopconfirm, NProgress, NSelect, NInput) render in dark color scheme. NConfigProvider darkTheme propagates fully. No light-themed artifacts in dialogs, notifications, or empty states.
result: [pending]

### 3. Drag-and-Drop Video Import

expected: Dragging video file over ImportZone shows solid #2080f0 border, background highlight, icon color change, and text swaps to "Drop to import". Dropping file imports it and it appears in QueueList with Valid status and metadata.
result: [pending]

### 4. Native File Dialog Import

expected: Clicking "Add Videos" opens native OS file dialog filtered to video extensions. Selecting a file imports it and displays metadata (duration, dimensions, size, codec) in QueueList.
result: [pending]

### 5. Seed Card Interactions

expected: Hover over seed card reveals Pencil/Copy/Trash2 action buttons with smooth opacity transition. Pencil activates inline rename (NInput replaces text, Enter confirms, Esc/blur cancels). Copy shows success toast. Delete shows NPopconfirm then removes seed. Clicking card toggles selection (blue accent border + Zap icon).
result: [pending]

### 6. Batch Processing Start/Cancel

expected: Selecting a seed enables Start button. Clicking Start changes button to Cancel (different type/color), BatchBanner appears with NProgress bar and "completed/total" counter, controls disable during processing. Cancel fires cancel_batch IPC, banner disappears, batch resets to Idle.
result: [pending]

### 7. i18n Locale Coverage (en and zh-CN)

expected: Switching locale to zh-CN translates all UI text — seed management (title, generate button, rename input, delete confirmation), queue (title, add videos button, metadata labels, clear all confirmation), batch (seed selector placeholder, concurrency label, start/cancel button, progress label), notifications (success/error toasts), and empty states.
result: [pending]

### 8. Guided Empty States

expected: With no seeds: SeedList shows Sparkles icon + "No seeds yet" message + "Generate First Seed" CTA that calls generateSeed(). With no videos: QueueList shows Clapperboard icon + "No videos yet" message + "Add Videos" CTA that opens file dialog.
result: [pending]

## Summary

- Item count: 8
- Complete: 0
- Blocked: 0
