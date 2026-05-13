---
phase: 02-rust-backend
plan: 02
subsystem: ffmpeg-interaction-layer
tags: [ffprobe, ffmpeg, filters, executor, progress-streaming, cancellation]
dependency-graph:
  requires: [02-01]
  provides: [04-rust-backend, commands-import, commands-batch]
  affects: [all-tauri-commands-using-ffmpeg]
tech-stack:
  added: [ffmpeg-sidecar, serde, tauri]
  patterns: [AtomicBool-cancel, FfmpegCommand-new_with_path, child.iter-progress-drain]
key-files:
  created: [src-tauri/src/ffmpeg/probe.rs, src-tauri/src/ffmpeg/filters.rs, src-tauri/src/ffmpeg/executor.rs]
  modified: []
decisions:
  - "Used ffmpeg-sidecar 2.5.x actual API (LogLevel, no FfmpegStdin, time-based progress) instead of plan's assumed API"
  - "Progress computed from time string (HH:MM:SS) vs total duration rather than non-existent percent field"
duration: 00:20:00
completed-date: "2026-05-13"
test-results:
  total: 12
  passed: 12
  failed: 0
---

# Phase 02 Plan 02: FFmpeg Interaction Layer Summary

**One-liner:** Built ffprobe metadata extraction, 7-filter chain construction with SEED-04 safety constraints, and cancel-aware FFmpeg process executor with progress streaming using ffmpeg-sidecar 2.5.x APIs.

## Task Summary

### Task 1: ffprobe metadata extraction (probe.rs)
- **Commit:** `3602b38` — `feat(02-rust-backend-02): implement ffprobe metadata extraction`
- Created `extract_metadata()` function that invokes ffprobe, parses JSON output, extracts all 6 VideoMetadata fields (duration, width, height, size, codec, fps), validates at least one video stream exists (D-14), and parses FPS from r_frame_rate num/den format.
- 4 unit tests covering: nonexistent file rejection, text file rejection, FPS parse 30/1, FPS parse 30000/1001 (29.97).

### Task 2: Filter chain builders (filters.rs)
- **Commit:** `b2a6c1a` — `feat(02-rust-backend-02): implement FFmpeg filter chain builders for 7 operation types`
- Created 7 public filter builder functions plus `build_filter_args()` dispatcher, one per OperationType variant: MathOverlay, PixelShift, FrameDrop, GopModify, MetadataErase, AudioTweak, Remux.
- All 3 SEED-04 safety constraints enforced via clamping: opacity <= 0.15, pixel shift [-3, 3], frame drop interval >= 15.
- 8 unit tests covering: clamp verification per type (math overlay opacity, pixel shift dx, frame drop interval, GOP size), metadata erase args, remux args, and full dispatch coverage.

### Task 3: FFmpeg executor (executor.rs)
- **Commit:** `3ebfc60` — `feat(02-rust-backend-02): implement FFmpeg executor with progress and cancel`
- Created `execute_single_file()` using `FfmpegCommand::new_with_path()` (never `new()`), `.input()`, `.args()`, `.output()`, `.spawn()`, and `child.iter()` for stderr draining (Pitfall 1 mitigation).
- AtomicBool cancel check with `Ordering::SeqCst` (Pitfall 5 mitigation), `child.kill()` + incomplete file cleanup on cancel (D-10).
- Emits `batch-progress` (time-based percentage) and `batch-log` (warnings/errors) events.
- `make_output_path()` with collision-safe suffix naming (D-16): `{stem}_{alias}.{ext}`, appends `-1`, `-2`, etc. if file exists.
- `parse_time_to_seconds()` helper for converting FFmpeg time strings to seconds for progress calculation.
- No unit tests (requires FFmpeg binary), but `cargo check` passes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed ffmpeg-sidecar 2.5.x API mismatches in executor.rs**
- **Found during:** Task 3 compilation
- **Issue:** Plan assumed `FfmpegStdin::DevNull`, `.stdin()` builder method, `FfmpegLogLevel` enum, and `progress.percent` field — none of which exist in ffmpeg-sidecar 2.5.x.
- **Fix:** Removed `.stdin()` call entirely (not needed for file-to-file processing). Changed `FfmpegLogLevel` to `LogLevel` from `ffmpeg_sidecar::event`. Replaced `progress.percent` with time-based calculation using `parse_time_to_seconds()` and `entry.metadata.duration_secs`. Changed `PathBuf` to string for `.output()` which requires `AsRef<str>`.
- **Files modified:** `src-tauri/src/ffmpeg/executor.rs`
- **Commit:** `3ebfc60`

## Verification

```bash
cd src-tauri && cargo check 2>&1    # PASS — zero errors
cd src-tauri && cargo test -- ffmpeg 2>&1  # PASS — 12/12 tests
```

| Verification Item | Result |
|-------------------|--------|
| cargo check exits 0 | PASS |
| cargo test -- ffmpeg (12 tests) | 12/12 PASS |
| probe.rs extracts 6 VideoMetadata fields | PASS |
| probe.rs validates video stream (D-14) | PASS |
| filters.rs 7 builders + dispatch | PASS |
| filters.rs SEED-04 opacity <= 0.15 | PASS |
| filters.rs SEED-04 pixel shift [-3,3] | PASS |
| filters.rs SEED-04 frame drop >= 15 | PASS |
| executor.rs new_with_path() usage | PASS |
| executor.rs child.iter() (Pitfall 1) | PASS |
| executor.rs Ordering::SeqCst (Pitfall 5) | PASS |
| executor.rs kill + cleanup (D-10) | PASS |
| executor.rs collision-safe naming (D-16) | PASS |

## Threat Flags

None — all threat mitigations from the plan's threat model were implemented: T-02-04 (FFmpeg argument injection mitigated via type-safe serde parsing), T-02-05 (stderr deadlock mitigated via child.iter()), T-02-09 (ARM cancel visibility mitigated via SeqCst).

## Self-Check: PASSED

- [x] probe.rs: 148 lines, committed at 3602b38
- [x] filters.rs: 248 lines, committed at b2a6c1a
- [x] executor.rs: 213 lines, committed at 3ebfc60
- [x] No stubs found in any created files
- [x] cargo check exits 0 (dead_code warnings expected — not yet wired to commands)
- [x] cargo test 12/12 pass
