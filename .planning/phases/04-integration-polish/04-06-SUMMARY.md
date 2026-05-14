---
phase: 04-integration-polish
plan: 06
subsystem: ffmpeg-executor
tags:
  - bugfix
  - time-parsing
  - progress
  - gap-closure
requires:
  - BATCH-02
affects:
  - batch-progress-percent
tech-stack:
  added: []
  patterns:
    - "match on parts.len() for multi-format time string parsing"
key-files:
  modified:
    - src-tauri/src/ffmpeg/executor.rs
decisions:
  - "Using match on parts.len() rather than nested if/else for clarity"
metrics:
  duration: 6s
  completed_date: "2026-05-14"
---

# Phase 04 Plan 06: Fix parse_time_to_seconds for videos under 1 hour

Fixed a bug where `parse_time_to_seconds` only handled the `HH:MM:SS.mm` (3-part) colon-separated time format but not `MM:SS.mm` (2-part), causing per-file progress to stay at 0% for all videos shorter than 1 hour.

## Task Summary

| # | Task | Type | Commit | Files |
|---|------|------|--------|-------|
| 1 | Add MM:SS.mm support to parse_time_to_seconds | auto | 2da52e9 | src-tauri/src/ffmpeg/executor.rs |

## Changes

Replaced the single `if parts.len() == 3` block in `parse_time_to_seconds` with a `match parts.len()` to handle all three FFmpeg time formats:

- **3 parts (HH:MM:SS.mm):** Hours × 3600 + minutes × 60 + seconds (unchanged)
- **2 parts (MM:SS.mm):** Minutes × 60 + seconds (new)
- **Other/plain float:** Falls back to `time_str.parse()` with `unwrap_or(0.0)` (unchanged)

The function signature and call site (line 101) were not modified. All parse failures continue to default to 0.0, maintaining the existing defensive behavior against malformed FFmpeg output.

## Verification

- `cargo check -p sandwich` passed (exit code 0)
- `match parts.len()` handles all colon-count cases (2, 3, _)
- Mental trace: `"01:30.50"` → `["01","30.50"]`, len=2 → `1*60 + 30.50 = 90.5` seconds (correct)
- 3-part arithmetic unchanged: `h * 3600.0 + m * 60.0 + s`
- No-colon fallback unchanged: `time_str.parse().unwrap_or(0.0)`

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Gap Closure

| Gap | Resolution |
|-----|------------|
| "parse_time_to_seconds MM:SS bug — per-file progress stays at 0% for videos under 1 hour" | Fixed by adding 2-part match arm. BATCH-02 per-file progress now works for all video durations. |

## Self-Check: PASSED

- [x] `src-tauri/src/ffmpeg/executor.rs` exists with MM:SS support
- [x] Commit 2da52e9 exists in log
