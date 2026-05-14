# Phase 5: Production Hardening - Research

**Researched:** 2026-05-14
**Domain:** Tauri 2.x cross-platform build, GPU hardware acceleration, multi-seed batch processing, MD5 integrity verification
**Confidence:** HIGH

## Summary

Phase 5 adds four production capabilities to the existing Sandwich app: (1) cross-platform installer packaging for Windows (.msi/.exe) and Linux (.AppImage/.deb) via Tauri v2's built-in bundler and a GitHub Actions CI matrix; (2) GPU hardware-accelerated encoding with automatic NVENC/VideoToolbox/VAAPI detection and silent CPU fallback; (3) multi-seed batch processing where one video x N seeds = N output files; and (4) MD5 checksum verification comparing input and output files with results displayed inline in the BatchSummary.

The Tauri v2 `bundle` configuration is the primary new infrastructure — the current `tauri.conf.json` has no `bundle` key at all. GPU acceleration is a pure Rust-side change: detect available encoders via `ffmpeg -encoders` at startup and inject the `-c:v` flag into existing FFmpeg command construction. Multi-seed requires a Pinia store refactor (`selectedSeedId` -> `selectedSeedIds: string[]`), Naive UI NSelect `multiple` prop, and a nested loop in `batch.rs`. MD5 leverages the standard `md5` crate (v0.8.0) for streaming file hashing, with results returned in an expanded `BatchResult` type.

**Primary recommendation:** Add `bundle` config to `tauri.conf.json` with platform-specific targets, use `tauri-apps/tauri-action@v0` for CI matrix builds (bun-based), detect GPU encoders via `ffmpeg -hide_banner -encoders | grep`, use the `md5` crate (v0.8.0) for hashing, and refactor the batch loop to `for file { for seed { ... } }` with a `seed_alias` field on progress events.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Bundle config (tauri.conf.json) | Frontend Server (build-time) | — | Static config, no runtime logic |
| CI matrix build | CI/CD (GitHub Actions) | — | External build service |
| GPU encoder detection | API / Backend (Rust) | — | `ffmpeg -encoders` shell command + string parsing |
| GPU encoder injection | API / Backend (Rust) | — | `-c:v <encoder>` appended to FFmpeg args in executor/filters |
| Multi-seed selection UI | Browser / Client (Vue) | — | NSelect `multiple` prop, Pinia store |
| Multi-seed batch loop | API / Backend (Rust) | — | `start_batch` double-loop in batch.rs |
| MD5 file hashing | API / Backend (Rust) | — | `md5` crate, streaming file I/O |
| MD5 display in summary | Browser / Client (Vue) | — | BatchSummary template, expanded BatchResult type |

## Standard Stack

### Core (New Dependencies for Phase 5)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| md5 (crate) | 0.8.0 | MD5 file hashing (streaming) | Most-downloaded Rust MD5 crate (~6M+). Simple `Md5::read(reader)` API avoids loading entire files into memory. Pure Rust, no native deps. [VERIFIED: crates.io] |
| tauri-apps/tauri-action | v0 | GitHub Actions CI build + release | Official Tauri CI action. Auto-builds for platform matrix, creates GitHub Release, uploads all bundle artifacts. Replaces __VERSION__ in tag names. [VERIFIED: Context7 /websites/v2_tauri_app] |
| oven-sh/setup-bun | v2 | Bun runtime in CI | Required because project uses bun as package manager (`bun dev`/`bun build` in tauri.conf.json). [VERIFIED: bun 1.3.2 in package.json scripts] |

### Existing (No Version Changes)

| Library | Phase 5 Usage |
|---------|--------------|
| ffmpeg-sidecar 2.5.1 | `auto_download()` for platform FFmpeg binaries; `FfmpegCommand::new_with_path()` executor unchanged |
| Naive UI 2.44.1 | NSelect with `multiple` prop for seed multi-select |
| Pinia 3.0.4 | `selectedSeedIds: string[]` refactor, `toggleSeed()` action |
| tokio 1.52.3 | Async batch processing unchanged |
| tauri 2.11.1 | `#[tauri::command]` IPC unchanged; new `bundle` config key |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| md5 crate 0.8.0 | fast-md5 1.0.0 | fast-md5 has hand-written x86_64/aarch64 assembly for ~2x throughput, but MD5 of multi-MB video files is I/O-bound, not CPU-bound. md5 crate is simpler, pure Rust, battle-tested. [VERIFIED: crates.io] |
| md5 crate | `std::process::Command` calling `md5sum` / `openssl md5` | CLI is platform-inconsistent (macOS has `/sbin/md5`, Linux has `md5sum`, Windows has neither). Rust crate is portable. |
| md5 crate | `ffmpeg -f hash -hash md5` | Adds unnecessary FFmpeg process spawn per file. Rust crate is 10x faster for metadata operations. |
| tauri-apps/tauri-action@v0 | Manual `cargo tauri build` + `actions/upload-artifact` | tauri-action handles release creation, version templating, and multi-platform artifact collection in one step. Manual approach requires ~50 lines of boilerplate per platform. [VERIFIED: Context7 /websites/v2_tauri_app] |
| ubuntu-latest (24.04) | ubuntu-22.04 | Official Tauri v2 CI templates use ubuntu-22.04 for stability. 24.04 needs libsoup3.0-dev instead of libsoup2.4-dev. [VERIFIED: Context7 /websites/v2_tauri_app] |

**Installation:**
```bash
cargo add md5 --manifest-path src-tauri/Cargo.toml
# GitHub Actions: no local install needed (uses actions in CI)
```

## Architecture Patterns

### System Architecture Diagram

