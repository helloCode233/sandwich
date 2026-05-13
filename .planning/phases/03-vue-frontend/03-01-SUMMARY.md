---
phase: 03-vue-frontend
plan: "01"
subsystem: ui
tags:
  - typescript
  - i18n
  - serde
  - camelCase

# Dependency graph
requires: []
provides:
  - "TypeScript interfaces mirroring Rust serde camelCase structs for IPC contract"
  - "Bilingual i18n keys for all Phase 3 domains (seed, queue, import, batch, notification)"
affects:
  - "03-02 pinia stores"
  - "03-03 seed composables"
  - "03-04 video queue composables"
  - "03-05 batch composables"
  - "03-06 UI components"
  - "03-07 notification composables"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TypeScript type files mirror Rust serde struct fields with camelCase naming"
    - "Comment header line 1 references Rust source file path"
    - "Rust Option<T> maps to T | null in TypeScript"
    - "serde_json::Value maps to Record<string, unknown>"
    - "Rust enum variants become lowercase string literal unions (camelCase transform)"
    - "i18n locale files use dot-notation key groups per domain (seed.title, queue.empty, etc.)"

key-files:
  created:
    - src/types/seed.ts
    - src/types/video.ts
    - src/types/batch.ts
  modified:
    - src/locales/en.json
    - src/locales/zh-CN.json

key-decisions:
  - "Rust serde camelCase is the wire format for all IPC; TypeScript types match field-for-field"
  - "I18n keys organized by domain (seed, queue, import, batch, notification) per UI-SPEC copywriting contract"
  - "OperationType uses explicit #[serde(rename = \"opType\")] on Rust op_type field, so TypeScript uses opType (not derived from camelCase)"

patterns-established:
  - "Type definition pattern: comment header with Rust source path, export interface with camelCase fields, inline comments explaining serde transforms"
  - "I18n key structure: top-level domain groups with flat sub-keys using dot-notation in templates"

requirements-completed:
  - UI-01
  - UI-02

# Metrics
duration: 4min
completed: 2026-05-13
---

# Phase 3 Plan 1: TypeScript Types and I18n Keys Summary

**TypeScript type definitions (Seed, Operation, VideoEntry, VideoMetadata, BatchProgress) mirrored from Rust serde camelCase structs with 130+ bilingual i18n keys covering seed, queue, import, batch, and notification domains**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-13T18:39:31+08:00
- **Completed:** 2026-05-13T18:43:02+08:00
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Created 3 TypeScript type definition files (seed.ts, video.ts, batch.ts) with 7 exported interfaces/types matching Rust serde camelCase wire format exactly
- Defined all 7 operation types as a discriminated union (`mathOverlay` through `remux`)
- Extended both en.json and zh-CN.json with 5 new domain groups (116 total new key-value pairs across both files)
- All existing common, ffmpeg, and download keys preserved unchanged in both locale files
- Zero snake_case field names in any TypeScript type file (all camelCase matching Rust `#[serde(rename_all = "camelCase")]`)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create TypeScript type definitions (seed.ts, video.ts, batch.ts)** - `37bbe53` (feat)
2. **Task 2: Extend i18n locale files with Phase 3 translation keys** - `32d228b` (feat)

## Files Created/Modified

- `src/types/seed.ts` - Seed, Operation, OperationType type definitions (7 operation types)
- `src/types/video.ts` - VideoEntry, VideoMetadata, VideoStatus type definitions
- `src/types/batch.ts` - BatchProgress, BatchResult, FileResult type definitions
- `src/locales/en.json` - 58 new English keys across seed, queue, import, batch, notification groups
- `src/locales/zh-CN.json` - 58 new Chinese keys across seed, queue, import, batch, notification groups

## Decisions Made

None - plan executed exactly as specified. All type field names and i18n key values came directly from the Rust source models and the UI-SPEC copywriting contract.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All subsequent Phase 3 plans (stores, composables, components) can now import these types as their IPC contract
- I18n keys provide the complete vocabulary for all user-visible text in both zh-CN and en locales
- Ready for Plan 03-02 (Pinia stores importing these types)

---
*Phase: 03-vue-frontend*
*Completed: 2026-05-13*
