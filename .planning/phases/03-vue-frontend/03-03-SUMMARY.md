---
phase: 03-vue-frontend
plan: "03"
type: execute
tags: [composables, ipc, events, useSeed, useQueue, useBatch]
requires: ["03-02"]
provides: ["03-04", "03-05", "03-06", "03-07"]
affects: [src/composables]
duration: "3m 47s"
completed: "2026-05-13T11:08:37Z"
extensions: 0
decisions:
  - "Module-level unlisten variables prevent duplicate event listeners on component re-mount"
  - "subscribe() performs initial data load after registering listeners for authoritative state sync"
  - "start_batch takes only seedId+outputDir; concurrency read from store by Rust (D-11)"
  - "video-imported event does optimistic store.addEntry; queue-updated event confirms with authority re-fetch"
  - "batch-file-error logs to console.warn; UI toast handling deferred to component layer"
  - "importVideo returns VideoEntry|null but does not update store — event listener handles optimistic update"
key-files:
  created:
    - src/composables/useSeed.ts
    - src/composables/useQueue.ts
    - src/composables/useBatch.ts
---

# Phase 03 Plan 03: IPC Composables Bridge Summary

**One-liner:** Three composables (useSeed, useQueue, useBatch) that encapsulate all 12 Tauri IPC `invoke()` calls and 7 `listen()` event subscriptions, following the exact pattern from `useFfmpeg.ts` with module-level unlisten variables, try-catch error wrapping, and reactive store updates.

## Plan Summary

Created three composable modules in `src/composables/` that serve as the sole integration layer between Vue 3 UI components and the Tauri Rust backend. Components must never call `invoke()` or `listen()` directly — composables encapsulate IPC calls, error handling, event subscription lifecycle, and store mutations.

### useSeed (`src/composables/useSeed.ts`)
- Wraps 5 IPC commands: `list_seeds`, `generate_seed`, `rename_seed`, `delete_seed`, `copy_seed`
- Subscribes to `seeds-updated` event (null payload — signal to re-fetch authority)
- `subscribe()` performs initial `loadSeeds()` after registering listener
- `generateSeed` and `copySeed` do optimistic store updates via `store.addSeed`
- Mutating commands return `boolean` or `Seed|null` for caller feedback

### useQueue (`src/composables/useQueue.ts`)
- Wraps 4 IPC commands: `get_queue`, `import_video`, `remove_from_queue`, `clear_queue`
- Two event listeners: `queue-updated` (invalidation signal, re-fetches authority) and `video-imported` (data-carrying, optimistic `store.addEntry`)
- `importVideo` returns `VideoEntry|null` but does not mutate store — the event listener handles it
- Two module-level unlisten variables (one per event source)

### useBatch (`src/composables/useBatch.ts`)
- Wraps 3 IPC commands: `start_batch`, `cancel_batch`, `get_batch_status`
- Four event listeners: `batch-progress`, `batch-file-error`, `batch-complete`, `batch-cancelled`
- `startBatch` takes `seedId` and `outputDir` only — concurrency read from config by Rust (D-11)
- `batch-complete` and `batch-cancelled` both call `store.stopProcessing(result)`
- Four module-level unlisten variables for the four event types

## Verification

All 3 composables pass automated verification:

```
=== src/composables/useSeed.ts ===
  5 invoked commands + 1 event listener + 1 unsubscribe

=== src/composables/useQueue.ts ===
  4 invoked commands + 2 event listeners + 1 unsubscribe

=== src/composables/useBatch.ts ===
  3 invoked commands + 4 event listeners + 1 unsubscribe
```

Individual task verifications (9 checks for useSeed, 8 for useQueue, 9 for useBatch) all passed.

## Commit History

| Commit   | Type | Description                                                                           |
| -------- | ---- | ------------------------------------------------------------------------------------- |
| d4f9bc1  | feat | create useSeed composable with all 5 seed IPC commands and event subscription         |
| 7d542a7  | feat | create useQueue composable with queue IPC commands and dual event subscriptions       |
| db23900  | feat | create useBatch composable with batch IPC commands and 4 event listeners              |

## Deviations from Plan

None — plan executed exactly as written. All three files match the exact implementation code specified in the plan tasks.

## Known Stubs

None. All composables perform real IPC calls and event subscriptions. No hardcoded empty values, placeholder text, or mock data.

## Threat Flags

No new threat surface beyond what was captured in the plan's `<threat_model>`. All four STRIDE threats (T-03-05 through T-03-08) are addressed at the appropriate disposition level.

## Self-Check

All three composable files exist and are committed:
- src/composables/useSeed.ts — created at d4f9bc1
- src/composables/useQueue.ts — created at 7d542a7
- src/composables/useBatch.ts — created at db23900