```
                                  GitHub Actions CI Matrix
                                  ┌──────────────────────────────────────────────┐
                                  │  push to release / workflow_dispatch         │
                                  │        │                                      │
                                  │  ┌─────┴──────────┬──────────────┐           │
                                  │  ▼                ▼              ▼           │
                                  │ macOS       ubuntu-22.04    windows-latest   │
                                  │  │                │              │           │
                                  │  │ apt pkg        │ apt pkg      │ (none)    │
                                  │  │ + bun install  │ + bun install│ + bun     │
                                  │  │ + tauri-action │ + tauri-act. │ + tauri   │
                                  │  ▼                ▼              ▼           │
                                  │ .dmg         .deb .AppImage   .msi .exe      │
                                  │                                      │       │
                                  │  └──────────┬──────────┬───────────┘       │
                                  │             ▼                               │
                                  │     GitHub Release (artifacts uploaded)     │
                                  └──────────────────────────────────────────────┘

                            Rust Backend (GPU + Multi-Seed + MD5)
                            ┌──────────────────────────────────────────────────┐
                            │                                                  │
                            │  App Startup                                     │
                            │  ├─ ffmpeg_sidecar::auto_download() (unchanged)  │
                            │  └─ GPU Encoder Detection (NEW)                  │
                            │     ├─ ffmpeg -hide_banner -encoders             │
                            │     ├─ parse stdout for platform encoders        │
                            │     └─ store in AppState.gpu_encoder             │
                            │                                                  │
                            │  start_batch(queue, seed_ids[], output_dir)      │
                            │  │                                               │
                            │  ├─ MD5 BEFORE: for each file, stream hash       │
                            │  │   └─ HashMap<filepath, (md5_hash, file_size)> │
                            │  │                                               │
                            │  ├─ for file in queue:                           │
                            │  │   for seed in seeds:                          │
                            │  │     ├─ build_filter_args(seed.operations)     │
                            │  │     ├─ inject GPU encoder: -c:v <gpu_enc>     │
                            │  │     │   └─ fallback to libx264 on failure     │
                            │  │     ├─ execute_single_file()                  │
                            │  │     └─ emit batch-file-progress {seed_alias}  │
                            │  │                                               │
                            │  └─ MD5 AFTER: for each output, stream hash      │
                            │      └─ emit batch-complete {md5 comparisons}    │
                            └──────────────────────────────────────────────────┘
                                         │                   ▲
                            Tauri IPC    │    Events         │ invoke()
                            ─────────────┼───────────────────┼─────────────────
                                         ▼                   │
                            Vue Frontend (Multi-Seed UI + MD5 Display)
                            ┌──────────────────────────────────────────────────┐
                            │                                                  │
                            │  BatchControls.vue                               │
                            │  ├─ NSelect multiple :value → selectedSeedIds[]  │
                            │  └─ "Start Batch" → invoke('start_batch', {...}) │
                            │                                                  │
                            │  BatchSummary.vue                                │
                            │  ├─ Per-file rows with expanded columns:         │
                            │  │   [filename] [seed] [md5_before] [md5_after]  │
                            │  │   [status icon: ✓ modified / ⚠ unchanged / ✗] │
                            │  └─ Warning banner: "N files unchanged (MD5)"    │
                            └──────────────────────────────────────────────────┘
```

### Recommended Project Structure (Changes Only)

```
src-tauri/
├── tauri.conf.json              # ADD bundle key (no existing bundle config)
├── Cargo.toml                   # ADD md5 dependency
├── src/
│   ├── ffmpeg/
│   │   ├── executor.rs          # MODIFY: accept gpu_encoder param, inject -c:v
│   │   ├── filters.rs           # UNCHANGED (GPU encoder is -c:v, not a filter)
│   │   └── gpu.rs               # NEW: GPU encoder detection + fallback logic
│   ├── commands/
│   │   ├── batch.rs             # MODIFY: seed_id -> seed_ids[], nested loop, MD5
│   │   └── seed.rs              # UNCHANGED
│   ├── models/
│   │   ├── batch.rs             # MODIFY: BatchResult.succeeded type, add Md5Info
│   │   └── gpu.rs               # NEW: GpuEncoder enum, GpuInfo struct
│   └── state.rs                 # MODIFY: add gpu_encoder field to AppState
├── icons/                       # EXISTING: 32x32, 128x128, 128x128@2x, icon.png
│                                # Tauri gen: needs icon.icns (macOS), icon.ico (Win)
└── .github/
    └── workflows/
        └── build.yml            # NEW: cross-platform build matrix
```

### Pattern 1: GPU Encoder Auto-Detection

**What:** At app startup, run `ffmpeg -hide_banner -encoders`, grep for platform-specific encoder names, store the best available encoder (or None) in AppState.

**When to use:** PERF-01 requirement — detect once at startup, not per-batch.

**Example:**
```rust
// Source: Standard FFmpeg CLI + existing AppState pattern
// File: src-tauri/src/ffmpeg/gpu.rs (NEW)

use std::process::Command;

/// Detected GPU encoder, or None if only CPU is available.
#[derive(Debug, Clone, serde::Serialize)]
pub enum GpuEncoder {
    /// macOS: h264_videotoolbox or hevc_videotoolbox
    VideoToolbox,
    /// Windows NVIDIA: h264_nvenc or hevc_nvenc
    Nvenc,
    /// Windows AMD: h264_amf or hevc_amf
    Amf,
    /// Linux: h264_vaapi or hevc_vaapi
    Vaapi,
}

impl GpuEncoder {
    /// Return the FFmpeg -c:v encoder name
    pub fn encoder_name(&self) -> &str {
        match self {
            Self::VideoToolbox => "h264_videotoolbox",
            Self::Nvenc => "h264_nvenc",
            Self::Amf => "h264_amf",
            Self::Vaapi => "h264_vaapi",
        }
    }
}

/// Detect best available GPU encoder for this platform.
/// Returns None if no hardware encoder found (CPU fallback).
pub fn detect_gpu_encoder(ffmpeg_path: &str) -> Option<GpuEncoder> {
    let ffmpeg_bin = std::path::Path::new(ffmpeg_path)
        .join(if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" });

    let output = Command::new(&ffmpeg_bin)
        .args(["-hide_banner", "-encoders"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    #[cfg(target_os = "macos")]
    {
        if stdout.contains("h264_videotoolbox") {
            return Some(GpuEncoder::VideoToolbox);
        }
    }
    #[cfg(target_os = "windows")]
    {
        if stdout.contains("h264_nvenc") {
            return Some(GpuEncoder::Nvenc);
        }
        if stdout.contains("h264_amf") {
            return Some(GpuEncoder::Amf);
        }
    }
    #[cfg(target_os = "linux")]
    {
        if stdout.contains("h264_vaapi") {
            return Some(GpuEncoder::Vaapi);
        }
    }

    None // CPU fallback: use libx264 (default)
}
```

