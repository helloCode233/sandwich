---
phase: 04-integration-polish
plan: 07
type: execute
subsystem: batch-banner
tags: [bug-fix, gap-closure, ui]
dependency_graph:
  requires: []
  provides: [correct-batch-completion-label]
  affects: [batch-results-display]
tech_stack:
  added: []
  patterns: [computed-property-derivation]
key_files:
  created: []
  modified: [src/components/batch/BatchBanner.vue]
decisions:
  - "Use (succeeded.length + failed.length) < progress.total as the sole cancellation indicator, matching BatchSummary logic exactly"
metrics:
  duration_seconds: 126
  completed_date: "2026-05-14"
---

# Phase 4 Plan 7: Fix BatchBanner Cancellation Detection Summary

**One-liner:** Fix BatchBanner labelText to distinguish batch completion from cancellation using the same (succeeded+failed) < total formula as BatchSummary.

## Context

BatchBanner.vue incorrectly used `batchStore.lastResult?.failed.length` (a truthiness check) as the determinant for displaying "Batch Cancelled" vs "Batch Complete". When a batch fully processed with some files failing (e.g., 3 succeeded + 2 failed = 5/5 files processed), the non-zero `failed.length` triggered the cancelled label. The fix replaces this with a `wasCancelled` computed that uses the identical formula from BatchSummary.vue: `(succeeded.length + failed.length) < progress.total`.

## Plan Completion Checklist

- [x] Task 1: Fix BatchBanner cancellation detection to match BatchSummary logic

## Design Summary

**Before the fix (buggy):**

```typescript
// BatchBanner.vue — uses failed.length as cancellation proxy
return batchStore.lastResult?.failed.length
  ? t('batch.summary.cancelledTitle')
  : t('batch.summary.completeTitle');
```

**After the fix:**

```typescript
// BatchBanner.vue — uses same formula as BatchSummary
const wasCancelled = computed(() => {
  if (!batchStore.lastResult) return false;
  const totalProcessed =
    batchStore.lastResult.succeeded.length + batchStore.lastResult.failed.length;
  return totalProcessed < batchStore.progress.total;
});

// labelText: return wasCancelled.value ? cancelledTitle : completeTitle;
```

## Commits

| Task | Type | Commit  | Message                                                                                       |
|------|------|---------|-----------------------------------------------------------------------------------------------|
| 1    | fix  | e15f7ac | fix(04-07): use (succeeded+failed)<total for BatchBanner cancellation detection |

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

None — this is a UI label fix with no new attack surface.

## Known Stubs

None.

## Self-Check

- [x] `src/components/batch/BatchBanner.vue` exists and contains `wasCancelled` computed
- [x] Commit `e15f7ac` exists in git log
- [x] Old `failed.length` truthiness check is removed
- [x] `wasCancelled` formula matches BatchSummary lines 13-16 exactly
- [x] vue-tsc passes (pre-existing tsconfig TS6310 unrelated)
