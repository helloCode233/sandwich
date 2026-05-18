---
phase: 07-audio-crop-meta
plan: 03
subsystem: ffmpeg
tags: [ffmpeg, filters, rust, audio, crop, metadata, frame-drop, select]

# Dependency graph
requires:
  - phase: 07-audio-crop-meta
    plan: 01
    provides: OperationType enum with 30 variants (Phase 7 additions: AudioResample through TrimEdges)
provides:
  - 10 new FFmpeg filter builder functions (5 audio, 2 video, 2 metadata, FrameDrop rewrite)
  - build_filter_args dispatches all 30 OperationType variants exhaustively
  - build_filter_args_separated returns Vec<(FilterKind, Vec<String>)> with metadata_ctx parameter
  - MetadataContext struct (HashMap<String, String>) for ffprobe data dependency
  - VideoSpeed and TrimEdges return both VideoFilter and AudioFilter (multi-filter support)
  - FrameDrop uses select filter for true frame decimation (not setpts jitter)
affects:
  - 07-audio-crop-meta
    plan: 05
    why: executor.rs and probes must pass MetadataContext to build_filter_args_separated

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Filter builder pattern: each OperationType has a dedicated build_*_filter function returning Vec<String> of FFmpeg CLI args"
    - "Safety clamping: all numeric parameters clamped to safe ranges before format!() interpolation"
    - "Multi-filter dispatch: build_filter_args_separated returns Vec of tuples so operations like VideoSpeed can contribute both -vf and -af"

key-files:
  created: []
  modified:
    - src-tauri/src/ffmpeg/filters.rs (10 new filter builders, MetadataContext struct, both dispatch functions updated, FrameDrop rewritten, tests updated)
    - src-tauri/src/ffmpeg/executor.rs (updated to iterate over new Vec return type)

key-decisions:
  - "FrameDrop rewired from setpts micro-timing jitter to select='mod(n+1,N)' filter for true frame decimation per D-17, D-18, D-19"
  - "AudioPitch uses asetrate+atempo+aresample chain (no rubberband) per D-05 built-in constraint"
  - "build_filter_args_separated return type changed to Vec<(FilterKind, Vec<String>)> to support multi-filter operations (VideoSpeed, TrimEdges)"
  - "MetadataContext struct defined in filters.rs (not probe.rs) so Plan 03 compiles independently in Wave 2"
  - "build_filter_args has zero wildcard arms — all 30 OperationType variants exhaustively matched"

patterns-established:
  - "Safety clamp -> format!() -> Ok(vec![flag, expr]) pattern used uniformly across all filter builders"
  - "Multi-filter operations return vec of (FilterKind, args) tuples, single-filter operations return vec of one tuple"

requirements-completed: [D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-10, D-11, D-12, D-13, D-14, D-15, D-16, D-17, D-18, D-19]

# Metrics
duration: 15min
completed: 2026-05-18
---

# Phase 7 Plan 3: FFmpeg Filter Builders Summary

**10 new filter builder functions for 10 Phase 7 operation types, FrameDrop rewritten to select filter, exhaustive 30-variant dispatch with multi-FilterKind return support**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-05-18T11:51:24Z
- **Completed:** 2026-05-18T12:06:33Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- 5 audio filter builders: AudioResample (aresample), AudioVolume (+/-3dB), AudioPitch (asetrate+atempo+aresample), AudioEQ (parametric), AudioChannel (swap/mono/stereo)
- 3 video/duration filter builders: Crop (asymmetric crop+scale with lanczos), VideoSpeed (synchronized setpts+atempo), TrimEdges (head/tail/both trim with PTS reset)
- 2 metadata filter builders: MetadataWrite (6 field types), MetadataSelectiveErase (category-based erase with ffprobe-driven writeback)
- FrameDrop rewritten: select='mod(n+1,N)' replaces setpts sine jitter for true frame decimation
- build_filter_args exhaustively dispatches all 30 OperationType variants (zero wildcard arms)
- build_filter_args_separated returns Vec<(FilterKind, Vec<String>)> — VideoSpeed and TrimEdges return both VideoFilter and AudioFilter

## Task Commits

Each task was committed atomically:

1. **Task 1: 5 audio filter builder functions** - `d5c58a3` (feat)
2. **Task 2: 3 video/duration filter builder functions** - `13c52b0` (feat)
3. **Task 3: Metadata builders, FrameDrop rewrite, dispatch update** - `767652d` (feat)

## Files Created/Modified
- `src-tauri/src/ffmpeg/filters.rs` - 10 new filter builder functions, MetadataContext struct, exhaustive 30-variant dispatch, FrameDrop select rewrite, updated tests (+422/-70 lines)
- `src-tauri/src/ffmpeg/executor.rs` - Updated to iterate over new `Vec<(FilterKind, Vec<String>)>` return type from `build_filter_args_separated`

## Decisions Made
- FrameDrop migrated from setpts micro-timing jitter to select='mod(n+1,N)' filter per D-17/D-18/D-19 requirements
- AudioPitch uses asetrate+atempo+aresample chain to avoid rubberband external dependency (D-05 constraint)
- build_filter_args_separated return type changed from single tuple to Vec of tuples to support multi-filter operations
- MetadataContext struct defined in filters.rs (not probe.rs) so this plan compiles independently
- build_filter_args has zero wildcard arms — all 30 variants exhaustively matched (compiler catches new variants)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated executor.rs for new return type**
- **Found during:** Task 3 (build_filter_args_separated signature change)
- **Issue:** executor.rs was unpacking the old single-tuple `(kind, args)` return type. The new `Vec<(FilterKind, Vec<String>)>` signature caused a compile error.
- **Fix:** Changed executor.rs to iterate over the returned Vec, consuming values by value (not by reference). Added `_args` prefix to unused binding.
- **Files modified:** src-tauri/src/ffmpeg/executor.rs
- **Committed in:** 767652d (Task 3 commit)

**2. [Rule 1 - Bug] Fixed unreachable wildcard pattern warning**
- **Found during:** Task 3 (verification)
- **Issue:** After adding all 10 Phase 7 match arms, the `_ =>` wildcard in `build_filter_args` became unreachable (all 30 variants covered). Cargo check emitted a warning.
- **Fix:** Removed the dead wildcard arm. If a new OperationType variant is added, the compiler will now error on the non-exhaustive match (safety net).
- **Files modified:** src-tauri/src/ffmpeg/filters.rs
- **Committed in:** 767652d (Task 3 commit)

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both auto-fixes were necessary for correctness. No scope creep.

## Issues Encountered
- lint-staged pre-commit hook required `cargo fmt --check` to pass; format! macro arguments in build_audio_pitch_filter needed multi-line formatting. Fixed by running `cargo fmt` before commit.

## Next Phase Readiness
- All 30 OperationType variants have filter builders and dispatch arms in both functions
- MetadataContext struct ready for Plan 05 (executor/probe integration)
- FrameDrop select filter ready for seed generation to use `interval` parameter instead of `offset`/`period`
- executor.rs automatically updated for new Vec return type — no additional migration needed
- No stubs remain; all 30 filter builders are fully implemented

---
*Phase: 07-audio-crop-meta*
*Completed: 2026-05-18*