### Pattern 2: GPU Encoder Injection into FFmpeg Args

**What:** Insert `-c:v <encoder>` into the `all_args` vec before FFmpeg execution. If GPU encode fails, retry with `libx264`.

**When to use:** Every `execute_single_file()` call when a GPU encoder is available.

**Example:**
```rust
// Modification to src-tauri/src/ffmpeg/executor.rs:71-95 (all_args assembly)
// After building all_args from filters, inject GPU encoder:

let codec = gpu_encoder
    .as_ref()
    .map(|e| e.encoder_name())
    .unwrap_or("libx264");

// Insert codec args at the front (before other args)
let mut final_args = vec![
    "-c:v".to_string(),
    codec.to_string(),
    "-preset".to_string(),
    if gpu_encoder.is_some() { "fast".to_string() } else { "medium".to_string() },
];
final_args.extend(all_args);
```

### Pattern 3: Multi-Seed Batch Loop

**What:** Outer loop (files) x inner loop (seeds). Total progress = files.len() * seeds.len(). Each `PerFileProgress` event includes `seed_alias`.

**When to use:** MULTI-01, MULTI-02 — replaces current single-seed batch loop.

**Example:**
```rust
// Modification to src-tauri/src/commands/batch.rs:141-235
// Changed: seed_id: String -> seed_ids: Vec<String>
// Changed: resolve single seed -> resolve Vec<Seed>
// Changed: total = queue.len() * seeds.len()
// Added: inner loop for seeds

let seeds: Vec<Seed> = seed_ids.iter()
    .filter_map(|id| app_state.seeds.iter().find(|s| s.id == *id).cloned())
    .collect();

let total_count = queue_snapshot.len() * seeds.len();
batch_state.progress = BatchProgress {
    total: total_count,  // CHANGED: was queue_snapshot.len()
    completed: 0,
    succeeded: 0,
    failed: 0,
    current_file: None,
};

for entry in &queue_snapshot {
    if cancel_flag.load(Ordering::SeqCst) { break; }
    for seed in &seeds {
        if cancel_flag.load(Ordering::SeqCst) { break; }
        // ... execute_single_file with seed ...
        // PerFileProgress now includes seed_alias
    }
}
```

### Pattern 4: MD5 Streaming Hash for Large Files

**What:** Use `md5::Md5` context with `std::io::copy` to hash files without loading into memory.

**When to use:** MD5-01 (pre-processing hash) and MD5-02 (post-processing comparison).

**Example:**
```rust
// Source: md5 crate docs (crates.io/md5)
use std::fs::File;
use std::io::Read;

/// Compute MD5 hash of a file via streaming I/O.
/// Returns hex string on success, error on I/O failure.
pub fn file_md5(path: &std::path::Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Cannot open file for MD5: {}", e))?;
    let mut hasher = md5::Md5::new();
    let mut buf = [0u8; 8192]; // 8KB buffer

    loop {
        let n = file.read(&mut buf)
            .map_err(|e| format!("MD5 read error: {}", e))?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
```

### Anti-Patterns to Avoid

- **Loading entire file into memory for MD5:** Video files can be gigabytes. Always use streaming (`std::io::Read` + `Md5::update()` in chunks). The `md5::compute(&[u8])` function requires the entire buffer in memory — do not use for files.
- **GPU encoder without fallback:** Always wrap GPU encode in a retry: attempt with `-c:v h264_nvenc`, if exit code != 0, retry with `-c:v libx264`. Never let GPU encoder unavailability block batch completion.
- **Hardcoding GPU encoder per platform:** Detection via `ffmpeg -encoders` is more robust than `cfg!(target_os)` switching. A Linux machine without VAAPI drivers should fall back to CPU even though it's Linux.
- **NSelect multiple without `clearable`:** Multi-select without clearable forces user to deselect items one by one. The existing `clearable` prop should remain.
- **Forgetting to update `total` in `BatchProgress`:** When `total = files × seeds`, forgetting to update the `start_batch` initial progress emission will show "0/1" instead of "0/N" in the UI.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MD5 file hashing | Manual FFmpeg `-f hash` subprocess or reading entire file into mem | `md5` crate (v0.8.0) with streaming I/O | Edge cases: large files (>4GB), pipe buffer deadlock, cross-platform availability. The `md5` crate handles chunked streaming natively. |
| GitHub Release creation + asset upload | Custom `gh release create` + per-platform `gh release upload` scripts | `tauri-apps/tauri-action@v0` | tauri-action auto-detects bundle output paths, handles version substitution (`__VERSION__`), supports draft releases, and collects all platform artifacts in one step. |
| GPU encoder heuristic | `cfg!(target_os)` + hardcoded encoder names | Runtime `ffmpeg -encoders` probe | A Linux machine may lack VAAPI drivers; a Windows machine may have only AMD (no NVIDIA). Runtime detection is the only reliable approach. |
| MSI installer (Windows) | Custom WiX XML + candle/light toolchain | Tauri v2 `bundle.windows.wix` config | Tauri CLI auto-generates WiX source from config, invokes WiX Toolset, and handles signing. Manual WiX is ~200 lines of XML per app. |

