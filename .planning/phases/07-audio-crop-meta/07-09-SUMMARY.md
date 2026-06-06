---
phase: 07-audio-crop-meta
plan: 09
subsystem: ffmpeg-filters
tags: [rust, ffmpeg, audio, channel-map, bug-fix, gap-closure]

# Dependency graph
requires:
  - phase: 07-03
    provides: build_audio_channel_filter function in filters.rs
  - phase: 07-08
    provides: Integration test infrastructure (generate_test_video, run_filter_on_test_video)
provides:
  - Fixed build_audio_channel_filter with aformat normalization for mono/stereo input compatibility
  - Integration test covering all three AudioChannel modes (swap, mono, stereo)
affects: [07-verify]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "aformat filter prefix normalizes input channel layout before downstream channel filters"
    - "channelmap requires all output channels explicitly mapped (N mappings for N-channel layout)"
    - "Integration test source upgraded to stereo audio for realistic channel-map testing"

key-files:
  modified:
    - src-tauri/src/ffmpeg/filters.rs (build_audio_channel_filter — aformat normalization)
    - src-tauri/tests/filter_integration_tests.rs (stereo test source + all-mode loop)

key-decisions:
  - "stereo mode uses channelmap FL-FL|FR-FR|FL-LFE:channel_layout=2.1 to expand stereo to 2.1 layout (3 explicit channel mappings)"
  - "mono mode replaced pan mixdown with aformat=channel_layouts=mono for FFmpeg-native downmix"
  - "Integration test cache filename changed to test_source_v2.mp4 to force regeneration with stereo audio"

requirements-completed: [D-02]

# Metrics
duration: 15 min
completed: 2026-06-06
---

# Phase 7 Plan 9: Fix AudioChannel Filter Builder

**Fixed build_audio_channel_filter to produce valid FFmpeg output for both mono and stereo input sources, and fixed the integration test to exercise all three modes.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-06-06
- **Completed:** 2026-06-06
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `aformat=channel_layouts=stereo` normalization prefix to "swap" and "stereo" modes, ensuring downstream channel filters receive predictable channel layouts regardless of input source
- Replaced `pan=mono|c0=0.5*FL+0.5*FR` with `aformat=channel_layouts=mono` for FFmpeg-native mono downmix
- Fixed "stereo" mode: channelmap requires all output channels explicitly mapped — uses `map=FL-FL|FR-FR|FL-LFE:channel_layout=2.1` (3 mappings for 3 output channels)
- Upgraded integration test source generator to produce stereo audio (`sine=...,aformat=channel_layouts=stereo`)
- Integration test now loops over all three modes (swap, mono, stereo) instead of testing only swap
- All 30 lib tests + integration test pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix build_audio_channel_filter** - `85a30f5` (fix)
2. **Task 2: Fix integration test** - `e4f9c67` (fix)

## Files Modified

- `src-tauri/src/ffmpeg/filters.rs` — build_audio_channel_filter: aformat normalization + stereo 2.1 mapping
- `src-tauri/tests/filter_integration_tests.rs` — stereo test source + all-mode loop in test_audio_channel_produces_valid_output

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] channelmap=map=FL-FC|FR-FC:channel_layout=2.1 rejected by FFmpeg**
- **Found during:** Task 2 verification
- **Issue:** 2.1 layout (FL+FR+LFE) has 3 channels but channelmap only mapped 2; also map targeted FC which doesn't exist in 2.1. First fix attempt (channel_layout=3.0) also failed: "3.0 does not match the number of channels mapped 2."
- **Fix:** Changed to `channelmap=map=FL-FL|FR-FR|FL-LFE:channel_layout=2.1` — explicitly maps all 3 output channels (FL→FL, FR→FR, FL→LFE) for a working 2.1 expansion
- **Files modified:** filters.rs
- **Commit:** e4f9c67
