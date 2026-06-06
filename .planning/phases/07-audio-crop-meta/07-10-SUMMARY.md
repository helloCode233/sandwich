---
phase: 07-audio-crop-meta
plan: 10
subsystem: ffmpeg/filters
tags: [metadata, ffprobe, mp4, global-scope, gap-closure]
depends_on: []
requires: [probe_global_metadata, build_metadata_write_filter]
provides: [metadata-global-injection]
affects: [src-tauri/src/ffmpeg/filters.rs, src-tauri/src/ffmpeg/probe.rs]
tech-stack:
  added: []
  patterns: [global-metadata-scope, ffprobe-format-only-parsing]
key-files:
  created: []
  modified:
    - src-tauri/src/ffmpeg/filters.rs
    - src-tauri/src/ffmpeg/probe.rs
    - src-tauri/tests/filter_integration_tests.rs
decisions:
  - "use -metadata:g for global/format-level metadata instead of -metadata (stream-level)"
  - "prepend -map_metadata 0 to ensure metadata track exists for lavfi-generated sources"
  - "MP4 muxer silently drops author key and overwrites encoder — test uses MP4-compatible keys (comment, copyright)"
  - "create RawFormatOutput struct for -show_format-only ffprobe JSON (no streams field)"
metrics:
  duration: ~15min
  completed_date: 2026-06-06
---

# Phase 07 Plan 10: Fix MetadataWrite Global Scope Injection Summary

**One-liner:** Fixed MetadataWrite filter to inject metadata at global/format level using `-metadata:g`, and fixed `probe_global_metadata` JSON parsing to correctly read format-only ffprobe output.

## Tasks Completed

| # | Task | Status | Commit |
|---|------|--------|--------|
| 1 | Fix build_metadata_write_filter to emit `-metadata:g` for global scope | Done | `df828a9` |
| 2 | Fix probe_global_metadata JSON parsing and update integration test | Done | `0861793` |

## What Changed

### Task 1: Metadata Write Filter Fix (`filters.rs`)

**Root cause:** `-metadata key=value` (no stream specifier) writes to the default stream scope, but `probe_global_metadata` reads `format.tags` which only contains global/container-level metadata.

**Fix applied:**
1. Changed `-metadata` to `-metadata:g` — the `:g` specifier writes to global/format metadata scope, matching what `probe_global_metadata` reads via `format.tags`.
2. Added `-map_metadata 0` as the first argument — ensures a metadata track exists in the output, which is critical for lavfi-generated sources that start with no metadata.

```rust
// Before: args.push("-metadata".to_string());
// After:
let mut args = vec!["-map_metadata".to_string(), "0".to_string()];
// ...
args.push("-metadata:g".to_string());
```

### Task 2: probe_global_metadata Fix (`probe.rs`) + Integration Test Update

**Root cause:** `probe_global_metadata` runs `ffprobe -show_format` (format-only output), but deserialized the JSON using `RawProbeOutput` which requires a `streams` field. This caused all calls to return `Err("missing field streams")`.

**Fix applied:**
1. Created `RawFormatOutput` struct with only a `format` field (no `streams`), matching the `-show_format` JSON shape.
2. Updated `probe_global_metadata` to use `RawFormatOutput` instead of `RawProbeOutput`.

**Integration test updated:**
- MP4 muxer silently drops the `author` metadata key (use `artist` for MP4 compatibility). Changed test assertions to verify `comment` and `copyright` instead.
- FFmpeg's lavf library always overwrites the `encoder` key with `LavfXX.XX.XXX`. Removed `encoder` assertion.
- Added explanatory comments documenting these format-level limitations.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] RawProbeOutput requires streams field but -show_format output has none**
- **Found during:** Task 2 (integration test verification)
- **Issue:** `probe_global_metadata` was failing with JSON parse error "missing field `streams`" because it used `RawProbeOutput` (which has `streams: Vec<RawStream>`) to deserialize format-only ffprobe output.
- **Fix:** Created dedicated `RawFormatOutput` struct with only `format: RawFormat` field, matching the `-show_format` JSON shape.
- **Files modified:** `src-tauri/src/ffmpeg/probe.rs`
- **Commit:** `0861793`

**2. [Rule 1 - Bug] Integration test assertions used MP4-unsupported metadata keys**
- **Found during:** Task 2 (manual ffprobe debugging)
- **Issue:** The integration test checked for `author` and `encoder` keys, but the MP4 muxer silently drops `author` (not a recognized MP4 atom) and FFmpeg always overwrites `encoder` with its own lavf version string.
- **Fix:** Changed test assertions to use MP4-compatible keys: `comment` and `copyright`. Added explanatory comments documenting format-level limitations.
- **Files modified:** `src-tauri/tests/filter_integration_tests.rs`
- **Commit:** `0861793`

### Plan-Level Adjustments
- **encoder assertion removed:** Plan requested adding `encoder` field verification, but FFmpeg's MP4 muxer unconditionally overwrites the encoder tag. This is upstream FFmpeg behavior, not a filter bug. Documented as known limitation.

## Verification Results

| Test Suite | Result |
|------------|--------|
| Unit tests (87 tests) | 87 passed, 0 failed |
| Integration test: test_metadata_write_injects_fake_fields | PASSED |
| Integration test: all other Phase 7 tests | 10 passed, 1 pre-existing failure (test_frame_drop_reduces_frame_count — FFmpeg 8.1.1 compat) |
| cargo check | PASSED |
| cargo fmt | PASSED |

## Known Stubs

None. All assertions are wired to real metadata values from the filter.

## Threat Flags

None. No new network endpoints, auth paths, or trust boundaries introduced.

## Self-Check: PASSED

- [x] `src-tauri/src/ffmpeg/filters.rs` exists and contains `-metadata:g` + `-map_metadata 0`
- [x] `src-tauri/src/ffmpeg/probe.rs` exists and contains `RawFormatOutput` struct
- [x] Commit `df828a9` exists: `fix(07-10): write metadata at global scope via -metadata:g and -map_metadata 0`
- [x] Commit `0861793` exists: `fix(07-10): fix probe_global_metadata JSON parsing and update integration test`