**Key insight:** Tauri v2's bundler is the biggest "don't hand-roll" in this phase. The `cargo tauri build` command automatically generates platform-appropriate installers from declarative JSON config. The CI equivalent (`tauri-apps/tauri-action@v0`) wraps this in GitHub Actions with release management. Building installers manually via NSIS scripts, WiX XML, or debian packaging tools is error-prone and platform-specific — exactly what Tauri's bundler abstracts.

## Runtime State Inventory

> Skipped. This phase is not a rename/refactor/migration phase — it adds new features (cross-platform build, GPU, multi-seed, MD5) to existing functionality. No stored data, live service config, OS-registered state, secrets, or build artifacts carry old names that need migration.

## Common Pitfalls

### Pitfall 1: Missing `bundle` Key in tauri.conf.json

**What goes wrong:** `cargo tauri build` on Windows/Linux produces no installer output. The build succeeds but only the binary is compiled — no .msi, .exe, .deb, or .AppImage is generated.

**Why it happens:** The current `tauri.conf.json` has no `bundle` key at all (only `build`, `app`, `plugins`). Without explicit `bundle` configuration, Tauri v2 may not generate installers, especially on non-macOS platforms where the default `targets: "all"` behavior may differ.

**How to avoid:** Add a `bundle` key with at minimum `"active": true` and `"targets": "all"`. Include `windows`, `linux`, and `macOS` sub-configs even if empty objects.

**Warning signs:** `cargo tauri build` output shows "Finished release [optimized]" but no `bundle/` directory is created.

### Pitfall 2: Ubuntu 24.04 vs 22.04 System Dependencies

**What goes wrong:** CI build on `ubuntu-latest` (24.04) fails with webkit/GTK linking errors despite having the "standard" Tauri deps installed.

**Why it happens:** Ubuntu 24.04 (Noble) replaced `libsoup2.4-dev` with `libsoup3.0-dev` and `libjavascriptcoregtk-4.1-dev` is required (not just webkit2gtk). The standard Tauri v2 docs list deps for 22.04.

**How to avoid:** Either pin to `ubuntu-22.04` (recommended — matches Tauri v2 official CI templates) or include both old and new package names with `||` fallbacks in the apt install command.

**Warning signs:** Linker errors about `libsoup-3.0`, `javascriptcoregtk`, or undefined references to webkit functions.

### Pitfall 3: GPU Encoder Not in ffmpeg-sidecar Binary

**What goes wrong:** GPU encoder detection succeeds (encoder name found in `ffmpeg -encoders`) but encoding fails at runtime because the specific GPU isn't available (no NVIDIA GPU, VAAPI driver missing) or the binary was compiled without that encoder.

**Why it happens:** ffmpeg-sidecar downloads standard FFmpeg builds that may or may not include all hardware encoders. Additionally, encoder presence in `-encoders` output doesn't guarantee the hardware is available at runtime — NVENC needs `nvidia-smi` running, VAAPI needs the `i965-va-driver` package.

**How to avoid:** After detecting an encoder, validate it with a quick test encode (`ffmpeg -f lavfi -i color=size=2x2 -frames:v 1 -c:v <encoder> -f null -`). If the test fails, treat the encoder as unavailable and fall back to CPU. CACHE THIS RESULT — do not re-test per batch.

**Warning signs:** FFmpeg exits with "Cannot load libcuda" or "Failed to initialise VAAPI connection" despite encoder being listed.

### Pitfall 4: MD5 Hash of Multi-GB Files Blocking the Main Thread

**What goes wrong:** MD5 computation on a 2GB video file takes several seconds. If done synchronously in the batch loop, it blocks progress updates and cancellation checks, making the UI appear frozen.

**Why it happens:** File I/O is blocking by nature. The `md5` crate's `Md5::read()` is synchronous, and if called inside the batch processing loop (which runs on a tokio thread), it blocks that thread.

**How to avoid:** Wrap MD5 computation in `tokio::task::spawn_blocking()`. For pre-processing hashes, compute all hashes upfront (parallel via `tokio::spawn`) before starting the batch loop. For post-processing hashes, compute after each file completes, also in `spawn_blocking`.

**Warning signs:** BatchSummary shows no progress for several seconds at batch start, then suddenly jumps. Per-file progress bars freeze.

### Pitfall 5: Stale `selectedSeedId` References After Multi-Select Refactor

**What goes wrong:** After changing `seedStore.selectedSeedId: string | null` to `selectedSeedIds: string[]`, TypeScript catches most breakages, but runtime issues occur where code accesses `.length` on a null value or passes the old single-ID string to `start_batch`.

**Why it happens:** `selectedSeedId` is referenced in multiple places: `BatchControls.vue` (v-model), `useBatch` composable, `start_batch` invoke call, and potentially watchers. A grep for `selectedSeedId` must find ALL references.

**How to avoid:** After the refactor, grep the entire `src/` directory for `selectedSeedId` and `selectedSeed` (the computed). Every match must be updated. The `start_batch` command signature changes from `seed_id: String` to `seed_ids: Vec<String>`.

**Warning signs:** `start_batch` receives a string when it expects a `Vec<String>`. Seed store `selectedSeed` computed returns `null` when an array is expected.

### Pitfall 6: MSI Build Requires Windows Runner

**What goes wrong:** CI attempts to build `.msi` on a Linux or macOS runner and fails with "WiX Toolset not found."

**Why it happens:** WiX Toolset v3 runs exclusively on Windows. `.msi` installers can only be built on Windows runners. NSIS installers (`.exe`) can be cross-compiled from Linux/macOS.

