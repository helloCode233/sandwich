---
phase: 07
plan: 18
subsystem: ffmpeg-pipeline
tags: [gpu, two-pass, pre-injection, schema-version]
requires: [07-17]
provides: [gpu-first-two-pass-execution, cpu-default-ops]
affects: [seed-generation, ffmpeg-executor, seed-schema]
tech-stack:
  added: []
  patterns: [two-pass-ffmpeg-execution, operation-grouping, pass-assembly]
key-files:
  created: []
  modified:
    - src-tauri/src/commands/seed.rs
    - src-tauri/src/ffmpeg/executor.rs
    - src-tauri/src/models/seed.rs
    - src-tauri/src/commands/export_seed.rs
decisions:
  - Pre-inject 6 CPU-only ops (Sharpen, MicroRotate, TrimEdges, MathOverlay, PixelShift, +1 overlay) as conservative-tier defaults in every seed
  - GPU pass compresses intermediate with -cq 28 (higher CRF) for smaller temp files
  - Two-pass only activates when both GPU encoder and GPU-capable ops are present
  - schema_version bumped to 4; old seeds load gracefully without new defaults
metrics:
  duration: ~10min
  completed: 2026-06-07T17:18:00Z
---

# Phase 7 Plan 18: Restructure processing pipeline to GPU-first two-pass with CPU-only ops as defaults Summary

**One-liner:** GPU-first two-pass FFmpeg pipeline with 6 conservative-tier CPU-only ops pre-injected as defaults in every seed, and schema_version bump to 4.

## Tasks Completed

| # | Task | Commit | Summary |
|---|------|--------|---------|
| 1 | Pre-inject CPU-only defaults | `9a4b3a9` | Added Sharpen, MicroRotate, TrimEdges, MathOverlay, PixelShift, and 1 random overlay as Conservative-tier defaults. Updated capacity to step_count + 8. Tests updated. |
| 2+3 | Two-pass GPU-first execution | `6c449e1` | Extracted run_ffmpeg_pass and assemble_pass_args helpers. Split ops into GPU/CPU groups. GPU pass with -cq 28 compression, CPU pass on intermediate. |
| 4+5 | Bump schema_version to 4 | `a115b99` | All new seeds get version 4. Import path uses version 4. Model comment and tests updated. Backward compatible. |

## Key Changes

### seed.rs: Pre-injection (8 defaults total)
- **GPU defaults (2):** Crop + FrameDrop (main tier)
- **CPU defaults (6):** Sharpen, MicroRotate, TrimEdges, MathOverlay, PixelShift (Conservative tier) + 1 random overlay from [SolidColorOverlay, GradientOverlay, WatermarkBlend]
- All 6 CPU ops use `StrengthTier::Conservative` regardless of seed's main tier
- They do NOT count toward step_count

### executor.rs: Two-pass execution
- **`run_ffmpeg_pass()`:** Extracted helper for single FFmpeg pass (spawn, iterate progress, handle cancel, check exit)
- **`assemble_pass_args()`:** Builds consistent argument lists from filter expressions, handles `hwdownload` prefix for GPU decode, FrameDrop `-vsync vfr`, `-c copy` stripping
- **`OpFilterArgs` struct:** Groups video/audio/other filter args per execution pass
- **Two-pass flow:**
  1. If GPU encoder available AND seed has GPU-capable ops: GPU pass first
  2. GPU pass: `hwaccel cuda` decode → GPU-side filters → NVENC encode with `-cq 28` compression → intermediate temp file
  3. CPU pass: decode intermediate → CPU-only filters → final encode → final output
  4. Cleanup intermediate temp file
- **Single-pass fallback:** When no GPU ops or no GPU encoder, merges both groups back into single pass

### Schema version bump
- `schema_version: 4` for newly generated and copied seeds
- Imported seeds get version 4 after migration
- Old seeds (<4) load gracefully — just lack the 6 new defaults
- Migration module (v2→v3) unchanged — produces v3-compatible format

## Deviations from Plan

None — plan executed exactly as written.

## Test Results

```
running 91 tests
test result: ok. 91 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All 91 tests pass, including:
- `pre_injected_ops_have_correct_types_and_order` — updated for 8 ops
- `generate_100_seeds_verify_pre_injected_structure` — extended to validate all CPU defaults
- `seed_strength_tier_round_trip` — updated for version 4
- All filter dispatch tests, migration tests, and model tests unchanged

## Threat Flags

None — no new network endpoints, auth paths, or trust boundaries introduced.
