---
phase: 07
plan: 11
subsystem: integration-tests
tags: [gap-closure, bug-fix, test-helper, ffprobe, frame-counting]
depends_on: [07-10]
requires: []
provides: ["reliable frame counting across all codecs/containers"]
affects: ["src-tauri/tests/filter_integration_tests.rs"]
tech-stack:
  added: []
  patterns: ["ffprobe -count_frames for reliable frame counting", "JSON fallback parsing for optional ffprobe fields"]
key-files:
  modified: ["src-tauri/tests/filter_integration_tests.rs"]
decisions:
  - "Use -count_frames flag to force ffprobe to actually count frames instead of relying on stream header metadata"
  - "Switch from CSV to JSON output format to enable fallback between nb_read_frames and nb_frames fields"
metrics:
  duration: "~5 minutes"
  completed-date: "2026-06-06T20:21:40Z"
---

# Phase 07 Plan 11: Fix count_frames Helper for Reliable Frame Counting

**One-liner:** Fixed `count_frames()` helper in integration tests to reliably count video frames across all codecs and containers by using ffprobe's `-count_frames` flag with JSON fallback parsing.

## Objective

Fix the `count_frames()` helper in the integration test suite (`src-tauri/tests/filter_integration_tests.rs`) to reliably count video frames. The previous implementation queried `nb_read_frames` without the `-count_frames` flag, causing ffprobe to return `N/A` for containers or codecs that don't store the total frame count in the stream header. This caused `test_frame_drop_reduces_frame_count` to panic with `"parse frame count 'N/A': invalid digit found in string"`.

## Changes Made

### Task 1: Fix count_frames helper

**Commit:** `1cb2d4f`

**What changed:**

1. **Added `-count_frames` flag** to the ffprobe command — forces ffprobe to actually decode and count every frame in the video stream, rather than relying on stream header metadata that may be absent.

2. **Switched from CSV to JSON output** — the previous `csv=p=0` format could only return a single value. JSON format enables structured parsing of multiple fields.

3. **Added fallback logic** — tries `nb_read_frames` first (populated when `-count_frames` is used), then falls back to `nb_frames` if the former is `N/A` or absent.

4. **Better error messages** — each parse failure now reports which field and value failed, and the final error clearly states "could not determine frame count" if neither field resolves.

**Key code change:**
- ffprobe command: added `-count_frames`, changed `stream=nb_read_frames` → `stream=nb_read_frames,nb_frames`, changed `-of csv=p=0` → `-of json`
- JSON parsing: extract `streams[0]`, try `nb_read_frames` → try `nb_frames` → return error

## Verification

```bash
cargo test --test filter_integration_tests test_frame_drop_reduces_frame_count -- --test-threads=1
```

**Result:** ✅ 1 passed, 0 failed. Test correctly counts source frames (~58 from 2s @ 30fps), verifies FrameDrop filter reduces frame count with `-vsync vfr`, and confirms output frame count is within expected range.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Self-Check: PASSED

- File exists: `src-tauri/tests/filter_integration_tests.rs` ✅
- Commit exists: `1cb2d4f` ✅
- Test passes: `test_frame_drop_reduces_frame_count` ✅