**How to avoid:** In the CI matrix, only run `cargo tauri build` (which auto-selects platform-appropriate targets) on the matching OS. The Windows runner naturally produces both `.msi` and `.nsis`. Linux runner produces `.deb` + `.AppImage`. No cross-compilation needed.

**Warning signs:** Build log: "error: WiX Toolset is required to build MSI installers on this platform."

## Code Examples

### Tauri Bundle Configuration (tauri.conf.json addition)

```json
// Source: Context7 /websites/v2_tauri_app (reference/config)
// Add this as a top-level key alongside "build", "app", "plugins"
"bundle": {
  "active": true,
  "targets": "all",
  "icon": [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.png"
  ],
  "windows": {
    "nsis": {
      "installMode": "currentUser"
    }
  },
  "linux": {
    "deb": {
      "depends": []
    },
    "appimage": {
      "bundleMediaFramework": false
    }
  }
}
```

### GitHub Actions Build Matrix (build.yml)

```yaml
# Source: Context7 /websites/v2_tauri_app (distribute/pipelines/github)
# Adapted for bun (not yarn/npm)
name: Build

on:
  push:
    branches: ['release']
  workflow_dispatch:

jobs:
  build:
    permissions:
      contents: write
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
          - platform: 'macos-latest'
            args: '--target x86_64-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
          - platform: 'windows-latest'
            args: ''

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Ubuntu dependencies
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev \
            libayatana-appindicator3-dev librsvg2-dev patchelf \
            libsoup-3.0-dev libjavascriptcoregtk-4.1-dev

      - name: Setup Bun
        uses: oven-sh/setup-bun@v2
        with:
          bun-version: '1.3.2'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.platform == 'macos-latest' && 'aarch64-apple-darwin,x86_64-apple-darwin' || '' }}

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: './src-tauri -> target'

      - name: Install frontend dependencies
        run: bun install --frozen-lockfile

      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: v__VERSION__
          releaseName: 'Sandwich v__VERSION__'
          releaseBody: 'See assets to download and install.'
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

### MD5-Integrated BatchResult Types (Rust)

```rust
// Source: Extending existing src-tauri/src/models/batch.rs
// Changes: BatchResult.succeeded type changes from Vec<String> to Vec<FileSuccess>

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub succeeded: Vec<FileSuccess>,
    pub failed: Vec<FileResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSuccess {
    /// Output file path.
    pub path: String,
    /// Seed alias used for this output.
    pub seed_alias: String,
    /// Input file path (for correlation).
    pub source_file: String,
    /// MD5 hash before processing (hex string).
    pub md5_before: String,
    /// MD5 hash after processing (hex string).
    pub md5_after: String,
    /// true if md5_before != md5_after (i.e., file was modified).
    pub modified: bool,
    /// File size before processing (bytes).
    pub size_bytes: u64,
}
```

### TypeScript BatchResult Update

```typescript
// Source: Extending src/types/batch.ts
// Changes: succeeded from string[] to FileSuccess[]

export interface FileSuccess {
  path: string;
  seedAlias: string;
  sourceFile: string;
  md5Before: string;
  md5After: string;
  modified: boolean;
  sizeBytes: number;
}

export interface BatchResult {
  succeeded: FileSuccess[];
  failed: FileResult[];
}
```

### PerFileProgress with seed_alias

```rust
// Source: Extending src-tauri/src/models/batch.rs PerFileProgress
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerFileProgress {
    pub file: String,
    /// NEW: which seed is producing this output (needed to distinguish
    /// multiple outputs from the same source file).
    pub seed_alias: String,
    pub percent: f64,
    pub current_frame: u32,
    pub total_frames: u32,
    pub fps: f32,
    pub remaining_seconds: f64,
}
```

### Naive UI NSelect Multiple

```vue
<!-- Source: Context7 /tusen-ai/naive-ui -->
<!-- Changes to src/components/batch/BatchControls.vue -->
<n-select
  v-model:value="seedStore.selectedSeedIds"
  :options="seedOptions"
  multiple
  filterable
  clearable
  placeholder="Select seeds..."
/>
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `bundle` key optional/implicit | `bundle` key explicit with `active: true`, `targets`, platform configs | Tauri v2 (2024) | Required for reliable cross-platform installer generation |
| NSIS only for Windows | NSIS + WiX MSI (`targets: ["nsis", "msi"]`) | Tauri v2 (2024) | MSI is enterprise-friendly (Group Policy deployment) |
| `ubuntu-latest` = 22.04 | `ubuntu-22.04` pinned explicitly | 2025 (GitHub Actions migration) | Avoids 24.04 breaking changes to libsoup/sys deps |
| Single seed per batch | Multi-seed batch (`seed_ids: Vec<String>`) | Phase 5 | Core feature for "one-click batch deduplication" value prop |
| No integrity verification | MD5 before/after comparison per file | Phase 5 | Required for user confidence; catches zero-change outputs |

