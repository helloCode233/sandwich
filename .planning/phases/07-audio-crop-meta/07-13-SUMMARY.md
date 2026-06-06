---
phase: 07-audio-crop-meta
plan: 13
subsystem: ffmpeg/gpu
tags: [gpu, nvenc, nvdec, hardware-decode, acceleration, gap-closure]
requires: [07-12]
provides: [gpu-end-to-end-pipeline]
affects: [executor.rs, filters.rs]
tech-stack:
  added: []
  patterns: [hwaccel-aware-filter-chain, hardware-decode-flag-injection]
key-files:
  created:
    - .planning/phases/07-audio-crop-meta/07-13-PLAN.md
    - .planning/phases/07-audio-crop-meta/07-13-SUMMARY.md
  modified:
    - src-tauri/src/ffmpeg/executor.rs
    - src-tauri/src/ffmpeg/filters.rs
decisions:
  - "NVDec hardware decode enabled automatically when NVENC GPU encoder is detected"
  - "CPU video filters receive hwdownload,format=nv12 prefix when hwaccel active to convert GPU→CPU frames"
  - "GPU crop filters skip hwupload_cuda/hwdownload when hwaccel active — frames stay on GPU end-to-end"
  - "FfmpegCommand constructed without .input() to ensure -hwaccel flags appear before -i input"
metrics:
  duration: ~10m
  completed: 2026-06-06T14:29:38Z
---

# Phase 07 Plan 13: GPU Hardware Decode Enablement Summary

**One-liner:** Enabled NVDec hardware decoding for NVENC pipeline — true GPU end-to-end (decode → filter → encode) without CPU round-trips.

## Tasks

| # | Status   | Commit   | Description                                              | Files Modified            |
|---|----------|----------|----------------------------------------------------------|---------------------------|
| 1 | Complete | 6db3769  | Add hardware decode support in executor + filter builders | executor.rs, filters.rs   |
| 2 | Complete | (pending)| Create PLAN.md and SUMMARY.md                             | 07-13-PLAN.md, 07-13-SUMMARY.md |

## Changes

### executor.rs — Hardware decode flag injection + filter chain awareness

- Added `hwaccel_active` detection: true when `gpu_encoder` is `Some(Nvenc(_))`
- Passes `hwaccel_active` flag to `build_filter_args_separated` for GPU-aware filter building
- Inserts `-hwaccel cuda -hwaccel_output_format cuda` before `-i input` using manual arg construction via `FfmpegCommand::args()` (bypasses `.input()` for correct CLI ordering)
- Prepends `hwdownload,format=nv12` to the video filter chain when hwaccel is active and no GPU crop filters (`crop_cuda`) are present — this converts GPU-decoded frames to CPU memory for CPU filter processing
- Updated diagnostic and error messages to reflect hwaccel flags

### filters.rs — hwaccel-aware crop filter builder

- `build_crop_filter` accepts new `hwaccel_active: bool` parameter
- Refactored GPU filter logic: replaced `(crop_name, scale_name, gpu_wrap)` tuple with `use_gpu_filters: bool` flag for clarity
- Three code paths:
  1. GPU filters without hwaccel (existing): `hwupload_cuda,crop_cuda=...,scale_cuda=...,hwdownload,format=nv12`
  2. GPU filters with hwaccel (new): `crop_cuda=...,scale_cuda=...` — frames stay on GPU from decode to encode
  3. CPU filters (unchanged): `crop=...,scale=...:flags=lanczos`
- `build_filter_args_separated` accepts and forwards `hwaccel_active` to `build_crop_filter`
- Legacy `build_filter_args` passes `false` for backward compatibility

## Pipeline Summary

| Path                          | Decode | Filters               | Encode | GPU Zero-Copy |
|-------------------------------|--------|-----------------------|--------|---------------|
| NVENC only (no hwaccel)       | CPU    | CPU/GPU via hwupload  | NVENC  | No            |
| NVENC + hwaccel, CPU filters  | NVDec  | GPU→CPU via hwdownload | NVENC  | Partial       |
| NVENC + hwaccel, GPU filters  | NVDec  | GPU (crop_cuda)       | NVENC  | **Yes**       |

## Deviations from Plan

None — plan executed exactly as written.

## Test Results

```
test result: ok. 87 passed; 0 failed; 0 ignored
```

All 87 existing tests pass unchanged. No new tests added (hwaccel_active defaults to false in all existing test paths).

## Known Stubs

None — hardware decode is fully functional and integrated into the existing GPU detection/encoding pipeline. No placeholder code.

## Threat Flags

None — no new network endpoints, auth paths, or file access patterns. Hardware decode flags are CLI arguments to FFmpeg at the same privilege level as existing FFmpeg execution.
