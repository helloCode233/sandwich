---
phase: 04-integration-polish
plan: 02
subsystem: frontend-contract
tags: [typescript, vue-i18n, PerFileProgress, batch-summary, localization]

# Dependency graph
requires: []
provides:
  - "PerFileProgress TypeScript interface (6 fields) for batch-file-progress event typing"
  - "17 new i18n keys in batch.* and batch.summary.* namespaces covering cancelling state, per-file progress labels, and completion summary"
  - "Deprecated batch.cancelConfirm replaced with cancelConfirmTitle + cancelConfirmBody for NModal dialog format"
affects: [04-03 plan (store/composable), 04-04 plan (components)]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TypeScript interfaces mirror Rust struct serialized payloads via serde rename_all=camelCase"
    - "vue-i18n template variables use {placeholder} syntax matching Rust event payload field names"
    - "i18n keys grouped by UI feature namespace (batch.summary.* for completion summary)"

key-files:
  created: []
  modified:
    - "src/types/batch.ts"
    - "src/locales/en.json"
    - "src/locales/zh-CN.json"

key-decisions:
  - "Followed plan exactly: PerFileProgress interface mirrors Rust struct with 6 primitive fields"
  - "Followed plan exactly: batch.cancelConfirm split into title + body for NModal dialog format"
  - "Followed plan exactly: batch.summary.* namespace groups 9 completion/cancellation keys"

patterns-established:
  - "Frontend contracts (types + i18n) established before store/component implementation in later waves"
  - "TypeScript interfaces use camelCase matching serde rename_all serialized payloads"

requirements-completed:
  - BATCH-02
  - BATCH-05

# Metrics
duration: 3min
completed: 2026-05-14
---

# Phase 04 Plan 02: Frontend Contract — PerFileProgress + Batch i18n Summary

**TypeScript PerFileProgress interface (6 fields) and 17 new i18n keys across both locales for batch processing UI contract**

## Performance

- **Duration:** 3 min
- **Started:** 2026-05-14T01:34:23Z
- **Completed:** 2026-05-14T01:37:20Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- PerFileProgress TypeScript interface with all 6 fields matching Rstruct PerFileProgress struct
- 17 new i18n keys added under batch.* and batch.summary.* namespaces across both en.json and zh-CN.json
- Deprecated batch.cancelConfirm single key replaced with cancelConfirmTitle + cancelConfirmBody for NModal dialog format
- Both locale files remain valid JSON with identical key structure

## Task Commits

Each task was committed atomically:

1. **Task 1: Add PerFileProgress TypeScript interface** - `cd9ef72` (feat)
2. **Task 2: Add new i18n keys to both locales** - `2088fb1` (feat)

**Plan metadata:** _(pending final commit)_

## Files Created/Modified
- `src/types/batch.ts` — Added PerFileProgress interface (6 fields: file, percent, currentFrame, totalFrames, fps, remainingSeconds) after FileResult
- `src/locales/en.json` — Replaced cancelConfirm with cancelConfirmTitle/body + 7 new batch keys + 9 batch.summary keys
- `src/locales/zh-CN.json` — Same key structure as en.json with Chinese translations

## New i18n Keys (17 per locale)

**batch.* (8 new):** cancelConfirmTitle, cancelConfirmBody, keepProcessing, cancelling, processingFile, fileProgress, fileEta, fileFailed, noOutputDir

**batch.summary.* (9 new):** completeTitle, cancelledTitle, succeededSection, failedSection, outputPath, fileError, clearResults, completeBody, cancelledBody

## Decisions Made
None — followed plan exactly as specified. The PerFileProgress interface fields and i18n key names match the 04-UI-SPEC.md design contract precisely.

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

The TypeScript PerFileProgress interface and i18n keys form the contract that Wave 3 (04-03: store/composable with event listeners) and Wave 4 (04-04: components with real UI) will build against. Both locales are complete and valid JSON.

---
*Phase: 04-integration-polish*
*Plan: 02*
*Completed: 2026-05-14*

## Self-Check: PASSED

- All 3 modified files exist on disk
- Both commits (cd9ef72, 2088fb1) found in git log
- All 17 i18n keys verified in both en.json and zh-CN.json
- Both locale files are valid JSON
- PerFileProgress interface has all 6 fields
- vue-tsc reports no errors in batch.ts

