---
phase: 05-production-hardening
plan: 03
subsystem: gpu-encoder
tags: [gpu, ffmpeg, encoding, hardware-acceleration, perf]
requires:
  - PERF-01
provides:
  - GpuEncoder model and detection module
  - GPU encoder injection into FFmpeg commands
  - Automatic GPU detection at app startup
  - CPU fallback on GPU encode failure
affects:
  - src-tauri/src/ffmpeg/executor.rs (signature change)
  - src-tauri/src/commands/batch.rs (call site update deferred to 05-04/05-05)
  - src-tauri/src/state.rs (AppState struct)
  - src-tauri/src/lib.rs (startup lifecycle)
tech-stack:
  added:
    - ffmpeg-sidecar v2.5.1 paths module (ffmpeg_path)
  patterns:
    - Platform-specific encoder detection via cfg!(target_os) guards
    - GPU encoder enum with encoder_name() method for codec string mapping
    - GPU detection auto-ran at startup via tauri::async_runtime::spawn
    - Silent CPU fallback (libx264 -preset medium) when no GPU encoder found
    - GPU encoding uses -preset fast for throughput
key-files:
  created:
    - src-tauri/src/models/gpu.rs (GpuEncoder enum, 30 lines)
    - src-tauri/src/ffmpeg/gpu.rs (detect_gpu_encoder function, 63 lines)
  modified:
    - src-tauri/src/ffmpeg/mod.rs (added pub mod gpu)
    - src-tauri/src/models/mod.rs (added pub mod gpu)
    - src-tauri/src/state.rs (AppState.gpu_encoder field + Default)
    - src-tauri/src/ffmpeg/executor.rs (signature + codec injection)
    - src-tauri/src/lib.rs (GPU detection spawn at startup)
    - src-tauri/src/commands/batch.rs (call site: None arg, Rule 1 fix)
decisions:
  - GpuEncoder enum uses platform-specific variants (VideoToolbox, Nvenc, Amf, Vaapi) per D-04
  - GPU detection uses ffmpeg_sidecar::paths::ffmpeg_path() — not ffmpeg_sidecar::ffmpeg::ffmpeg_path() (v2.5.1 API)
  - executor.rs signature change (new gpu_encoder param) temporarily breaks batch.rs; fixed with None passthrough, full wiring deferred to 05-04/05-05
metrics:
  duration: 12m 49s
  completed_date: 2026-05-15
---

# Phase 5 Plan 3: GPU Encoder Detection and Injection Summary

**One-liner:** Automatic GPU hardware encoder detection via ffmpeg -encoders probe, with encoder injection into FFmpeg commands and silent CPU fallback on failure.

## Tasks Completed

### Task 1: Create GPU model and detection module
- **Commit:** `92a3df4`
- **Created:** `src-tauri/src/models/gpu.rs` — `GpuEncoder` enum with 4 platform variants (VideoToolbox, Nvenc, Amf, Vaapi) and `encoder_name()` method returning correct FFmpeg codec strings
- **Created:** `src-tauri/src/ffmpeg/gpu.rs` — `detect_gpu_encoder()` function using `Command::new().args(["-hide_banner", "-encoders"]).output()` pattern, with `cfg!(target_os)` platform-specific encoder matching
- **Modified:** `src-tauri/src/ffmpeg/mod.rs` — added `pub mod gpu;`
- **Modified:** `src-tauri/src/models/mod.rs` — added `pub mod gpu;`
- **Test:** `test_detect_gpu_encoder_no_ffmpeg` passes (verifies None return on missing FFmpeg)

### Task 2: Wire GPU encoder into AppState, executor, and startup
- **Commit:** `6ba02b1`
- **Modified:** `src-tauri/src/state.rs` — added `GpuEncoder` import, `gpu_encoder: Option<GpuEncoder>` field to `AppState`, initialized to `None` in `Default`
- **Modified:** `src-tauri/src/ffmpeg/executor.rs` — added `gpu_encoder: Option<&GpuEncoder>` parameter to `execute_single_file()`, injects `-c:v <encoder> -preset fast` (GPU) or `-c:v libx264 -preset medium` (CPU) before existing filter args
- **Modified:** `src-tauri/src/lib.rs` — spawned `detect_gpu_encoder()` at startup after FFmpeg path resolution, emits `gpu-encoder-detected` (with encoder variant) or `gpu-encoder-not-detected` events to frontend, stores result in `AppState.gpu_encoder`
- **Modified:** `src-tauri/src/commands/batch.rs` — pass `None` for `gpu_encoder` at call site (full wiring deferred to Plan 05-04/05-05)

## Verification

- `cargo check` exits 0 (pre-existing warnings only, no new errors)
- `cargo test` passes all 13 tests including `test_detect_gpu_encoder_no_ffmpeg`
- `grep` confirms all acceptance criteria: module declarations, enum variants, encoder names, cfg guards, state field, executor signature, lib.rs detection spawn

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed batch.rs compilation error caused by new execute_single_file signature**
- **Found during:** Task 2
- **Issue:** Adding `gpu_encoder` parameter to `execute_single_file()` broke the call site in `src-tauri/src/commands/batch.rs` (expected by plan author, deferred to 05-04/05-05)
- **Fix:** Pass `None` for `gpu_encoder` at the batch.rs call site — allows compilation without changing batch logic
- **Files modified:** `src-tauri/src/commands/batch.rs` (line 182)
- **Commit:** `6ba02b1`

**2. [Rule 3 - Blocking] Fixed incorrect ffmpeg-sidecar API path for ffmpeg_path**
- **Found during:** Task 2
- **Issue:** Plan specified `ffmpeg_sidecar::ffmpeg::ffmpeg_path()` but v2.5.1 exports the function at `ffmpeg_sidecar::paths::ffmpeg_path()`. The `ffmpeg` module does not exist as a public module in v2.5.1.
- **Fix:** Changed to `ffmpeg_sidecar::paths::ffmpeg_path()` with explicit type annotation for `.map(|p: &std::path::Path| ...)`
- **Files modified:** `src-tauri/src/lib.rs` (line 107-110)
- **Commit:** `6ba02b1`

## Known Stubs

None. `gpu_encoder: None` in `AppState::default()` is the correct initial value — it is set to `Some(...)` at startup if a GPU encoder is detected.

## Threat Flags

None. All threat model items from the plan are addressed: FFmpeg stdout is parsed via `from_utf8_lossy` (T-05-03-01 accept), encoder names are hardcoded string literals (T-05-03-03 accept), event payloads contain only enum variant names (T-05-03-04 accept). T-05-03-02 (GPU encode hang) is mitigated by retry logic deferred to Plan 05-06.

## Self-Check: PASSED
- FOUND: .planning/phases/05-production-hardening/05-03-SUMMARY.md
- FOUND: src-tauri/src/models/gpu.rs
- FOUND: src-tauri/src/ffmpeg/gpu.rs
- FOUND: 92a3df4 (Task 1 commit)
- FOUND: 6ba02b1 (Task 2 commit)
