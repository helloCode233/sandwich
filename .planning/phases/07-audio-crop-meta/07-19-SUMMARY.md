---
phase: 07
plan: 19
subsystem: ffmpeg-pipeline
tags: [gpu, fix, colorspace_cuda, color-ops]
requires: [07-17, 07-18]
provides: [cpu-only-color-ops, valid-filter-paths]
affects: [seed-generation, ffmpeg-filters]
tech-stack:
  added: []
  patterns: [cpu-only-color-processing]
key-files:
  created: []
  modified:
    - src-tauri/src/ffmpeg/filters.rs
    - src-tauri/src/commands/seed.rs
decisions:
  - Removed all colorspace_cuda usage — filter only supports `range` (tv/pc), not `space`, `trc`, `primaries`, `iall`, or `all` params
  - 4 color operations (HueRotate, SaturationAdjust, BrightnessContrast, ColorBalance) are CPU-only; GPU acceleration from hwaccel decode + NVENC encode
  - BuildHueRotateFilter, BuildSaturationAdjustFilter, BuildBrightnessContrastFilter, BuildColorBalanceFilter now take only `op: &Operation` (no GPU params)
  - Removed HueRotate/SaturationAdjust/BrightnessContrast/ColorBalance from is_gpu_capable
  - Redistributed pick_operation_type_gpu_preferred: GPU zone 670 weight (was 750), 4 color ops moved to CPU zone with 20 each
metrics:
  duration: ~4min
  completed: 2026-06-07T19:00:00Z
---

# Phase 7 Plan 19: Fix broken GPU filter paths — remove colorspace_cuda usage Summary

**One-liner:** Removed broken `colorspace_cuda=iall=...:all=...` GPU paths from 4 color operations; all color ops are now CPU-only.

## What Was Done

The `colorspace_cuda` FFmpeg filter ONLY supports a `range` option (tv/pc). It does NOT support color space conversion parameters like `space`, `trc`, `primaries`, `iall`, or `all`. The GPU paths added in Plan 07-17 for 4 color operations used invalid `colorspace_cuda=iall=...:all=...` syntax that fails at runtime, causing silent CPU fallback.

### Changes

1. **`build_hue_rotate_filter`** — Removed `gpu_encoder`/`hwaccel_active` params; GPU path removed; signature simplified to `fn(op: &Operation)`.
2. **`build_saturation_adjust_filter`** — Same treatment; CPU-only `eq` filter.
3. **`build_brightness_contrast_filter`** — Same treatment; CPU-only `eq` filter.
4. **`build_color_balance_filter`** — Same treatment; CPU-only `colorbalance` filter.
5. **`build_filter_args` dispatch** — Updated 4 color op arms to call without GPU params.
6. **`build_filter_args_separated` dispatch** — Updated 4 color op arms to call without GPU params.
7. **`is_gpu_capable`** — Removed 4 color op types (HueRotate, SaturationAdjust, BrightnessContrast, ColorBalance).
8. **`pick_operation_type_gpu_preferred`** — Redistributed weights: GPU zone 1..=670 (was 750), 4 color ops moved to CPU zone 921..=1000 (20 each).

### Working GPU Filters Preserved

These verified-against-actual-FFmpeg-binary GPU paths remain intact:
- `bilateral_cuda=sigmaS=S:sigmaR=R` — `build_gaussian_blur_filter` ✓
- `yadif_cuda=mode=send_frame:parity=tff` — `build_film_grain_filter` ✓
- `scale_cuda=W:H` + `pad_cuda=w=W:h=H:x=X:y=Y` — `build_crop_filter` ✓
- `scale_cuda=W:H` — `build_tiny_scale_filter` ✓
- `transpose_cuda=clock` — `build_flip_filter` ✓

### Updated GPU Weight Distribution

| Zone | Before | After |
|------|--------|-------|
| GPU-capable ops | 750/1000 (75%) | 670/1000 (67%) |
| CPU-only ops | 250/1000 (25%) | 330/1000 (33%) |

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None.

## Verification

- `cargo check` — passed with no errors
- `cargo fmt src-tauri` — passed
- `cargo test --lib` — all 91 tests passed

## Commit

`16fc508` — `fix(07-19): remove broken colorspace_cuda GPU paths from 4 color operations`

## Self-Check

- [x] SUMMARY.md created at `.planning/phases/07-audio-crop-meta/07-19-SUMMARY.md`
- [x] Commit `16fc508` exists in git history
- [x] Modified files exist: `src-tauri/src/ffmpeg/filters.rs`, `src-tauri/src/commands/seed.rs`
- [x] All 91 tests pass
- [x] No file deletions in commit
