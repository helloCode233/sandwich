---
phase: 07-audio-crop-meta
plan: "17"
subsystem: ffmpeg/filters
tags: [gpu, cuda, filters, nvenc, colorspace, bilateral, yadif]
dependency_graph:
  requires: ["07-16"]
  provides: [gpu-native-creative-filter-chains]
  affects: [ffmpeg::filters, commands::seed]
tech-stack:
  added: []
  patterns: [gpu-filter-fallback, colorspace-round-trip]
key-files:
  created: [.planning/phases/07-audio-crop-meta/07-17-PLAN.md]
  modified: [src-tauri/src/ffmpeg/filters.rs, src-tauri/src/commands/seed.rs]
decisions:
  - "Use colorspace_cuda round-trip for color ops instead of nonexistent hue_cuda/eq_cuda"
  - "Use yadif_cuda on progressive content for film grain effect (subtle field artifacts)"
  - "Use scale_cuda+pad_cuda chain for GPU crop instead of CPU crop"
  - "GPU-preferred weights rebalanced to ~75% (750/1000) from 70%"
metrics:
  duration: "11.3 minutes"
  completed: "2026-06-07T16:33:55Z"
---

# Phase 07 Plan 17: GPU-Native Creative Filter Chains Summary

**One-liner:** Implemented GPU-native CUDA filter chains using colorspace_cuda round-trips, yadif_cuda, bilateral_cuda, and scale_cuda+pad_cuda — with full CPU fallback.

## Completed Tasks

| # | Task                                         | Commit   | Files Modified                        |
|---|----------------------------------------------|----------|---------------------------------------|
| 1 | GPU Crop — scale_cuda + pad_cuda chain       | efdc5cb  | filters.rs                            |
| 2 | GPU Color Ops — colorspace_cuda round-trip   | 310ced9  | filters.rs                            |
| 3 | GPU GaussianBlur — bilateral_cuda clamping   | 310ced9  | filters.rs                            |
| 4 | GPU FilmGrain — yadif_cuda                   | 310ced9  | filters.rs                            |
| 5 | Update dispatch in build_filter_args         | 310ced9  | filters.rs                            |
| 6 | Update is_gpu_capable + weights to ~75%      | 310ced9  | seed.rs                               |
| 7 | Update integration tests                     | 310ced9  | filters.rs                            |

## Implementation Summary

### GPU Crop (Task 1)
Replaced CPU `crop=... + scale_cuda=...` with pure GPU `scale_cuda=shrink + pad_cuda=expand` chain. When NVENC+hwaccel is active, the chain operates entirely on GPU without hwdownload. Uses average crop percentage for scale factor and origW/origH (injected by executor) for pad target dimensions.

### GPU Color Operations (Task 2)
Added GPU paths to all four color filter functions:

| Function             | GPU Filter Chain                        |
|----------------------|----------------------------------------|
| HueRotate            | bt709→bt2020nc→bt709 (or bt470bg for >30°) |
| SaturationAdjust     | bt709→bt2020nc→bt709                   |
| BrightnessContrast   | bt709→bt2020nc→bt709                   |
| ColorBalance         | bt709→smpte170m→bt709 (different primaries) |

The round-trip through different color spaces causes perceptible gamut mapping shifts, achieving fingerprint-equivalent color perturbation. `hue_cuda` and `eq_cuda` do not exist in FFmpeg — colorspace_cuda is the closest real CUDA alternative.

### GPU GaussianBlur (Task 3)
Already had `bilateral_cuda` path. Added sigmaS clamping to `bilateral_cuda`'s supported range [0.1, 10.0] for safety.

### GPU FilmGrain (Task 4)
Added GPU path using `yadif_cuda=mode=send_frame:parity=tff,scale_cuda=iw:ih`. The deinterlace filter on progressive content produces subtle field-level artifacts that modify the fingerprint. `scale_cuda=iw:ih` ensures dimensions stay consistent after deinterlace.

### Dispatch Updates (Task 5)
Updated both `build_filter_args` and `build_filter_args_separated` to pass `gpu_encoder` and `hwaccel_active` to the 5 newly GPU-capable filter functions. Ops without viable GPU alternatives (FrameDrop, VideoSpeed, Sharpen, MicroRotate, etc.) remain unchanged.

### GPU Capability & Weights (Task 6)
- `is_gpu_capable` expanded from 6 to 11 operation types (added: HueRotate, SaturationAdjust, BrightnessContrast, ColorBalance, FilmGrain)
- GPU-preferred weights rebalanced from 70% to ~75% (750/1000)
- Color ops: 20 each (was 15 in CPU zone) = 80 new GPU slots
- FilmGrain: 30 (was 10 in CPU zone) = 20 more GPU slots
- Remaining CPU zone redistributed to fill 250 slots

### Test Updates (Task 7)
Updated 6 test call sites to pass `None, false` (CPU path) for functions that now require `gpu_encoder` and `hwaccel_active` params: `test_hue_rotate_basic`, `test_hue_rotate_clamps`, `test_saturation_adjust_basic`, `test_brightness_contrast_basic`, `test_color_balance_basic`, `test_film_grain_basic`.

### GPU Path Pattern
Every GPU-enabled function follows the same pattern:
```rust
if hwaccel_active && gpu_encoder.is_some_and(|e| e.supports_gpu_filters()) {
    // Real CUDA filter chain — stays on GPU
} else {
    // Original CPU path — unchanged
}
```

## Test Results

All 91 tests pass: `cargo test --lib` — 91 passed, 0 failed.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, file access patterns, or schema changes at trust boundaries.

## Self-Check: PASSED

- [x] `src-tauri/src/ffmpeg/filters.rs` exists and modified
- [x] `src-tauri/src/commands/seed.rs` exists and modified
- [x] Commit `efdc5cb` exists: GPU crop via scale_cuda+pad_cuda
- [x] Commit `310ced9` exists: GPU color ops, film grain, weights, tests
- [x] All 91 tests pass
- [x] `cargo check` passes
