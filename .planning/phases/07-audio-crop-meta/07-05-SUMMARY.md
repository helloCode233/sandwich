---
phase: 07-audio-crop-meta
plan: 05
subsystem: ffmpeg
tags: [rust, tauri, ffmpeg, ffprobe, metadata, executor, probe]

# Dependency graph
requires:
  - phase: 07-audio-crop-meta
    provides: Plan 03 — MetadataContext struct, updated build_filter_args_separated Vec return type
  - phase: 07-audio-crop-meta
    provides: Plan 01 — Phase 7 OperationType variants (AudioResample, MetadataSelectiveErase, etc.)
provides:
  - Executor loop handles Vec<(FilterKind, Vec<String>)> per operation (VideoSpeed returns 2)
  - -vsync vfr injected before encoder args when FrameDrop present (D-17)
  - probe_global_metadata extracts all format.tags from ffprobe
  - MetadataContext conditionally constructed by ffprobe when MetadataSelectiveErase present
  - build_filter_args_separated called with metadata_ctx.as_ref() parameter
affects: [07-06, ffmpeg, executor]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Conditional ffprobe before filter building: probe only when MetadataSelectiveErase in seed"
    - "-vsync vfr injection pattern: detect FrameDrop -> prepend before encoder args"

key-files:
  created: []
  modified:
    - src-tauri/src/ffmpeg/executor.rs
    - src-tauri/src/ffmpeg/probe.rs
    - src-tauri/src/ffmpeg/filters.rs
    - src-tauri/src/models/seed.rs
    - src-tauri/src/commands/seed.rs

key-decisions:
  - "-vsync vfr goes BEFORE encoder args in FFmpeg command line to take effect"
  - "MetadataContext is lazy: only probed when MetadataSelectiveErase is in seed operations"
  - "ffprobe failure for metadata returns None context (graceful fallback to full erase)"

patterns-established:
  - "build_filter_args_separated returns Vec<(FilterKind, Vec<String>)> for multi-filter operations"

requirements-completed: [D-09, D-12, D-17]

# Metrics
duration: 19min
completed: 2026-05-18
---

# Phase 7 Plan 5: Executor Phase 7 Upgrade — Multi-Filter Loop, FrameDrop vsync, Global Metadata Probe

**Executor loop handles Vec of FilterKinds from build_filter_args_separated; -vsync vfr injected for FrameDrop; conditional ffprobe global metadata extraction for selective erase**

## Performance

- **Duration:** 19min 34s
- **Started:** 2026-05-18T12:46:41Z
- **Completed:** 2026-05-18T13:06:15Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Executor operation loop iterates Vec<(FilterKind, Vec<String>)> to support multi-filter operations (VideoSpeed returns both -vf and -af)
- -vsync vfr injected before encoder args when FrameDrop is present in seed operations (D-17 fix: prevents ffmpeg from inserting duplicate frames to maintain CFR)
- probe_global_metadata function extracts all format.tags from ffprobe for MetadataSelectiveErase
- MetadataContext conditionally constructed via ffprobe probe when MetadataSelectiveErase operation type is present
- Graceful fallback: ffprobe failure emits batch-log warning and continues with None context (full erase)

## Task Commits

Each task was committed atomically:

1. **Task 1: Adapt executor loop for multi-FilterKind return type and inject -vsync vfr** - `fe34f0e` (feat)
2. **Task 2: Add probe_global_metadata function and extend RawFormat.tags field** - `936784b` (feat)
3. **Task 3: Wire MetadataContext into the executor — conditional ffprobe before filter building** - `27a0ea5` (feat)

**Plan metadata:** (pending — committed with SUMMARY.md)

