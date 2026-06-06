---
phase: 07-audio-crop-meta
plan: 12
subsystem: ffmpeg-filters
tags: [gpu, nvenc, crop, cuda, filters]
dependency_graph:
  requires: []
  provides: [gpu_crop_filter_chain]
  affects: [build_crop_filter, build_filter_args_separated, execute_single_file]
tech_stack:
  added: []
  patterns:
    - "GPU filter dispatch via GpuEncoder::supports_gpu_filters() capability check"
    - "hwupload_cuda → GPU filters → hwdownload,format=nv12 GPU memory management"
key_files:
  created: []
  modified:
    - src-tauri/src/models/gpu.rs
    - src-tauri/src/ffmpeg/filters.rs
    - src-tauri/src/ffmpeg/executor.rs
    - src-tauri/tests/filter_integration_tests.rs
decisions:
  - "Only NVENC gets GPU filter support — other backends (AMF, VAAPI, VideoToolbox) lack reliable FFmpeg filter equivalents"
  - "GPU scale_cuda omits flags=lanczos (not supported); CPU path retains lanczos for quality"
  - "GPU filter chain wraps with hwupload_cuda/hwdownload,format=nv12 for proper CUDA memory management"
metrics:
  duration: "3 tasks"
  completed_date: "2026-06-06"
---

# Phase 7 Plan 12: GPU-Accelerated Crop Filter Chain

GPU-accelerated filter support for the Crop operation: when the GPU encoder is NVENC, the crop filter chain switches from CPU filters (`crop`, `scale`) to CUDA equivalents (`crop_cuda`, `scale_cuda`). FrameDrop, VideoSpeed, and TrimEdges remain on CPU — no FFmpeg GPU equivalents exist for their core filters (select, setpts, trim).

## Completed Tasks

### Task 1 — GpuEncoder::supports_gpu_filters()
**Commit:** `e83e11b`

Added `supports_gpu_filters()` capability method to the `GpuEncoder` enum. Returns `true` only for `Nvenc` (which has `crop_cuda` and `scale_cuda` filter equivalents). Other backends (VideoToolbox, Amf, Vaapi) return `false` — they still benefit from GPU encoding but use CPU filters.

### Task 2 — GPU-aware build_crop_filter
**Commit:** `7bd9d2b`

Modified `build_crop_filter` to accept `gpu_encoder: Option<&GpuEncoder>` parameter. When NVENC is detected:
- Produces `crop_cuda=x:y:w:h,scale_cuda=w:h` chain
- Wraps with `hwupload_cuda` prefix and `hwdownload,format=nv12` suffix for GPU memory management
- Omits `flags=lanczos` (not supported by `scale_cuda`)

When GPU encoder is `None` or non-NVENC: produces the original `crop=...,scale=...:flags=lanczos` CPU chain unchanged.

### Task 3 — Wire GPU encoder through executor
**Commit:** `a5f53a9`

- Updated `build_filter_args_separated` signature to accept `gpu_encoder: Option<&GpuEncoder>`
- Crop match arm passes `gpu_encoder` to `build_crop_filter`
- Executor passes `gpu_encoder` (already in scope) to `build_filter_args_separated`
- All unit tests and integration test calls updated

## Deviations from Plan

None — plan executed exactly as written.

## Verification

```bash
cargo check    # PASS — no compilation errors
cargo fmt      # PASS — all files formatted (nightly-only warnings expected)
cargo test --lib -- --test-threads=1  # PASS — 87/87 tests passing
```

## Known Stubs

None. All GPU filter paths are properly wired and produce valid FFmpeg filter expressions.

## Threat Flags

None. Changes are within existing trust boundaries — GPU filters run in the same FFmpeg child process with same privilege level as CPU filters.

## Self-Check: PASSED

- `src-tauri/src/models/gpu.rs` — FOUND, `supports_gpu_filters()` method present
- `src-tauri/src/ffmpeg/filters.rs` — FOUND, `build_crop_filter` accepts `gpu_encoder` param
- `src-tauri/src/ffmpeg/executor.rs` — FOUND, `build_filter_args_separated` call passes `gpu_encoder`
- Commit `e83e11b` — FOUND (Task 1)
- Commit `7bd9d2b` — FOUND (Task 2)
- Commit `a5f53a9` — FOUND (Task 3)
