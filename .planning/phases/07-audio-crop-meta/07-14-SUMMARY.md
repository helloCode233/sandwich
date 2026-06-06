---
phase: "07"
plan: "14"
subsystem: "ffmpeg-gpu-filters"
tags: [gpu, cuda, nvenc, filters, performance]
requires: ["07-13"]
provides: ["GPU-native filter chains"]
affects: ["ffmpeg/filters.rs", "ffmpeg/executor.rs"]
tech-stack:
  added: ["ffmpeg CUDA filters: fps_cuda, hflip_cuda, transpose_cuda, hue_cuda, eq_cuda, gblur_cuda, unsharp_cuda, scale_cuda"]
  patterns: ["GPU filter dispatch via gpu_encoder + hwaccel_active params"]
key-files:
  created: [".planning/phases/07-audio-crop-meta/07-14-PLAN.md"]
  modified: ["src-tauri/src/ffmpeg/filters.rs", "src-tauri/tests/filter_integration_tests.rs"]
decisions:
  - "D-01: GPU filter path uses supports_gpu_filters() gate (NVENC only)"
  - "D-02: Default source fps 30.0 for frame drop and video speed calculations"
  - "D-03: Color balance approximated via eq_cuda brightness (no colorbalance_cuda exists)"
  - "D-04: Overlay filters (colorize, geq) have no CUDA equivalents — CPU-only"
metrics:
  duration: "37 minutes"
  tasks: 9
  files_changed: 2
  lines: "+318 / -93"
  completed: "2026-06-06"
---

# Phase 07 Plan 14: GPU CUDA Filter Equivalents Summary

Converted 7 CPU video filter builders to NVIDIA GPU (CUDA) equivalents, enabling
end-to-end GPU pipeline when NVENC + hardware decode are active. The filter chain
now stays in GPU memory (zero-copy) when possible, avoiding CPU↔GPU round-trips.

## Results

| Filter Builder | CPU Filter | GPU Filter | Status |
|---|---|---|---|
| FrameDrop | select + setpts | fps_cuda + setpts | ✅ Direct |
| VideoSpeed | setpts (video) | fps_cuda + setpts | ✅ Direct |
| Flip (horizontal) | hflip | hflip_cuda | ✅ Direct |
| Flip (vertical) | vflip | transpose_cuda×2 + hflip_cuda | ✅ Approx |
| Flip (both) | hflip+vflip | transpose_cuda×2 | ✅ Approx |
| HueRotate | hue | hue_cuda | ✅ Direct |
| SaturationAdjust | eq | eq_cuda | ✅ Direct |
| BrightnessContrast | eq | eq_cuda | ✅ Direct |
| ColorBalance | colorbalance | eq_cuda brightness | ✅ Approx |
| GaussianBlur | gblur | gblur_cuda | ✅ Direct |
| Sharpen | unsharp | unsharp_cuda | ✅ Direct |
| TinyScale | scale | scale_cuda | ✅ Direct |
| SolidColorOverlay | colorize | (CPU only) | ⚠️ No CUDA equiv |
| GradientOverlay | geq | (CPU only) | ⚠️ No CUDA equiv |
| WatermarkBlend | geq | (CPU only) | ⚠️ No CUDA equiv |

## Deviations from Plan

### Auto-fixed Issues

None — plan executed as written. Minor formatting fixes applied (cargo fmt).

### Intentional Deviations

**1. [Architecture] Overlay ops have no CUDA equivalents**
- **Found during:** Task 7
- **Issue:** Plan assumed solid_color_overlay, gradient_overlay, and watermark_blend
  used the `overlay` filter (which has `overlay_cuda`). In reality, these use
  `colorize` and `geq` filters which have no CUDA equivalents in FFmpeg.
- **Decision:** Added GPU params to maintain API consistency (all filter builders
  now accept `gpu_encoder` and `hwaccel_active`), but GPU path uses CPU filters.
  The executor's hwdownload handling will still move frames to CPU for these ops.
- **Future:** migrate solid_color_overlay to overlay-based approach with overlay_cuda
  in a future plan if CUDA performance for these ops is needed.

## Deferred Issues

None.

## Known Stubs

| File | Line | Description |
|---|---|---|
| filters.rs | FrameDrop GPU path | Default fps 30.0 hardcoded — source fps not available at filter-build time |
| filters.rs | VideoSpeed GPU path | Default fps 30.0 hardcoded — source fps not available at filter-build time |
| filters.rs | ColorBalance GPU path | eq_cuda brightness approximation — per-channel color balance lost on GPU |

These stubs are intentional:
- Source fps is not in Operation params (available only at executor runtime via ffprobe)
- ColorBalance approximation is documented as acceptable per plan spec ("OK with approximate effects")
- Future improvement: inject source_fps into Operation params at runtime (like origW/origH for Crop)

## Commits

| Hash | Message |
|---|---|
| 447eefb | feat(07-14): GPU FrameDrop — fps_cuda replaces select for NVENC+hwaccel |
| 81b6dff | feat(07-14): GPU VideoSpeed — fps_cuda replaces setpts for NVENC+hwaccel |
| 698edb9 | feat(07-14): GPU Flip — hflip_cuda and transpose_cuda for NVENC+hwaccel |
| 3b9c5fa | feat(07-14): GPU Color ops — hue_cuda and eq_cuda for NVENC+hwaccel |
| ebbd45b | feat(07-14): GPU Blur and Sharpen — gblur_cuda and unsharp_cuda for NVENC+hwaccel |
| 594f1f5 | feat(07-14): GPU TinyScale — scale_cuda for NVENC+hwaccel |
| 4dc1f8a | feat(07-14): GPU Overlay ops — add GPU params (no CUDA equivalents exist yet) |

## Self-Check: PASSED

- [x] `src-tauri/src/ffmpeg/filters.rs` exists and modified
- [x] `src-tauri/tests/filter_integration_tests.rs` updated
- [x] All 102 tests pass (90 unit + 12 integration)
- [x] All 7 commits verified in git log
- [x] `cargo check` passes
- [x] `cargo fmt` passes
