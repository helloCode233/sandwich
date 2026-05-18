---
status: partial
phase: 07-audio-crop-meta
source: [07-VERIFICATION.md]
started: 2026-05-18T21:30:00+08:00
updated: 2026-05-18T21:30:00+08:00
---

## Current Test

[awaiting human testing]

## Tests

### 1. Filter builder correctness
expected: Run each of the 10 new filter builders against actual FFmpeg with test media. Verify that AudioResample, AudioVolume, AudioPitch, AudioEQ, AudioChannel, Crop, MetadataWrite, MetadataSelectiveErase, VideoSpeed, and TrimEdges each produce valid FFmpeg filter arguments that execute without error.
result: [pending]

### 2. Pre-injection behavioral verification
expected: Generate 100 seeds. Verify each seed contains at least one Crop operation and at least one FrameDrop operation (pre-injected defaults per D-04, D-19).
result: [pending]

### 3. FrameDrop select filter visual verification
expected: Process a test video with FrameDrop applied. Confirm frames are actually dropped (frame count decreases), not duplicated (which would happen if -vsync cfr overrides the select filter without -vsync vfr).
result: [pending]

### 4. Migration integration test
expected: Create a Phase 6 seed file with AudioTweak operations and old FrameDrop (setpts) params. Restart the app. Verify AudioTweak is split into AudioVolume/AudioPitch, echo sub-effect is dropped, and FrameDrop is re-parameterized to select-based interval.
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
