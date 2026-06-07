---
phase: 07
plan: 15
subsystem: seed-generation
tags: [gpu, seed, operation-picker, nvenc]
dependency-graph:
  requires: [07-14]
  provides: [gpu-preferred-seed-generation]
  affects: [generate_seed, batch-processing]
tech-stack:
  added: []
  patterns: [weighted-random-selection, gpu-aware-dispatch]
key-files:
  created:
    - .planning/phases/07-audio-crop-meta/07-15-PLAN.md
  modified:
    - src-tauri/src/commands/seed.rs
decisions:
  - "D-15: GPU-preferred seed generation uses 700/1000 bias for GPU-capable ops when GPU encoder detected"
  - "is_gpu_capable identifies 10 operation types: Crop, FrameDrop, VideoSpeed, HueRotate, SaturationAdjust, BrightnessContrast, ColorBalance, GaussianBlur, Sharpen, TinyScale"
  - "pick_operation_type_gpu_preferred maintains proportional distribution within GPU and CPU groups"
metrics:
  duration: ~4 minutes
  completed: 2026-06-07
---

# Phase 07 Plan 15: GPU-Preferred Seed Generation Summary

**One-liner:** Seed generation biases toward GPU-capable operations (700/1000 weight) when any GPU encoder is detected, with CPU-only ops at reduced 300/1000 weight.

## What Was Built

When an NVENC (or any GPU encoder) is detected by the FFmpeg probe at startup and stored in `AppState.gpu_encoder`, the `generate_seed` command now uses a GPU-biased operation picker instead of the uniform CPU-default picker. This ensures seeds built for GPU-accelerated batch processing favor operation types that benefit from GPU-side FFmpeg filters.

### Changes

1. **`is_gpu_capable()` helper** — Identifies 10 operation types that have GPU-side FFmpeg filter equivalents (color processing, blur/sharpen, scale, crop, frame-drop, video-speed).

2. **`pick_operation_type_gpu_preferred()`** — New weighted random picker with 700/1000 probability for GPU-capable ops and 300/1000 for CPU-only ops. Within each group, weights maintain proportional distribution matching the original CPU-default picker's relative weights.

3. **`generate_seed` GPU detection** — Before the random operation loop, reads `state.gpu_encoder` from `AppState`. If any GPU encoder is detected (`is_some()`), dispatches to `pick_operation_type_gpu_preferred`; otherwise uses the existing `pick_operation_type`.

4. **Test coverage** — Added `pick_operation_type_gpu_preferred_covers_all_active_types` that verifies the GPU-preferred picker produces all 28 active operation types (same as CPU-default, excluding deprecated AudioTweak and Flip).

## Weight Distribution

| Group | Operations | Weight | % of Total |
|-------|-----------|--------|------------|
| Color processing | HueRotate, SatAdjust, BrightContrast, ColorBalance | 90 each (360) | 36% |
| Noise/blur | GaussianBlur, Sharpen, TinyScale | 90 each (270) | 27% |
| Duration | VideoSpeed | 40 | 4% |
| Default | Crop, FrameDrop | 15 each (30) | 3% |
| **GPU subtotal** | | **700** | **70%** |
| Math overlay | MathOverlay ×3 | 20 each (60) | 6% |
| Blend overlay | SolidColor, Gradient, Watermark | 20 each (60) | 6% |
| Audio | 5 types | 10 each (50) | 5% |
| Trim | TrimEdges | 25 | 2.5% |
| Other CPU | PixelShift, FilmGrain, MicroRotate, GopModify, MetadataErase, MetadataWrite, MetaSelErase, Remux | 15/10 each (105) | 10.5% |
| **CPU subtotal** | | **300** | **30%** |
| **Total** | | **1000** | **100%** |

## Verification

- `cargo check` — Clean (no warnings)
- `cargo fmt` — Passed
- `cargo test --lib` — **91 passed, 0 failed**, including:
  - `pick_operation_type_gpu_preferred_covers_all_active_types` ✅
  - `pick_operation_type_covers_all_active_types` ✅
  - `generate_100_seeds_verify_pre_injected_structure` ✅
  - All existing seed, batch, ffmpeg, import, queue, migration, model tests ✅

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- [x] `src-tauri/src/commands/seed.rs` exists and contains `is_gpu_capable`, `pick_operation_type_gpu_preferred`, and GPU-aware `generate_seed`
- [x] `07-15-PLAN.md` created at `.planning/phases/07-audio-crop-meta/`
- [x] Commit `06b985a` exists: `feat(07-15): add GPU-preferred seed generation with reweighted operation picker`
- [x] All 91 tests pass
