---
status: complete
phase: 07-audio-crop-meta
source: [07-VERIFICATION.md]
started: 2026-05-18T21:30:00+08:00
updated: 2026-06-06T12:00:00+08:00
---

## Current Test

[testing complete]

## Tests

### 1. Filter builder correctness
expected: Run each of the 10 new filter builders against actual FFmpeg with test media. Verify that AudioResample, AudioVolume, AudioPitch, AudioEQ, AudioChannel, Crop, MetadataWrite, MetadataSelectiveErase, VideoSpeed, and TrimEdges each produce valid FFmpeg filter arguments that execute without error.
result: issue
reported: "cargo test --test filter_integration_tests: 9/12 passed, 3 failed. test_audio_channel_produces_valid_output: AudioChannel filter should produce valid output. test_metadata_write_injects_fake_fields: title metadata should be injected (left: None, right: Some(\"Integration Test\")). test_frame_drop_reduces_frame_count: parse frame count 'N/A': invalid digit found in string"
severity: major

### 2. Pre-injection behavioral verification
expected: Generate 100 seeds. Verify each seed contains at least one Crop operation and at least one FrameDrop operation (pre-injected defaults per D-04, D-19).
result: pass

### 3. FrameDrop select filter visual verification
expected: Process a test video with FrameDrop applied. Confirm frames are actually dropped (frame count decreases), not duplicated (which would happen if -vsync cfr overrides the select filter without -vsync vfr).
result: issue
reported: "cargo test test_frame_drop_reduces_frame_count: FAILED. parse frame count 'N/A': invalid digit found in string. ffprobe returned 'N/A' for nb_frames."
severity: major

### 4. Migration integration test
expected: Create a Phase 6 seed file with AudioTweak operations and old FrameDrop (setpts) params. Restart the app. Verify AudioTweak is split into AudioVolume/AudioPitch, echo sub-effect is dropped, and FrameDrop is re-parameterized to select-based interval.
result: pass

### 5. GPU filter operations for video
expected: All GPU-capable video operations (Crop, FrameDrop, VideoSpeed, TrimEdges) use GPU filters (crop_cuda, scale_cuda, fps_cuda, etc.) instead of CPU-only filters.
result: issue
reported: "需要将原来使用cpu滤镜实现的操作改为gpu操作" (clarified: 全部可 GPU 化的操作 — video filters use GPU, audio stays CPU)
severity: major

## Summary

total: 5
passed: 2
issues: 3
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "AudioChannel filter produces valid FFmpeg output"
  status: failed
  reason: "test_audio_channel_produces_valid_output panicked: 'AudioChannel filter should produce valid output'"
  severity: major
  test: 1
  artifacts:
    - path: "src-tauri/tests/filter_integration_tests.rs"
      issue: "AudioChannel test at line ~275"
  missing: []
  root_cause: ""
  debug_session: ""

- truth: "MetadataWrite injects fake metadata fields (title, author, etc.) into output video"
  status: failed
  reason: "test_metadata_write_injects_fake_fields: assertion failed — title metadata was None, expected Some(\"Integration Test\")"
  severity: major
  test: 1
  artifacts:
    - path: "src-tauri/tests/filter_integration_tests.rs"
      issue: "MetadataWrite test at line ~336; ffprobe not finding injected metadata"
  missing: []
  root_cause: ""
  debug_session: ""

- truth: "FrameDrop reduces frame count (select filter + -vsync vfr actually drops frames)"
  status: failed
  reason: "test_frame_drop_reduces_frame_count: parse frame count 'N/A' — ffprobe returned 'N/A' for nb_frames stream tag"
  severity: major
  test: 3
  artifacts:
    - path: "src-tauri/tests/filter_integration_tests.rs"
      issue: "Frame count parsing at line ~455; count_frames helper uses nb_frames which can be 'N/A'"
  missing: []
  root_cause: ""
  debug_session: ""

- truth: "Video filter operations use GPU-accelerated filters where available"
  status: failed
  reason: "User reported: 需要将原来使用cpu滤镜实现的操作改为gpu操作 (all GPU-capable video ops)"
  severity: major
  test: 5
  artifacts:
    - path: "src-tauri/src/ffmpeg/filters.rs"
      issue: "Crop, FrameDrop, VideoSpeed, TrimEdges build functions use CPU filters (crop, select, setpts, trim)"
  missing:
    - "Replace CPU video filters with GPU equivalents (crop_cuda, scale_cuda, etc.)"
  root_cause: ""
  debug_session: ""
