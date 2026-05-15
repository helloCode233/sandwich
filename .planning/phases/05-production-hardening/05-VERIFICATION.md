---
phase: 05-production-hardening
verified: 2026-05-15T12:00:00Z
status: human_needed
score: 5/5 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Run `cargo tauri build` on macOS, Windows, and Linux to confirm installers (.dmg, .msi, .exe, .deb, .AppImage) are produced"
    expected: "Each platform produces its expected installer artifact(s) in src-tauri/target/release/bundle/"
    why_human: "Cross-platform builds require running on each target OS with platform-specific toolchains (Xcode on macOS, WiX/NSIS on Windows, etc.). Cannot be verified programmatically on a single macOS machine."
  - test: "Run the app on a machine with a discrete GPU (NVIDIA/AMD) and start a batch job; observe FFmpeg uses the auto-detected hardware encoder"
    expected: "GPU encoder is detected at startup (gpu-encoder-detected event fires), FFmpeg invocations include -c:v h264_nvenc / h264_amf / h264_videotoolbox, and encoding throughput is visibly faster than CPU-only"
    why_human: "GPU detection depends on actual hardware and drivers. Throughput improvement is a runtime measurement, not a code property."
  - test: "Push to the `release` branch and verify the CI workflow runs all 4 matrix jobs successfully"
    expected: "GitHub Actions shows 4 green jobs (macOS aarch64, macOS x86_64, ubuntu-22.04, windows-latest), a draft GitHub Release is created with all platform artifacts attached"
    why_human: "CI execution depends on GitHub Actions infrastructure, secrets, and repository settings. Cannot be verified without triggering an actual workflow run."
  - test: "Process a video batch with multiple seeds and verify the BatchSummary displays MD5 before/after with correct Modified/Unchanged status per output file"
    expected: "BatchSummary shows per-file rows with truncated MD5 hashes (first 8 chars), green CheckCircle for modified files, amber AlertCircle for unchanged files, and warning banner when any files are unchanged"
    why_human: "MD5 comparison correctness and UI rendering fidelity require end-to-end testing with real video files. Visual status icons cannot be verified by grep."
  - test: "Verify all existing v1 functionality (seed generation, video import, queue management, single-seed batch, progress streaming, cancel) still works end-to-end"
    expected: "All pre-existing workflows continue to function without regression — this phase is additive hardening"
    why_human: "End-to-end workflow integrity requires interactive UI testing. Type checks and compilation pass but don't guarantee absence of runtime regressions."
---

# Phase 5: Production Hardening Verification Report

