---
phase: 03-vue-frontend
plan: 04
subsystem: ui
tags: [vue3, naive-ui, layout, composables, providers]

# Dependency graph
requires:
  - phase: 03-vue-frontend
    plan: "03"
    provides: "useSeed, useQueue, useBatch composables with subscribe/unsubscribe lifecycle"
provides:
  - "App.vue with Naive UI provider hierarchy (NConfigProvider > dialog > message > notification)"
  - "MainLayout.vue with dual-panel resizable layout and composable subscription lifecycle"
  - "ESLint browser globals configuration (enables window/document/PointerEvent in .vue files)"
affects: [03-vue-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Naive UI provider nesting: NConfigProvider > NDialogProvider > NMessageProvider > NNotificationProvider"
    - "App-lifetime composable subscriptions: subscribe all in onMounted, unsubscribe all in onUnmounted"
    - "Resizable panel: pointer events with clamped bounds (250px-70vw) and body-level user-select/cursor management"

key-files:
  created:
    - src/components/MainLayout.vue
  modified:
    - src/App.vue
    - eslint.config.mjs

key-decisions:
  - "NDialogProvider/NMessageProvider/NNotificationProvider nesting order follows established Naive UI pattern (order-independent per docs)"
  - "MainLayout subscribes all three composables in onMounted (app-lifetime pattern, Pitfall 2 avoidance)"
  - "Resize handle uses window-level pointermove/pointerup listeners (not element-level) for reliable drag outside handle"

patterns-established:
  - "Provider hierarchy: NConfigProvider > NDialogProvider > NMessageProvider > NNotificationProvider wrapping v-if view chain"
  - "Resizable dual-panel: NLayout with has-sider, NLayoutSider left, resize handle div, NLayoutContent right"
  - "Sticky batch footer: NLayoutFooter inside NLayoutContent for right-panel bottom controls"
  - "Conditional component rendering: BatchBanner v-if batchStore.isProcessing before QueueList"

requirements-completed: [UI-01, UI-02]

# Metrics
duration: 5min
completed: 2026-05-13
---

# Phase 3 Plan 4: App Layout and Provider Hierarchy Summary

**Naive UI provider tree wrapping a dual-panel resizable MainLayout with app-lifetime composable subscriptions for seed, queue, and batch event buses**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-13T11:15:50Z
- **Completed:** 2026-05-13T11:19:05Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- App.vue upgraded with NDialogProvider, NMessageProvider, NNotificationProvider wrapping the full view tree so useMessage/useDialog/useNotification work in all child components
- MainLayout.vue created with dual-panel NLayout (has-sider), draggable resize divider (250px-70% viewport clamp), and right-panel queue/batch stacking
- All three composable subscriptions initialized in onMounted and cleaned up in onUnmounted (app-lifetime pattern, no duplicate listeners)

## Task Commits

Each task was committed atomically:

1. **Task 1: Update App.vue with Naive UI providers and MainLayout import** - `071537a` (feat)
2. **Task 2: Create MainLayout.vue (dual-panel resizable layout + composable subscriptions)** - `3d799c0` (feat)

## Files Created/Modified
- `src/App.vue` - Added NDialogProvider, NMessageProvider, NNotificationProvider imports and provider tree; replaced PlaceholderHome with MainLayout
- `src/components/MainLayout.vue` - New dual-panel layout component with NLayout/has-sider, draggable resize handle, composable lifecycle, right-panel queue/batch stacking
- `eslint.config.mjs` - Added browser globals (globals.browser) to Vue languageOptions

## Decisions Made
- Provider nesting order (dialog > message > notification) follows established Naive UI pattern; order is independent per Naive UI docs
- MainLayout is the app-lifetime component that subscribes composables once and cleans up on unmount
- Resize handle uses window-level pointer events (not element-level) for reliable drag tracking outside the handle boundary

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] ESLint config missing browser globals**
- **Found during:** Task 2 (MainLayout.vue commit)
- **Issue:** MainLayout.vue uses `window`, `document`, `PointerEvent` in script — ESLint flagged as `no-undef`. The flat ESLint config lacked `globals.browser` environment declaration. Previous components (FFmpegStatus, App) did not use browser globals in script, so this surfaced here first.
- **Fix:** Added `import globals from 'globals'` and `globals: globals.browser` to the `.vue` languageOptions block in `eslint.config.mjs`
- **Files modified:** `eslint.config.mjs`
- **Verification:** `npx eslint src/components/MainLayout.vue` — 0 errors
- **Committed in:** `3d799c0` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (blocking)
**Impact on plan:** ESLint fix was necessary for commit to pass lint-staged pre-commit hook. No scope creep — simply enabled browser globals that were implicitly expected.

## Issues Encountered
- Absolute-path safety (#3099): First Write to App.vue used main repo path; corrected to worktree path (`/Users/ghost/Code/sandwich/.claude/worktrees/agent-a35fed75/src/App.vue`)
- Child components (SeedList, ImportZone, QueueList, BatchControls, BatchBanner) not yet created — expected for parallel wave execution; imports resolve when all wave plans complete

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- App.vue and MainLayout.vue provide the layout shell for all Phase 3 child components
- SeedList, ImportZone, QueueList, BatchControls, BatchBanner must exist before full build succeeds (created by parallel plans in this wave)
- NLayout contract (has-sider, position absolute, collapse-mode width) verified against Naive UI v2.3.0+ requirements

---
*Phase: 03-vue-frontend*
*Plan: 04*
*Completed: 2026-05-13*
