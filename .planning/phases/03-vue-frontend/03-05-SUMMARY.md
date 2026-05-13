---
phase: 03-vue-frontend
plan: 05
subsystem: ui
tags: [vue, naive-ui, seed-management, card-component, list-component, i18n, pinia]

# Dependency graph
requires:
  - phase: 03-03
    provides: useSeed composable (generateSeed, renameSeed, deleteSeed, copySeed), useSeedStore Pinia store, Seed type definitions
provides:
  - SeedCard.vue component with hover-revealed actions, inline rename, NPopconfirm delete, selection state
  - SeedList.vue left-panel container with header, guided empty state, scrollable seed card list
affects: [03-06 MainLayout.vue (imports SeedList)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Naive UI tree-shakeable imports (NCard, NButton, NIcon, NText, NTag, NPopconfirm, NInput, NScrollbar, NSpace)
    - useMessage() for operation success/error toasts
    - UnoCSS important modifier (!) for overriding Naive UI component border styles
    - Hover-revealed action buttons via v-show + mouseenter/mouseleave
    - Inline-edit pattern (NInput replacing display text with Enter/Esc handlers)
    - Composition API store usage within components

key-files:
  created:
    - src/components/seed/SeedCard.vue
    - src/components/seed/SeedList.vue
  modified: []

key-decisions:
  - "SeedCard selection toggles (click same card → deselect, click different → select) per D-05"
  - "Rename uses inline NInput replacing alias text, Enter confirms, Esc/blur cancels per D-06"
  - "Delete uses NPopconfirm for destructive action confirmation per D-09"
  - "Copy and rename are silent execution (no confirmation, just toast feedback) per D-09"
  - "Action buttons visible only on card hover (v-show=isHovered with opacity transition) per D-06"
  - "Operation tags display max 3 visible, +N overflow tag for additional operations"
  - "Selected card visual: 2px solid #2080f0 border plus Zap icon indicator"
  - "SeedList empty state directly calls generateSeed() when CTA clicked per D-08"

patterns-established:
  - "Hover-revealed action pattern: v-show bound to mouseenter/mouseleave ref with @click.stop preventing card selection"
  - "Inline rename pattern: ref flag (isRenaming) toggling between NText display and NInput edit mode"
  - "Tiered confirmation pattern: destructive actions (delete) get NPopconfirm; silent actions (rename, copy) get toasts only"

requirements-completed: [UI-01]

# Metrics
duration: 5min
completed: 2026-05-13
---

# Phase 3 Plan 5: Seed Components Summary

**SeedList.vue left panel with SeedCard.vue cards featuring hover-revealed actions, inline rename, and NPopconfirm delete confirmation**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-05-13T11:10:46Z
- **Completed:** 2026-05-13T11:16:05Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- SeedCard.vue renders compact seed card with alias (bold), operation type tags (max 3 + overflow), and creation timestamp
- Hover reveals action buttons (rename/copy/delete) with opacity transition and @click.stop isolation
- Delete triggers NPopconfirm with i18n confirmation text before calling deleteSeed per D-09
- Copy calls copySeed silently with success/error toast via useMessage per D-10
- Inline rename: NInput replaces alias on Pencil click; Enter confirms, Esc/blur cancels
- Selection toggle on card click with 2px solid #2080f0 accent border and Zap icon indicator
- SeedList.vue renders header with title, seed count, and Generate Seed button
- Empty state (Sparkles icon 48px, descriptive text, large CTA) shown when store.seedCount === 0
- Empty state CTA directly calls generateSeed() per D-08
- Populated state: NScrollbar wrapping v-for of SeedCard components with space-y-2 gap
- All user-visible text uses vue-i18n t() function

## Task Commits

Each task was committed atomically:

1. **Task 1: Create SeedCard.vue** - `0939884` (feat)
2. **Task 2: Create SeedList.vue** - `4429fbe` (feat)

## Files Created/Modified
- `src/components/seed/SeedCard.vue` (169 lines) - Individual seed card with hover-revealed actions, inline rename, selection state, NPopconfirm delete
- `src/components/seed/SeedList.vue` (69 lines) - Left panel container with header, guided empty state, scrollable seed card list

## Decisions Made
- Selection toggle behavior (click selected card deselects, click different card selects) follows D-05 store contract
- Rename uses inline NInput replacing alias text rather than a separate modal — keeps context visible
- Action buttons use @click.stop to prevent card selection when clicking buttons — avoids unintended selection changes
- Operation tags capped at 3 visible with "+N more" overflow — prevents card height inflation for seeds with many operations
- All seed mutations (rename, copy, delete) use useMessage() for feedback — consistent UX per D-10

## Deviations from Plan

None - plan executed exactly as written. Lint-staged hooks (eslint, prettier) made cosmetic formatting adjustments during commit but no behavioral changes.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required. Components use existing i18n keys, store, and composable APIs.

## Next Phase Readiness
- SeedCard.vue and SeedList.vue are ready for integration into MainLayout.vue (Plan 03-06)
- Components import from useSeed composable and useSeedStore which are already committed
- All i18n keys (seed.*, notification.*) already exist in en.json and zh-CN.json
- No blocking dependencies — components can be imported immediately

## Threat Flags

None. Threat model mitigations (T-03-11 rename validation, T-03-12 delete confirmation) are implemented as specified. No new threat surface beyond plan scope.

---

*Phase: 03-vue-frontend*
*Completed: 2026-05-13*