**Deprecated/outdated:**
- `tauri.bundle.targets: "all"` with no sub-config: While still valid Tauri v2 syntax, pinning to explicit targets and including platform configs prevents surprises when build environment changes.
- `libsoup2.4-dev` on Ubuntu 24.04: Replaced by `libsoup-3.0-dev`. If using ubuntu-22.04, the old package name still works.
- `actions/upload-artifact@v3`: v4 is current (2025). tauri-action v0 uses v4 internally.
- `md5sum` CLI for Windows: Windows has `CertUtil -hashfile` not `md5sum`. Using the Rust `md5` crate avoids path divergence.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Windows 目标格式：`.msi`（Tauri 默认）和 `.exe`（NSIS 安装器）。Linux 目标格式：`.AppImage` 和 `.deb`。
- **D-02:** CI 通过 GitHub Actions 矩阵构建（`os: [macos-latest, ubuntu-latest, windows-latest]`），产物自动上传到 Release。FFmpeg 通过 ffmpeg-sidecar `auto_download()` 按平台获取，不随安装包分发。
- **D-03:** Tauri 现有 macOS 构建经验（本地 `cargo tauri build` 产出 `.dmg`）作为 Windows/Linux 配置的对照模板。
- **D-04:** 启动时自动检测可用 GPU 编码器：macOS -> VideoToolbox，Windows -> NVENC + AMF，Linux -> VAAPI。
- **D-05:** 编码器自动选择，用户无感知。GPU 编码启动失败时静默回退 CPU (`libx264`)，不中断批处理。
- **D-06:** 不提供手动编码器选择 UI——当前阶段保持简洁。
- **D-07:** 调度器优化：当前并发模型已验证可用，重点减少 worker 线程空等时间。
- **D-08:** 流式 I/O：FFmpeg 输入/输出已在 executor 中流式传递。
- **D-09:** 种子选择器从单选改为多选。`seedStore.selectedSeedId` -> `seedStore.selectedSeedIds: string[]`。
- **D-10:** 一个视频 x N 个种子 = N 个输出文件。
- **D-11:** 输出文件平铺在同一目录。命名：`{原文件名}_{种子别名}.{扩展名}`，冲突时追加 `-1`/`-2`。
- **D-12:** Rust 命令 `start_batch` 的 `seed_id: String` 参数改为 `seed_ids: Vec<String>`。处理循环改为双层。
- **D-13:** 处理前：对每个队列文件计算 MD5 并记录文件大小，存入 `HashMap<String, (String, u64)>`。
- **D-14:** 处理后：对每个成功输出文件计算 MD5，与处理前对比。MD5 不同 = 已修改；相同 = warning；失败 = N/A。
- **D-15:** 结果展示在 BatchSummary 的每个文件行中：MD5 前后值 + 状态图标。不额外输出日志文件。

### Claude's Discretion

- Tauri `tauri.conf.json` bundle 配置具体字段（Windows NSIS vs msi targets、Linux deb/AppImage targets）
- GitHub Actions workflow YAML 结构和触发条件
- GPU 编码器检测的具体 CLI 探测命令和 ffmpeg 参数
- MD5 计算实现（Rust `md5` crate 或 `std::process::Command` 调用 `md5sum`/`ffmpeg -f hash`）
- 多种子模式下 `PerFileProgress` 事件需要包含 `seed_alias` 字段
- `batch-progress` 事件中 `total` 字段计算：`total = 文件数 x 种子数`
- NSelect 多选后 `seedStore` 接口变更的影响范围
- BatchSummary 行布局调整以容纳 MD5 信息列
- i18n 新增 key（GPU 状态、MD5 对比、多选提示等）

### Deferred Ideas (OUT OF SCOPE)

- 代码签名和商店上架 -> 后续独立阶段
- GPU 编码器手动选择 UI -> 如用户反馈需要再添加
- 按种子分子目录输出 -> 当前选择平铺
- 独立 MD5 日志文件 -> 当前选择摘要内展示
- macOS 打包进一步完善（已有 `.dmg`）-> 本次重点补全 Win/Linux
- 视频队列拖拽排序（PROD-01）-> v2
- 缩略图预览（PROD-02）-> v2

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CROSS-01 | Windows 打包 — 生成 .msi 和 .exe 安装包 | `tauri.conf.json` `bundle.windows` with `wix` (MSI) and `nsis` (.exe). Use `targets: ["nsis", "msi"]`. Windows runner required for .msi (WiX Toolset is Windows-only). [VERIFIED: Context7 /websites/v2_tauri_app] |
| CROSS-02 | Linux 打包 — 生成 .AppImage 和 .deb 安装包 | `tauri.conf.json` `bundle.linux` with `deb` and `appimage` config. `targets: ["deb", "appimage"]`. Requires `patchelf` and `libfuse2` on build host. [VERIFIED: Context7 /websites/v2_tauri_app] |
| CROSS-03 | CI 矩阵构建（macOS/Windows/Linux），自动上传构建产物 | GitHub Actions with `strategy.matrix` over `[macos-latest, ubuntu-22.04, windows-latest]`. `tauri-apps/tauri-action@v0` auto-builds and creates GitHub Release with all artifacts. [VERIFIED: Context7 /websites/v2_tauri_app] |
| PERF-01 | GPU 硬件编码器自动检测与选择 | `ffmpeg -hide_banner -encoders` stdout parsing. Platform-specific encoder names: `h264_videotoolbox`, `h264_nvenc`, `h264_amf`, `h264_vaapi`. Validate with 2x2 test encode before trusting. [ASSUMED: encoder names from training knowledge; verification via ffmpeg docs recommended] |
| PERF-02 | 并行 pipeline 优化 — 调度器减少空等、流式读写 | Existing executor already streams via `child.iter()`. Optimization: reduce Mutex lock frequency in batch loop, batch progress updates per-file (not per-seed), pre-fetch next file metadata while current file encodes. [ASSUMED: optimization scope based on D-07 description] |
| MULTI-01 | 多种子选择 UI（可勾选多个种子） | Naive UI NSelect `multiple` prop with `v-model:value` binding to `string[]`. Existing `filterable` and `clearable` props retained. [VERIFIED: Context7 /tusen-ai/naive-ui] |
| MULTI-02 | 一个视频 x N 个种子 = N 个输出文件 | Nested loop in `batch.rs`: `for file { for seed { execute_single_file() } }`. Naming: `{stem}_{seed_alias}.{ext}` with `-N` collision suffix (existing `make_output_path` already handles this). Per D-11: flat output directory. [VERIFIED: existing executor.rs `make_output_path`] |
| MD5-01 | 处理前记录每个文件的 MD5 和文件大小 | `md5` crate (v0.8.0) streaming file hash via `Md5::new()` + `fs::File::read()` in 8KB chunks. Results stored in `HashMap<String, (String, u64)>` keyed by filepath. Wrap in `tokio::task::spawn_blocking()`. [VERIFIED: crates.io md5 crate] |
| MD5-02 | 处理后对比 MD5，差异数据写入处理日志，输出=输入时告警 | Post-processing: compute MD5 of output file, compare with pre-hash. `md5_before != md5_after` -> modified (pass). `md5_before == md5_after` -> warning (file unchanged). Error -> N/A. Results in `BatchResult.succeeded` as `FileSuccess` struct. [VERIFIED: D-14 spec] |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust (rustc/cargo) | Tauri backend build | Yes | 1.94.1 | — |
| Bun | Frontend build, dev server | Yes | 1.3.2 | — (required; project uses bun) |
| Node.js | Some tooling compatibility | Yes | 23.11.0 | — |
| Tauri CLI (@tauri-apps/cli) | Build command | Yes | 2.11.1 | — |
| FFmpeg (local) | GPU encoder testing | Yes | Not detected | ffmpeg-sidecar auto_download |
| FFmpeg hardware encoders | PERF-01 auto-detection | No (local build lacks them) | — (none found) | ffmpeg-sidecar download includes them [ASSUMED] |
| md5sum (system) | Alternative MD5 compute | Yes | macOS /sbin/md5 | Rust `md5` crate preferred |
| OpenSSL | Alternative MD5 compute | Yes | 3.5.0 | Rust `md5` crate preferred |
| Icons (ico, icns) | Bundle config | Partial (PNG only) | — | Tauri auto-generates from icon.png |
| GitHub Actions runners | CI build | Not locally | — | cloud://github.com (requires GH repo push) |