## Files Created/Modified
- `src-tauri/src/ffmpeg/executor.rs` - Multi-FilterKind loop, has_frame_drop detection, -vsync vfr injection, MetadataContext construction + wiring
- `src-tauri/src/ffmpeg/probe.rs` - probe_global_metadata function, RawFormat.tags field
- `src-tauri/src/ffmpeg/filters.rs` - MetadataContext struct, build_metadata_selective_erase_filter, updated build_filter_args_separated signature (Plan 03 dependencies)
- `src-tauri/src/models/seed.rs` - Phase 7 OperationType variants (AudioResample through TrimEdges, Plan 01 dependency)
- `src-tauri/src/commands/seed.rs` - Wildcard arm for Phase 7 operation types in seed param generation

## Decisions Made
- -vsync vfr is placed BEFORE encoder args in the FFmpeg command line (prepend pattern) to ensure it takes effect
- MetadataContext is lazily constructed: ffprobe runs only when MetadataSelectiveErase is in the seed's operation list
- ffprobe failure gracefully degrades to full metadata erase (None context) with a batch-log warning event
- Plan 03 dependency (MetadataContext struct, build_filter_args_separated Vec return) applied as Rule 3 auto-fix to enable compilation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Applied Plan 03 dependency changes to filters.rs (MetadataContext + build_filter_args_separated signature)**
- **Found during:** Task 1 (executor loop update)
- **Issue:** Plan 03 changes (MetadataContext struct, Vec return type for build_filter_args_separated, build_metadata_selective_erase_filter) were not present in this worktree. The orchestrator spawned this worktree at commit da55cd4 which predates Plan 03's commits.
- **Fix:** Added MetadataContext struct, HashMap import, updated build_filter_args_separated signature (Option<&MetadataContext> parameter, Vec return), added build_metadata_selective_erase_filter function, wrapped all return values in vec![]
- **Files modified:** src-tauri/src/ffmpeg/filters.rs
- **Committed in:** fe34f0e (Task 1 commit)

**2. [Rule 3 - Blocking] Applied Plan 01 dependency changes to seed.rs (Phase 7 OperationType variants)**
- **Found during:** Task 1 (cargo check after adding MetadataSelectiveErase match)
- **Issue:** Phase 7 OperationType enum variants (AudioResample, AudioVolume, AudioPitch, AudioEQ, AudioChannel, Crop, MetadataWrite, MetadataSelectiveErase, VideoSpeed, TrimEdges) were not defined in this worktree. Plan 01 was supposed to add them.
- **Fix:** Added 10 Phase 7 variants to OperationType enum with proper serde camelCase naming
- **Files modified:** src-tauri/src/models/seed.rs
- **Committed in:** fe34f0e (Task 1 commit)

**3. [Rule 3 - Blocking] Added wildcard arms for Phase 7 types in seed command param generation**
- **Found during:** Task 1 (cargo check after adding OperationType variants)
- **Issue:** src/commands/seed.rs had an exhaustive match on OperationType that didn't include the new Phase 7 variants
- **Fix:** Added wildcard arm returning serde_json::json!({}) for Phase 7 types; these are generated by dedicated Phase 7 seed generators
- **Files modified:** src-tauri/src/commands/seed.rs
- **Committed in:** fe34f0e (Task 1 commit)

---

**Total deviations:** 3 auto-fixed (all Rule 3 - blocking dependencies)
**Impact on plan:** Dependency application necessary for compilation. All changes are idempotent with Plan 01/03 commits — the orchestrator merge will unify these into identical code.

## Issues Encountered
- Cargo fmt pre-commit hook flagged formatting issues on has_frame_drop closure (split across lines) and probe_global_metadata function signature (single long line); resolved by running cargo fmt before commit
- Worktree did not contain Plan 01 or Plan 03 changes at spawn time; applied minimal dependency changes to enable compilation while preserving merge compatibility

## Next Phase Readiness
- Executor now supports all Phase 7 operation types via the updated build_filter_args_separated dispatch
- -vsync vfr is correctly injected for FrameDrop operations
- MetadataSelectiveErase receives current file metadata via conditional ffprobe
- Ready for Plan 07-06 (frontend integration)

---
*Phase: 07-audio-crop-meta*
*Completed: 2026-05-18*