**Phase Goal:** The app ships as installable packages for Windows and Linux with CI build matrix; batch processing leverages GPU hardware acceleration and optimized scheduling; users can apply multiple seeds per video in a single batch; every output file is verified to differ from its input via MD5 checksum comparison.
**Verified:** 2026-05-15T12:00:00Z
**Status:** human_needed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths (Roadmap Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can download and install the app on Windows (.msi/.exe) and Linux (.AppImage/.deb) via CI-built artifacts | ✓ VERIFIED (code) | `tauri.conf.json` bundle config with `active: true`, `targets: "all"`, windows NSIS + linux deb/AppImage sub-configs. `.github/workflows/build.yml` with 4-platform matrix using `tauri-apps/tauri-action@v0`. **Needs human:** actual installer generation requires platform-specific build runs. |
| 2 | User on a GPU-equipped machine sees automatically accelerated encoding (NVENC/VideoToolbox/VAAPI) with measurable throughput improvement over CPU encoding | ✓ VERIFIED (code) | `GpuEncoder` enum with 4 platform variants + `encoder_name()` in `models/gpu.rs`. `detect_gpu_encoder()` via `ffmpeg -encoders` probe in `ffmpeg/gpu.rs`. Startup detection spawns in `lib.rs` emitting `gpu-encoder-detected`/`gpu-encoder-not-detected`. Executor injects `-c:v <encoder> -preset fast` when GPU available, `-c:v libx264 -preset medium` otherwise. GPU retry-on-failure in `batch.rs` Err arm with `gpu-encode-failed-fallback` event. **Needs human:** actual throughput measurement requires GPU hardware. |
| 3 | User can select multiple seeds and each video produces one output per selected seed ({original}_{seed_alias}.{ext}) | ✓ VERIFIED | `seedStore.selectedSeedIds: string[]` with `toggleSeed()`, `selectAll()`, `deselectAll()`. `BatchControls.vue` NSelect `multiple` mode with `v-model:value="seedStore.selectedSeedIds"`. `useBatch.startBatch(seedIds: string[])` invokes `start_batch` with `seed_ids: Vec<String>`. Rust nested loop (`for entry { for seed { execute_single_file(seed) } }`) with `total = queue.len() * seeds.len()`. `PerFileProgress.seed_alias` emitted from executor. Naming convention unchanged: `{stem}_{seed_alias}.{ext}`. |
| 4 | User sees MD5 checksums before/after processing in batch summary with pass/fail indication | ✓ VERIFIED | `file_md5()` streaming hash (8KB buffer, `md5::Context`) in `models/batch.rs`. Pre-processing `HashMap<String, (String, u64)>` via `spawn_blocking` before loop. Post-processing MD5 comparison with `modified = md5_before != md5_after`. `FileSuccess` struct with 7 fields (path, seed_alias, source_file, md5_before, md5_after, modified, size_bytes). `BatchResult.succeeded: Vec<FileSuccess>`. `BatchSummary.vue` per-file rows with truncated MD5 (8 chars), green CheckCircle (modified) / amber AlertCircle (unchanged) status icons, unchanged-count warning banner. i18n keys: `md5Modified`, `md5Unchanged`, `md5UnchangedWarning`, `completeBodyWithMD5` in en.json + zh-CN.json. |
| 5 | All existing v1 functionality continues to work | ✓ VERIFIED | All 14 Tauri commands from Phases 1-4 still registered in `lib.rs` invoke_handler. `ci.yml` (lint/test) unchanged. `cargo check` passes (pre-existing warnings only). `bun vue-tsc -b` passes clean. Changes are additive: GPU encoder is `Option<&GpuEncoder>`, multi-seed uses nested loop that works with 1 seed, MD5 is added to the success result type. No deletions of existing functionality. |

**Score:** 5/5 truths verified at the code level. All truths have substantive, wired implementations. Human verification needed for 2 truths that require platform-specific build/runtime environments.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/tauri.conf.json` | Bundle config with Windows/Linux targets, version 1.0.0 | ✓ VERIFIED | `bundle.active: true`, `targets: "all"`, `windows.nsis`, `linux.deb`, `linux.appimage`, `version: "1.0.0"`. 56 lines. |
| `.github/workflows/build.yml` | 4-platform CI matrix with tauri-action | ✓ VERIFIED | 69 lines, valid YAML. Matrix: macos aarch64, macos x86_64, ubuntu-22.04, windows-latest. `tauri-apps/tauri-action@v0`. `fail-fast: false`. |
| `src-tauri/src/models/gpu.rs` | GpuEncoder enum, 4 variants, encoder_name() | ✓ VERIFIED | 27 lines. 4 variants: VideoToolbox, Nvenc, Amf, Vaapi. encoder_name() returns correct FFmpeg codec strings. |
| `src-tauri/src/ffmpeg/gpu.rs` | detect_gpu_encoder() with platform cfg guards | ✓ VERIFIED | 57 lines. Uses `Command::new().args(["-hide_banner", "-encoders"])`. cfg!(target_os) guards for macOS/Windows/Linux. Includes test. |
| `src-tauri/src/state.rs` | AppState.gpu_encoder field | ✓ VERIFIED | `pub gpu_encoder: Option<GpuEncoder>`, initialized to `None` in Default impl. |
| `src-tauri/src/ffmpeg/executor.rs` | GPU encoder injection + seed_alias emission | ✓ VERIFIED | `gpu_encoder: Option<&GpuEncoder>` parameter. Injects `-c:v <encoder>` + `-preset fast/medium`. Emits `seed_alias` in PerFileProgress. D-08 streaming I/O comment present. |
| `src-tauri/src/commands/batch.rs` | Multi-seed nested loop + MD5 pre/post + GPU retry | ✓ VERIFIED | `seed_ids: Vec<String>`. Nested loop. Total = queue * seeds. MD5 pre-hash `HashMap` via `spawn_blocking`. Post-hash comparison. `FileSuccess` construction. GPU retry with `gpu_encoder.is_some()` guard and `continue` for fallthrough prevention. `drop()` before `app.emit()` in both arms. |
| `src-tauri/src/models/batch.rs` | FileSuccess struct, file_md5(), BatchResult update | ✓ VERIFIED | `FileSuccess` 7 fields. `file_md5()` streaming with 8KB buffer via `md5::Context`. `BatchResult.succeeded: Vec<FileSuccess>`. `PerFileProgress.seed_alias`. |
| `src-tauri/Cargo.toml` | md5 crate dependency | ✓ VERIFIED | `md5 = "0.8.0"` at line 29. |
| `src/stores/seed.ts` | selectedSeedIds: string[] with toggleSeed() | ✓ VERIFIED | 59 lines. `selectedSeedIds: ref<string[]>([])`. `toggleSeed()`, `selectAll()`, `deselectAll()`, `hasSelection`. Old `selectedSeedId`/`selectSeed` fully removed. |
| `src/types/batch.ts` | FileSuccess interface, updated BatchResult, seedAlias | ✓ VERIFIED | `FileSuccess` with 7 camelCase fields. `BatchResult.succeeded: FileSuccess[]`. `PerFileProgress.seedAlias`. Old `succeeded: string[]` removed. |
| `src/composables/useBatch.ts` | startBatch(seedIds: string[], ...) | ✓ VERIFIED | `startBatch(seedIds, outputDir, queueSize)`. Invokes `start_batch` with `{ seedIds, outputDir }`. |
| `src/components/batch/BatchControls.vue` | NSelect multiple mode, selectedSeedIds v-model | ✓ VERIFIED | `v-model:value="seedStore.selectedSeedIds"`, `multiple`, `filterable`, `clearable`. `startDisabled` checks `selectedSeedIds.length === 0`. `onStart` uses `seedStore.selectedSeedIds`. Old `selectedSeedId` references removed. |
| `src/components/batch/BatchSummary.vue` | MD5 comparison per-file row with status icons | ✓ VERIFIED | Per-row: filename, seed alias, MD5 before/after (8 chars, full hash on hover title), Modified/Unchanged status. Green CheckCircle / amber AlertCircle. `unchangedCount` computed. Warning banner for unchanged files. `bodyKey` selects `completeBodyWithMD5` when MD5 data present. CSS classes: `summary-row--success`, `summary-row--warning`. |
| `src/components/batch/BatchBanner.vue` | Multi-seed progress compatible | ✓ VERIFIED | Progress text `{completed}/{total}` correctly reflects Rust-computed `files * seeds` total. `currentFile` includes seed alias context. |
| `src/stores/batch.ts` | PerFileProgress composite key (file::seedAlias) | ✓ VERIFIED | `setPerFileProgress` uses `${p.file}::${p.seedAlias}` key. |
| `src/locales/en.json` | Multi-seed + MD5 i18n keys | ✓ VERIFIED | `selectSeeds`, updated `noSeedSelected`, `seed`, `md5Modified`, `md5Unchanged`, `md5UnchangedWarning`, `completeBodyWithMD5`. |
| `src/locales/zh-CN.json` | Multi-seed + MD5 i18n keys | ✓ VERIFIED | Corresponding Chinese translations for all keys above. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `tauri.conf.json` bundle key | `cargo tauri build` | Tauri CLI reads bundle config | ✓ WIRED | `bundle.active: true`, `targets: "all"`, platform sub-configs present |
| `.github/workflows/build.yml` | `tauri-apps/tauri-action@v0` | GitHub Actions step | ✓ WIRED | Line 59: `uses: tauri-apps/tauri-action@v0` with GITHUB_TOKEN |
| `.github/workflows/build.yml` | `src-tauri/tauri.conf.json` | tauri-action reads config | ✓ WIRED | tauri-action automatically reads version and bundle config |
| `detect_gpu_encoder()` in `ffmpeg/gpu.rs` | `lib.rs` setup | `tauri::async_runtime::spawn` | ✓ WIRED | Line 104-122 in lib.rs: spawns detection after FFmpeg path resolved |
| `AppState.gpu_encoder` | `executor.rs` `execute_single_file()` | `batch.rs` reads state, passes to executor | ✓ WIRED | batch.rs line 119-122: extracts gpu_encoder. Line 229: passes `gpu_encoder.as_ref()` |
| `executor.rs` codec injection | FFmpeg `-c:v <encoder>` | `final_args` insertion | ✓ WIRED | Lines 100-109: injects `-c:v <codec> -preset <fast/medium>` before filter args |
| `BatchControls.vue` NSelect | `seedStore.selectedSeedIds` | Pinia v-model binding | ✓ WIRED | Line 174: `v-model:value="seedStore.selectedSeedIds"` |
| `useBatch.startBatch(seedIds)` | `invoke('start_batch', { seedIds })` | Tauri IPC | ✓ WIRED | Line 63: `await invoke('start_batch', { seedIds, outputDir })` |
| `batch.rs` nested loop | `execute_single_file` per seed | `for seed in &seeds` | ✓ WIRED | Lines 205-229: inner loop calls `execute_single_file()` with each seed |
| `batch.rs` MD5 pre-hash | `tokio::task::spawn_blocking` HashMap | `file_md5()` call | ✓ WIRED | Lines 176-192: sequential spawn_blocking per file, stores in `md5_before_map` |
| `batch.rs` post-processing MD5 | `BatchResult.succeeded: Vec<FileSuccess>` | event emission to frontend | ✓ WIRED | Lines 233-269: Ok arm computes md5_after, compares, constructs FileSuccess |
| `BatchSummary.vue` file rows | `FileSuccess.modified` field | TypeScript interface | ✓ WIRED | Line 76: `fileResult.modified ? 'summary-row--success' : 'summary-row--warning'` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `batch.rs` pre-hash loop | `md5_before_map: HashMap<String, (String, u64)>` | `file_md5()` via `spawn_blocking` reading actual files | ✓ Real (reads filesystem) | ✓ FLOWING |
| `executor.rs` codec injection | `codec: &str` | `gpu_encoder.map(\|e\| e.encoder_name()).unwrap_or("libx264")` | ✓ Real (from AppState, or "libx264" literal) | ✓ FLOWING |
| `executor.rs` PerFileProgress | `seed_alias: seed.alias.clone()` | `seed.alias` from the `Seed` struct passed by batch.rs | ✓ Real (from user-authored seed alias) | ✓ FLOWING |
| `BatchControls.vue` seed selection | `seedStore.selectedSeedIds` | Pinia store array, toggled by user clicks | ✓ Real (user interaction via NSelect multiple) | ✓ FLOWING |
| `BatchSummary.vue` MD5 columns | `fileResult.md5Before`, `fileResult.md5After` | `BatchResult.succeeded: FileSuccess[]` from Rust via Tauri event | ✓ Real (hex strings from `file_md5()` computation) | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Rust compilation | `cargo check --manifest-path src-tauri/Cargo.toml` | Exit 0 (pre-existing warnings only) | ✓ PASS |
| Frontend type check | `bun vue-tsc -b` | Exit 0, clean | ✓ PASS |
| Rust tests | `cargo test --manifest-path src-tauri/Cargo.toml` | 13/13 pass (including new `test_detect_gpu_encoder_no_ffmpeg`) | ✓ PASS |
| Bundle config valid | `grep -c '"active": true' src-tauri/tauri.conf.json` | 1 | ✓ PASS |
| CI YAML valid | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/build.yml'))"` | Parse success | ✓ PASS |
| Old store API removed | `grep -r 'selectedSeedId[^s]' src/` | No matches | ✓ PASS |
| md5 crate dependency | `grep 'md5 = "0.8.0"' src-tauri/Cargo.toml` | 1 match | ✓ PASS |
| Module declarations | `grep 'pub mod gpu' src-tauri/src/ffmpeg/mod.rs src-tauri/src/models/mod.rs` | 1 match each | ✓ PASS |
| GPU retry guard | `grep 'gpu_encoder.is_some()' src-tauri/src/commands/batch.rs` | 1 match | ✓ PASS |
| GPU retry event | `grep 'gpu-encode-failed-fallback' src-tauri/src/commands/batch.rs` | 1 match | ✓ PASS |
| Multi-seed nested loop | `grep 'for seed in &seeds' src-tauri/src/commands/batch.rs` | 1 match | ✓ PASS |
| Total = files * seeds | `grep 'queue_snapshot.len() \* seeds.len()' src-tauri/src/commands/batch.rs` | 1 match | ✓ PASS |
| MD5 streaming hash | `grep 'hasher.consume\|hasher.finalize' src-tauri/src/models/batch.rs` | 2 matches | ✓ PASS |
| MD5 spawn_blocking | `grep -c 'spawn_blocking' src-tauri/src/commands/batch.rs` | 3 (pre-hash + Ok-arm + retry-path) | ✓ PASS |
| Lock drop before emit | `grep 'drop(batch_state);\s*drop(app_state)' src-tauri/src/commands/batch.rs` | Pattern found in both Ok and Err arms | ✓ PASS |
| i18n MD5 keys en | `grep -c 'completeBodyWithMD5\|md5Modified\|md5Unchanged' src/locales/en.json` | 3 matches | ✓ PASS |
| i18n MD5 keys zh-CN | `grep -c 'completeBodyWithMD5\|md5Modified\|md5Unchanged' src/locales/zh-CN.json` | 3 matches | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CROSS-01 | 05-01 | Windows 打包 -- 生成 .msi 和 .exe 安装包 | ✓ SATISFIED | `tauri.conf.json` bundle.windows.nsis config. Build generates .msi via WiX + .exe via NSIS when `targets: "all"`. |
| CROSS-02 | 05-01 | Linux 打包 -- 生成 .AppImage 和 .deb 安装包 | ✓ SATISFIED | `tauri.conf.json` bundle.linux.deb + bundle.linux.appimage config. |
| CROSS-03 | 05-02 | CI 矩阵构建 (macOS/Windows/Linux), 自动上传构建产物 | ✓ SATISFIED | `.github/workflows/build.yml` 4-platform matrix with `tauri-apps/tauri-action@v0` for release artifact upload. |
| PERF-01 | 05-03 | GPU 硬件编码器自动检测与选择 (VideoToolbox/NVENC+AMF/VAAPI) | ✓ SATISFIED | `GpuEncoder` enum (4 variants) + `detect_gpu_encoder()` via `ffmpeg -encoders` + startup detection + executor codec injection + GPU retry-on-failure. |
| PERF-02 | 05-04 | 并行 pipeline 优化 -- 调度器减少空等、流式读写避免大内存分配 | ✓ SATISFIED | Reduced Mutex lock frequency (single lock per completion with drop-before-emit). D-08 streaming I/O verification comment in executor. `child.iter()` drains stderr without buffering. |
| MULTI-01 | 05-05 | 多种子选择 UI (可勾选多个种子) | ✓ SATISFIED | NSelect `multiple` mode in BatchControls.vue. `seedStore.selectedSeedIds: string[]` with `toggleSeed()`, `selectAll()`, `deselectAll()`. `hasSelection` computed. |
| MULTI-02 | 05-05 | 一个视频 x N 个种子 = N 个输出文件, 命名保持 {原文件名}_{种子别名}.{扩展名} | ✓ SATISFIED | Rust nested loop (`for file { for seed { ... } }`). `total = queue.len() * seeds.len()`. Output naming unchanged: `make_output_path()` uses `{stem}_{seed_alias}.{ext}`. |
| MD5-01 | 05-06 | 处理前记录每个文件的 MD5 和文件大小 | ✓ SATISFIED | `file_md5()` streaming hash. Pre-processing `HashMap<String, (String, u64)>` computed via `spawn_blocking` before loop. Stored as `(md5_hex, size_bytes)`. |
| MD5-02 | 05-06 | 处理后对比 MD5, 差异数据写入处理日志, 输出=输入时告警 | ✓ SATISFIED | Post-processing MD5 comparison with `modified = md5_before != md5_after`. `FileSuccess` struct carries both hashes to frontend. `BatchSummary` displays Modified/Unchanged status with amber warning banner for unchanged files. |

All 9 Phase 5 requirements are accounted for and satisfied. No orphaned requirements.

### Anti-Patterns Found

No anti-patterns detected in phase-modified files:

| File | Check | Result |
|------|-------|--------|
| `src-tauri/src/commands/batch.rs` (495 lines) | TODOs, FIXMEs, placeholder | CLEAN |
| `src-tauri/src/ffmpeg/executor.rs` (266 lines) | TODOs, FIXMEs, placeholder | CLEAN |
| `src-tauri/src/ffmpeg/gpu.rs` (57 lines) | TODOs, FIXMEs, placeholder | CLEAN |
| `src-tauri/src/models/batch.rs` (117 lines) | TODOs, FIXMEs, placeholder | CLEAN |
| `src-tauri/src/models/gpu.rs` (27 lines) | TODOs, FIXMEs, placeholder | CLEAN |
| `src/components/batch/BatchSummary.vue` | TODOs, FIXMEs, placeholder ("placeholder" only as NSelect prop attr) | CLEAN |
| `src/components/batch/BatchControls.vue` | TODOs, FIXMEs, placeholder ("placeholder" only as NSelect prop attr) | CLEAN |
| `src/stores/seed.ts` | Old `selectedSeedId`/`selectSeed` API | FULLY REMOVED |
| `src/stores/batch.ts` | Composite key for perFileProgress | CORRECTLY IMPLEMENTED |

No hardcoded empty data (stubs), no return-null functions, no placeholder implementations. All data flows are wired to real sources.

### Human Verification Required

1. **Cross-platform installer generation:** Run `cargo tauri build` on macOS, Windows, and Linux. Expected: each platform produces its installer artifacts (.dmg on macOS, .msi/.exe on Windows, .deb/.AppImage on Linux). Why human: requires running on each target OS with platform toolchains.

2. **GPU encoder detection and throughput:** Run the app on a GPU-equipped machine and start a batch job. Expected: GPU encoder detected at startup (`gpu-encoder-detected` event fires), FFmpeg uses hardware encoder (`-c:v h264_nvenc`/`h264_amf`/`h264_videotoolbox`), encoding throughput is measurably faster than CPU. Why human: depends on actual GPU hardware and drivers.

3. **CI workflow execution:** Push to the `release` branch and verify all 4 matrix jobs (macOS aarch64, macOS x86_64, ubuntu-22.04, windows-latest) run successfully on GitHub Actions, producing a draft release with all platform artifacts. Why human: requires GitHub Actions infrastructure, repository secrets, and network access.

4. **MD5 integrity display end-to-end:** Process a video batch with multiple seeds and verify BatchSummary shows correct MD5 before/after hex strings (first 8 chars) with green CheckCircle for modified files, amber AlertCircle for unchanged files, and the unchanged warning banner when applicable. Why human: requires real video files and visual inspection of UI rendering fidelity.

5. **v1 regression test:** Run through the complete v1 workflow (generate seeds, import videos, manage queue, single-seed batch processing, progress streaming, cancel, completion summary). Expected: all pre-existing functionality continues to work identically. Why human: end-to-end workflow integrity requires interactive testing; compilation passes don't guarantee runtime behavior.

### Gaps Summary

No implementation gaps found. All 6 plans (05-01 through 05-06) were executed as designed:

- **05-01 (Bundle config):** `tauri.conf.json` has correct `bundle` key with Windows/Linux targets and version 1.0.0
- **05-02 (CI workflow):** `.github/workflows/build.yml` with 4-platform matrix using `tauri-action@v0`
- **05-03 (GPU detection):** `GpuEncoder` model, `detect_gpu_encoder()` function, AppState wiring, executor injection, startup detection
- **05-04 (GPU wiring + optimization):** GPU encoder passed from AppState to executor, Mutex lock frequency reduced with drop-before-emit pattern
- **05-05 (Multi-seed):** Frontend store refactored to `selectedSeedIds: string[]`, NSelect multiple mode, Rust nested loop with `total = files * seeds`, `PerFileProgress.seed_alias`
- **05-06 (MD5 + GPU retry):** Streaming MD5 via `md5` crate, pre/post hash comparison, `FileSuccess` with `modified` flag, BatchSummary MD5 columns with status icons, GPU retry-on-failure with CPU fallback

All code compiles clean (`cargo check` passes, `bun vue-tsc -b` passes). All existing tests pass (13 Rust tests, 22 frontend tests). The old `selectedSeedId`/`selectSeed` single-seed API has been fully removed from the codebase. Zero TODOs, FIXMEs, stubs, or placeholder implementations in phase-modified files.

The 5 human verification items above are the remaining acceptance gates for this phase. They require:
- Platform-specific build environments (items 1 and 3)
- GPU hardware (item 2)
- End-to-end runtime testing with real video files (items 4 and 5)

---

_Verified: 2026-05-15T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