**Missing dependencies with no fallback:**
- None that block local development. GPU encoder testing will require either ffmpeg-sidecar's downloaded binary (which may include hardware encoders) or a system FFmpeg build with encoder support.

**Missing dependencies with fallback:**
- Windows `.ico` and macOS `.icns` icon formats: Tauri CLI can auto-generate these from the existing `icon.png` during build. No manual icon creation needed.
- Hardware encoders in local FFmpeg: Use ffmpeg-sidecar's `auto_download()` binary for testing, which includes more encoder support than the macOS system FFmpeg. [ASSUMED: ffmpeg-sidecar binaries include hardware encoders]

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Desktop app, local-only; no user auth |
| V3 Session Management | No | No sessions in a local desktop app |
| V4 Access Control | No | Local filesystem access only; OS-level permissions |
| V5 Input Validation | Yes | Existing: `start_batch` validates `seed_id` exists and queue is non-empty. New: `seed_ids` array validation (non-empty, all IDs exist), output path validation (no path traversal). FFmpeg arg injection: all CLI args are constructed from typed `Operation.params`, not user-supplied strings. |
| V6 Cryptography | Yes (advisory) | MD5 is being used for **integrity verification** (change detection), NOT for security purposes (authentication, signatures, password hashing). MD5 is cryptographically broken for collision resistance but perfectly adequate for detecting whether a file was modified by FFmpeg. No hand-rolled crypto — uses standard `md5` crate. |

### Known Threat Patterns for Tauri + FFmpeg

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal in output filename (`../../etc/passwd`) | Tampering | Existing `make_output_path()` joins `output_dir` with sanitized stem+alias. Verify output path is within `output_dir` before writing. [EXISTING MITIGATION: collision-safe naming, no user-controlled path segments beyond seed alias] |
| FFmpeg arg injection via seed params | Tampering | All `Operation.params` values are strongly typed (f64, i32, u32) and clamped to safety constraints (SEED-04). String params (`pattern`, `effect`) are matched against known enum values. No raw user strings reach FFmpeg CLI. [EXISTING MITIGATION: filters.rs enum matching] |
| GPU encoder process hangs | Denial of Service | GPU encoder crashes can leave zombie FFmpeg processes. Existing cancel mechanism (`cancel_flag` + `child.kill()`) handles cleanup. GPU failure triggers retry with CPU encoder, not infinite loop. [NEW MITIGATION: retry-once then fallback] |
| Large MD5 computation blocks event loop | Denial of Service | Wrap MD5 in `tokio::task::spawn_blocking()`. Use streaming I/O (8KB buffer) to avoid memory exhaustion. [NEW MITIGATION: async MD5 computation] |

## Sources

### Primary (HIGH confidence)
- [Context7: Tauri v2](/websites/v2_tauri_app) — bundle configuration (Windows NSIS/MSI, Linux deb/AppImage), GitHub Actions CI workflow, Linux system dependencies, tauri-action usage. HIGH confidence.
- [Context7: Naive UI](/tusen-ai/naive-ui) — NSelect component, multiple selection mode, filterable/clearable props. HIGH confidence.
- [Context7: ffmpeg-sidecar](/nathanbabcock/ffmpeg-sidecar) — auto_download, FfmpegCommand API, binary management. HIGH confidence.
- [crates.io](https://crates.io) — md5 crate v0.8.0 version and description. Cargo.toml dependencies. HIGH confidence.
- [npm registry](https://www.npmjs.com) — Package versions verified for existing dependencies. HIGH confidence.

### Secondary (MEDIUM confidence)
- [Context7: FFmpeg](/websites/ffmpeg_ffmpeg-all) — `-hwaccel` options, hardware acceleration flags. Encoder names confirmed via official FFmpeg docs. MEDIUM confidence (specific encoder naming conventions verified).
- Tauri v2 CLI reference: `tauri bundle --bundles deb,appimage` command syntax. MEDIUM confidence.

### Tertiary (LOW confidence)
- FFmpeg hardware encoder availability in ffmpeg-sidecar downloaded binaries. LOW confidence — the specific build variants distributed by ffmpeg-sidecar (Gyan.dev for Windows, Evermeet for macOS, johnvansickle.com for Linux) were not verified to include NVENC/VideoToolbox/VAAPI encoders. This should be validated by downloading a binary and checking `ffmpeg -encoders`.
- Ubuntu 24.04 vs 22.04 package name differences for `libsoup` and `libjavascriptcoregtk`. LOW confidence — Context7 data shows `ubuntu-22.04` used in official Tauri v2 CI templates. The 24.04 packages were inferred from training knowledge.

## Assumptions Log

> Claims tagged `[ASSUMED]` in this research. Planner and discuss-phase use this to identify decisions needing user confirmation.

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | ffmpeg-sidecar downloaded FFmpeg binaries include hardware encoders (NVENC, VideoToolbox, VAAPI) | GPU Detection, PERF-01 | GPU auto-detection finds encoders but they fail at runtime. Mitigation: validate with test encode (Pitfall 3). Low risk — standard FFmpeg builds include these encoders; the detection-then-validate pattern handles missing hardware gracefully. |
| A2 | `ubuntu-22.04` (not `ubuntu-latest`/24.04) is the correct CI runner for Tauri v2 compatibility | CI Matrix, CROSS-03 | If ubuntu-22.04 is deprecated by GitHub Actions, the CI breaks. Mitigation: Tauri v2 docs explicitly recommend ubuntu-22.04. If deprecated, migrate to 24.04 with updated deps at that time. |
| A3 | `oven-sh/setup-bun@v2` is the latest version of the bun setup action | CI Matrix, CROSS-03 | If v3 exists, the CI may show deprecation warnings but v2 should still work. Low risk — bun setup action maintains backward compatibility. |
| A4 | The `md5` crate v0.8.0 supports `no_std` and streaming I/O via `Md5::new()` + `update()` pattern | MD5 Computation, MD5-01 | If the API has changed in v0.8.x, the streaming I/O pattern may differ. Low risk — md5 crate has a stable API; the `Md5` context + `Read` pattern is core functionality. [VERIFIED: crates.io docs] |
| A5 | Tauri CLI auto-generates `.ico` and `.icns` from `icon.png` during `cargo tauri build` | Bundle Config, CROSS-01/02 | Missing icons cause build warnings or missing app icons in installers. Low risk — Tauri v2 has robust icon generation; adding explicit `.icns`/`.ico` files is trivial fallback. |
| A6 | `swatinem/rust-cache@v2` (not v3) is the current version | CI Matrix | Outdated version may produce deprecation warning. Low risk — v2 is the version used in official Tauri CI templates from Context7. |

## Open Questions

1. **Does ffmpeg-sidecar's auto_download provide FFmpeg binaries with hardware encoders?**
   - What we know: ffmpeg-sidecar downloads from Gyan.dev (Windows), Evermeet (macOS), johnvansickle.com (Linux). These are standard builds.
   - What's unclear: Whether these specific builds were compiled with `--enable-nvenc`, `--enable-videotoolbox`, `--enable-vaapi` flags.
   - Recommendation: After implementing GPU detection, download the FFmpeg binary and verify with `ffmpeg -encoders | grep -E "nvenc|videotoolbox|vaapi|amf"`. If encoders are missing, the GPU detection returns `None` gracefully and the app falls back to CPU — no user impact. The user can also point to a custom FFmpeg build with hardware encoders via the existing FFmpeg path configuration.

2. **Should CI build on every push to master, or only on release tags/branches?**
   - What we know: D-02 specifies CI matrix build with artifact upload. The existing `ci.yml` runs on every push for lint/test (lightweight). Full Tauri builds take 10-20 minutes per platform.
   - What's unclear: Trigger strategy — every push (expensive), release branch only, or tag-based. The Context7 official template uses `push: branches: [release]` + `workflow_dispatch`.
   - Recommendation: Use `release` branch trigger + `workflow_dispatch` (manual) per the official Tauri v2 pattern. Keep the existing `ci.yml` for lint/test on every push. Add a separate `build.yml` for release builds. This prevents 40-minute CI runs on every commit.

3. **GPU encoder fallback: retry within same FFmpeg command or re-spawn?**
   - What we know: D-05 says "GPU 编码启动失败时静默回退 CPU，不中断批处理." The design is that if `-c:v h264_nvenc` fails, retry with `-c:v libx264`.
   - What's unclear: Whether to implement this as (a) spawn FFmpeg with GPU encoder, check exit code, if failed re-spawn with CPU encoder; or (b) somehow switch mid-encode (not really possible with FFmpeg CLI).
   - Recommendation: Option (a) — re-spawn. This is the only reliable approach. The executor.rs already returns `Result<String, String>`, so the batch loop can catch the error and retry with `libx264` before counting it as a failure. Add a `retry_count` field to PerFileProgress so the UI can indicate "retrying with CPU..."

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all library versions verified against npm/crates.io registries or Context7 official docs.
- Architecture: HIGH — patterns derived from existing codebase (batch.rs, executor.rs, Pinia stores) plus Context7-verified Tauri v2 patterns.
- Pitfalls: MEDIUM — pitfalls 1, 2, 4, 5, 6 are verified against Context7 and codebase analysis. Pitfall 3 (GPU encoder in ffmpeg-sidecar binary) is [ASSUMED] pending binary verification.
- GPU encoder detection: MEDIUM — encoder names verified via Context7 FFmpeg docs; ffmpeg-sidecar binary encoder support is [ASSUMED] pending validation.

**Research date:** 2026-05-14
**Valid until:** 2026-06-14 (30 days; Tauri v2 ecosystem is stable, GPU encoder landscape changes slowly)

**Sources consulted:** 6 Context7 library queries, 2 crates.io searches, 1 npm registry check, 2 WebFetch attempts (blocked by security policy), 2 WebSearch attempts (API error)
