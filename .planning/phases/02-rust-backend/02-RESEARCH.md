# Phase 2: Rust Backend - Research

**Researched:** 2026-05-13
**Domain:** Tauri 2.x IPC commands, Rust state management, FFmpeg/ffprobe process orchestration, weighted random generation, tauri-plugin-store persistence
**Confidence:** HIGH

## Summary

Phase 2 builds all domain operations as typed Tauri IPC commands backed by Rust-managed authoritative state. The backend orchestrates: (1) weighted random seed generation across 7 operation types with safety-constrained parameters, (2) video import with ffprobe metadata extraction, (3) an in-memory video queue with disk persistence, and (4) batch FFmpeg processing with per-file failure isolation and cancellation via `AtomicBool` + `FfmpegChild::kill()`.

The existing Phase 1 codebase provides solid patterns to follow: `#[tauri::command] async fn` with `tauri::generate_handler![]` registration in `lib.rs`, `app.store()` for persistence, `AppHandle::emit()` for events, and `AtomicBool` + `OnceLock<Mutex<>>` for cancelable operations. Phase 2 extends these patterns across four new command modules.

**Primary recommendation:** Use `AppHandle::manage(Mutex::new(AppState))` for authoritative in-memory state (video queue, batch processing status), `app.store("seeds.json")` / `app.store("queue.json")` for persistence, `FfmpegCommand::new_with_path(ffmpeg_path)` for FFmpeg execution with the stored binary path, and `ffprobe_path()` + `std::process::Command` for JSON metadata extraction.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Pure random generation. Users cannot edit operation chain parameters. Operation chain is transparent but read-only; display summary only (e.g., "ripple + frame drop + GOP"). Copy-and-re-randomize supported later.
- **D-02:** 7 operation types with weighted random: mathematical overlay (ripple/stripes/concentric) highest weight (~30%), remaining types evenly distributed. Rationale: math overlay is most fingerprint-effective with minimal visual artifacts.
- **D-03:** 3-7 random steps per seed. Step count is a key diversity dimension.
- **D-04:** Auto-alias default: timestamp format. Users can manually rename later (SEED-05).
- **D-05:** Full persistence: seed list, video queue (with ffprobe metadata), output directory preference, concurrency preference -- all via tauri-plugin-store. Restore to pre-close state on restart.
- **D-06:** Queue video path validity check. Moved/deleted files marked invalid (preserve metadata, notify user).
- **D-07:** Crash recovery: on restart detect incomplete processing, mark queue as "pending" state, preserve completed output files. User manually restarts batch.
- **D-08:** User-selectable concurrency 1-4, default 1. Default 1 because single FFmpeg process uses ~500MB-1GB memory.
- **D-09:** Concurrency preference persisted. Remember user's choice.
- **D-10:** Cancel behavior: kill all FFmpeg processes, clean up incomplete/in-progress output files. Completed files preserved. Queue returns to pending state.
- **D-11:** Single-file failure isolation: one file failure auto-skipped, remaining files continue.
- **D-12:** Supported formats: mp4, mov, avi, mkv, webm, flv, wmv. Filter by extension + ffprobe final validation.
- **D-13:** No hard file size limit. Only check available disk space for output.
- **D-14:** ffprobe validation on import (readable video stream). Invalid files rejected with specific error message.
- **D-15:** Allow duplicate file path imports into queue. Users may process same source with different seeds.
- **D-16:** Naming conflict: auto-append numeric suffix. If `{original}_{seed_alias}.mp4` exists, become `{original}_{seed_alias}-1.mp4`, `-2.mp4`, etc. Silent handling, no batch interruption.

### Claude's Discretion

- Exact weight percentages for the 7 operation types (beyond ~30% for math overlay)
- Specific safety constraint parameter values (opacity <= 0.15, pixel shift <= 3px, frame drop interval >= 15 already locked by REQUIREMENTS.md SEED-04)
- ffprobe metadata parsing: specific fields and implementation approach
- FFmpeg filter chain construction for the 7 operation types
- Rust module organization (command file granularity under `commands/`)
- IPC command naming and signature design

### Deferred Ideas (OUT OF SCOPE)

None -- all discussion was within Phase 2 scope.

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SEED-01 | One-click random seed generation (3-7 step operation chain) | rand 0.9.x weighted distribution; serde JSON serialization; Section: Seed Generation Architecture |
| SEED-02 | 7 operation types: math overlay, pixel shift, frame drop, GOP modify, metadata erase, audio tweak, remux | FFmpeg filter chains for each type documented in Architecture Patterns |
| SEED-03 | Operation chain format: [op type] + [start frame] + [duration frames] + [params] | Serde enum + struct design; Section: Core Data Types |
| SEED-04 | Auto-generated parameters with safety constraints (opacity <=0.15, shift <=3px, frame drop >=15) | Constraint constants in Rust; rand::Rng with clamped ranges |
| SEED-05 | Seed alias support (auto timestamp, manual rename) | chrono timestamp formatting; store persistence |
| SEED-06 | Seed list management (view, rename, delete, copy) | Vec<Seed> in managed state; store serialization |
| IMPORT-01 | Drag-and-drop video import | Tauri drag-drop event handled in Phase 3 frontend; Phase 2 provides `import_video` command |
| IMPORT-02 | File picker video import | tauri-plugin-dialog already initialized; Phase 2 command accepts path from frontend |
| QUEUE-01 | Video queue display (filename, duration, resolution, size) | ffprobe JSON parsing extracts all fields; VideoEntry struct with serde |
| QUEUE-02 | Video queue management (remove single, clear all) | Mutex<Vec<VideoEntry>> managed state; store sync |
| BATCH-01 | Select seed, set output dir, process all queued videos | BatchConfig struct; tokio::spawn_blocking for CPU-intensive FFmpeg execution |
| BATCH-03 | Cancel in-progress batch processing | AtomicBool cancel flag; FfmpegChild::kill(); incomplete file cleanup |
| BATCH-04 | Single-file failure isolation | Per-file Result handling in batch loop; continue on Err |
| OUTPUT-01 | Select output directory (default ~/Videos/sandwich-output/) | tauri-plugin-dialog save dialog; persisted preference; dirs crate for home dir |
| OUTPUT-02 | Output naming: {original}_{seed_alias}.{ext} | Path manipulation in Rust; suffix collision detection with while-loop increment |

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Seed generation (random) | Rust Backend | -- | Random number generation, weighted distribution logic, and safety constraint enforcement belong in Rust for performance and correctness |
| Seed persistence (JSON) | Rust Backend | -- | tauri-plugin-store writes from Rust; no frontend involvement in serialization |
| Seed list management (CRUD) | Rust Backend | Vue Frontend (display) | Rust owns authoritative state; frontend calls commands and renders results |
| ffprobe metadata extraction | Rust Backend | -- | Process spawning, JSON parsing, and error handling must happen in backend; frontend only receives parsed structs |
| Video queue state (authoritative) | Rust Backend | -- | Managed via `AppHandle::manage(Mutex<Vec<VideoEntry>>)`; all mutations through commands |
| Video queue persistence | Rust Backend | -- | tauri-plugin-store serializes the queue on every mutation |
| Video import validation | Rust Backend | Vue Frontend (UX) | File dialog and drag-drop are frontend concerns; path is passed to Rust for ffprobe validation |
| Batch FFmpeg execution | Rust Backend | -- | CPU-intensive; must run in `tokio::spawn_blocking` to avoid UI thread blocking |
| Batch progress emission | Rust Backend | Vue Frontend (listener) | Rust emits events via `AppHandle::emit()`; frontend listens |
| Batch cancellation | Rust Backend | -- | `AtomicBool` flag checked between files; `FfmpegChild::kill()` for immediate termination |
| Output file naming | Rust Backend | -- | Path construction and collision detection are deterministic logic; no UI needed |
| Output directory management | Rust Backend | Vue Frontend (dialog) | Frontend opens dialog; Rust persists preference and creates directories |
| Concurrency control | Rust Backend | -- | Semaphore or channel-based concurrency limiting in Rust; frontend only displays current setting |

## Standard Stack

### Core (Already in Cargo.toml)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.11.1 | Desktop framework | Required by project constraints |
| ffmpeg-sidecar | 2.5.1 | FFmpeg/ffprobe binary management and execution | Already in Phase 1; provides `FfmpegCommand`, `ffprobe_path()`, `ffmpeg_is_installed()` |
| serde | 1.0.149 | Serialization/deserialization | All IPC command args and returns; `#[serde(rename_all = "camelCase")]` for TypeScript interop |
| serde_json | 1.0.149 | JSON parsing | ffprobe output parsing; store value construction |
| tokio | 1.52.3 | Async runtime | `spawn_blocking` for FFmpeg; async Tauri commands |
| tauri-plugin-store | 2.4.3 | Key-value persistence | D-05 requires full persistence; `app.store()` API already in use |
| tauri-plugin-shell | 2.3.5 | Process execution (unused in Phase 2) | Already registered; Phase 2 uses ffmpeg-sidecar not raw shell |
| anyhow | 1.0.102 | Error handling | Ergonomic `Result<T>` with context |
| chrono | 0.4 | Timestamps | Seed creation timestamps; alias generation; file modification times |
| rstest | 0.26.1 | Parameterized tests | Table-driven seed generation tests |

### New Dependencies for Phase 2

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rand | 0.9.x | Random number generation | Weighted distribution selection, random parameter ranges, seed shuffling. De facto standard Rust RNG crate. [VERIFIED: crates.io] |
| uuid | 1.x | Unique seed IDs | Stable unique identifiers for seeds; survives renames. Feature: `v4` for random UUIDs. [VERIFIED: crates.io] |

### Supporting (Phase 1 crates reused but not primary focus)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| reqwest | 0.12 | HTTP client | Not used in Phase 2 but already in Cargo.toml from Phase 1 download |
| fs2 | 0.4.3 | File system utilities | Disk space checking for D-13 (no hard limit, check available space) |
| futures-util | 0.3 | Stream utilities | If async streaming needed for FFmpeg stderr |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rand 0.9.x | fastrand | fastrand is lighter but lacks weighted distribution (`rand::distributions::WeightedIndex`). rand is the ecosystem standard. |
| uuid v4 | nanoid | nanoid produces shorter strings (URL-friendly) but uuid is the Rust ecosystem standard, has better serde support, and seed IDs don't need to be URL-safe. |
| app.store() per item | Single JSON blob in store | app.store() with individual `.set()` calls per key is simpler but less efficient for large collections; recommended to serialize the full Vec as a single JSON string value per store file. |

**Installation:**
```bash
cargo add rand@0.9 --manifest-path src-tauri/Cargo.toml
cargo add uuid@1 --features v4 --manifest-path src-tauri/Cargo.toml
```

**Version verification:**
```bash
cargo search rand --limit 1      # 0.9.x confirmed
cargo search uuid --limit 1      # 1.x confirmed
```

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     VUE FRONTEND (webview)                       │
│  invoke('generate_seed')         invoke('import_video', {path})  │
│  invoke('start_batch', config)   invoke('cancel_batch')          │
│  .on('batch-progress', ...)      .on('batch-complete', ...)      │
└──────────────┬──────────────────────────────────────────────────┘
               │  IPC (Tauri Invoke / Event System)
               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      TAURI COMMAND LAYER                         │
│                                                                  │
│  commands/seed.rs        commands/import.rs                      │
│  ├─ generate_seed()      ├─ import_video(path)                   │
│  ├─ rename_seed()        ├─ validate_video_with_ffprobe()        │
│  ├─ delete_seed()        └─ extract_metadata()                   │
│  ├─ copy_seed()                                                 │
│  └─ list_seeds()         commands/queue.rs                       │
│                           ├─ remove_from_queue()                  │
│  commands/mod.rs          ├─ clear_queue()                       │
│  (re-exports all)         └─ get_queue()                         │
│                                                                  │
│  commands/batch.rs                                               │
│  ├─ start_batch() ────► tokio::spawn_blocking ────► FFmpeg      │
│  ├─ cancel_batch() ───► AtomicBool::store(true)                  │
│  └─ get_batch_status()                                           │
└──────────────┬──────────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────────┐
│                    MANAGED STATE (AppHandle::manage)              │
│                                                                  │
│  AppState {                                                      │
│    seeds: Mutex<Vec<Seed>>,                                      │
│    queue: Mutex<Vec<VideoEntry>>,                                │
│    batch_state: Mutex<BatchState>,                                │
│  }                                                               │
│                                                                  │
│  BatchState {                                                    │
│    cancel_flag: AtomicBool,                                      │
│    active_children: Vec<FfmpegChild>,  // for kill on cancel     │
│    status: BatchStatus,  // Idle | Running | Cancelling          │
│    current_progress: BatchProgress,                              │
│  }                                                               │
└──────────────┬──────────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────────┐
│                   PERSISTENCE (tauri-plugin-store)                │
│                                                                  │
│  seeds.json ─── Vec<Seed> serialized as JSON                      │
│  queue.json ─── Vec<VideoEntry> serialized as JSON                │
│  sandwich-config.json ─── output_dir, concurrency, ffmpeg_path   │
│  (ffmpeg-config.json ─── Phase 1, unchanged)                     │
└──────────────┬──────────────────────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────────────────────┐
│                   FFMPEG / FFPROBE (via ffmpeg-sidecar)          │
│                                                                  │
│  ffprobe_path() ──► std::process::Command                        │
│    "ffprobe -v quiet -print_format json -show_format             │
│     -show_streams {input}"                                       │
│                                                                  │
│  FfmpegCommand::new_with_path(ffmpeg_path)                        │
│    .input(source) .filter_complex("...") .output(out)             │
│    .spawn() ──► FfmpegChild                                      │
│    .iter() ──► FfmpegEvent stream (progress, logs)               │
│    .kill() ──► terminate on cancel                                │
└─────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src-tauri/src/
├── main.rs                    # Entry point (unchanged)
├── lib.rs                     # Tauri builder, plugin init, command registration, state init
├── commands/
│   ├── mod.rs                 # Re-exports all command modules
│   ├── ffmpeg.rs              # Phase 1: FFmpeg detection/verification (unchanged)
│   ├── download.rs            # Phase 1: FFmpeg download (unchanged)
│   ├── seed.rs                # NEW: seed generation, CRUD, list
│   ├── import.rs              # NEW: video import, ffprobe validation, metadata extraction
│   ├── queue.rs               # NEW: video queue management (remove, clear, get)
│   └── batch.rs               # NEW: batch processing, cancel, progress emission
├── state.rs                   # NEW: AppState, BatchState, BatchProgress structs
├── models/
│   ├── mod.rs                 # Re-exports model types
│   ├── seed.rs                # Seed, Operation, OperationType, SafetyConstraint types
│   ├── video.rs               # VideoEntry, VideoMetadata, VideoStatus types
│   └── batch.rs               # BatchConfig, BatchResult, FileResult types
└── ffmpeg/
    ├── mod.rs                 # Re-exports ffmpeg utilities
    ├── filters.rs             # Filter chain construction for 7 operation types
    ├── probe.rs               # ffprobe invocation and JSON parsing
    └── executor.rs            # FfmpegCommand construction, progress parsing, cancel
```

### Pattern 1: Managed State with AppHandle

**What:** Use `app.manage(Mutex::new(AppState {...}))` in `lib.rs` setup, access via `State<'_, Mutex<AppState>>` in commands.

**When to use:** All Phase 2 state (seeds, queue, batch status).

**Example:**
```rust
// Source: https://v2.tauri.app/develop/state-management [VERIFIED]

// In lib.rs setup:
use std::sync::Mutex;
struct AppState {
    seeds: Vec<Seed>,
    queue: Vec<VideoEntry>,
}

app.manage(Mutex::new(AppState::default()));

// In command:
#[tauri::command]
async fn generate_seed(
    state: tauri::State<'_, Mutex<AppState>>,
    app: tauri::AppHandle,
) -> Result<Seed, String> {
    let mut state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let seed = Seed::generate_random()?;
    state.seeds.push(seed.clone());
    // Persist to store
    persist_seeds(&app, &state.seeds)?;
    Ok(seed)
}
```

### Pattern 2: tauri-plugin-store Persistence

**What:** Serialize full collections to store as JSON values. Load on startup in `setup()`.

**When to use:** Seed list, video queue, config preferences.

**Example:**
```rust
// Source: existing code in src-tauri/src/commands/ffmpeg.rs (verify_ffmpeg) [VERIFIED]

fn persist_seeds(app: &AppHandle, seeds: &[Seed]) -> Result<(), String> {
    let store = app.store("seeds.json")
        .map_err(|e| format!("Failed to open seeds store: {}", e))?;
    let json = serde_json::to_value(seeds)
        .map_err(|e| format!("Serialization error: {}", e))?;
    store.set("seeds", json);
    store.save().map_err(|e| format!("Failed to save seeds: {}", e))?;
    Ok(())
}

fn load_seeds(app: &AppHandle) -> Result<Vec<Seed>, String> {
    let store = app.store("seeds.json")
        .map_err(|e| format!("Failed to open seeds store: {}", e))?;
    match store.get("seeds") {
        Some(value) => serde_json::from_value(value)
            .map_err(|e| format!("Deserialization error: {}", e)),
        None => Ok(Vec::new()), // First launch — empty seed list
    }
}
```
[VERIFIED: existing codebase pattern in `ffmpeg.rs` line 186-188]

### Pattern 3: Batch Processing with Cancellation

**What:** `AtomicBool` cancel flag checked between files. `FfmpegChild::kill()` for immediate termination of running process.

**When to use:** BATCH-03 (cancel), BATCH-04 (failure isolation).

**Example:**
```rust
// Source: existing code in src-tauri/src/commands/download.rs (cancel pattern) [VERIFIED]
// Extended with ffmpeg-sidecar FfmpegChild::kill() from docs.rs [VERIFIED]

use std::sync::atomic::{AtomicBool, Ordering};
use ffmpeg_sidecar::command::FfmpegCommand;

struct BatchContext {
    cancel_flag: AtomicBool,
}

#[tauri::command]
async fn start_batch(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    seed_id: String,
    output_dir: String,
) -> Result<(), String> {
    let cancel_flag = AtomicBool::new(false);
    // Store cancel_flag in managed state for cancel_batch() to access

    let queue = /* clone queue from state */;
    for entry in queue {
        if cancel_flag.load(Ordering::SeqCst) {
            // Clean up incomplete output file
            let _ = std::fs::remove_file(&expected_output_path);
            break;
        }

        let result = tokio::task::spawn_blocking(move || {
            let mut child = FfmpegCommand::new_with_path(&ffmpeg_path)
                .input(&entry.path)
                .args(build_filter_args(&seed))
                .output(&output_path)
                .spawn()
                .map_err(|e| format!("FFmpeg spawn failed: {}", e))?;

            // Iterate events for progress
            for event in child.iter().map_err(|e| e.to_string())? {
                if cancel_flag.load(Ordering::SeqCst) {
                    child.kill().ok(); // Forcible termination per D-10
                    return Err("Cancelled".into());
                }
                // Emit progress to frontend
                if let FfmpegEvent::Progress(p) = event {
                    let _ = app.emit("batch-progress", p);
                }
            }
            child.wait().map_err(|e| format!("FFmpeg wait error: {}", e))?;
            Ok(())
        }).await.map_err(|e| format!("Join error: {}", e))?
        .map_err(|e| {
            // D-11: Single-file failure isolation — log error, continue
            let _ = app.emit("batch-file-error", FileError { file: entry.name.clone(), error: e });
            // Continue to next file
        });
    }
    Ok(())
}
```

### Pattern 4: ffprobe Metadata Extraction

**What:** Run `ffprobe` as a subprocess with JSON output, parse into typed struct.

**When to use:** IMPORT-01/02 (import with validation), QUEUE-01 (display metadata).

**Example:**
```rust
// Source: ffmpeg-sidecar ffprobe_path() docs [VERIFIED]

use ffmpeg_sidecar::ffprobe::ffprobe_path;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
struct FfprobeOutput {
    format: FfprobeFormat,
    streams: Vec<FfprobeStream>,
}

#[derive(Debug, Clone, Deserialize)]
struct FfprobeFormat {
    filename: String,
    duration: String,    // "123.456000"
    size: String,        // "12345678"
}

#[derive(Debug, Clone, Deserialize)]
struct FfprobeStream {
    codec_type: String,  // "video", "audio"
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<String>,
}

fn probe_video(path: &str) -> Result<VideoMetadata, String> {
    let output = Command::new(ffprobe_path())
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .map_err(|e| format!("ffprobe execution failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // D-14: Invalid file — reject with specific error
        return Err(format!("ffprobe failed (invalid video?): {}", stderr));
    }

    let probe: FfprobeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("ffprobe JSON parse error: {}", e))?;

    // Verify at least one video stream (D-14)
    let has_video = probe.streams.iter().any(|s| s.codec_type == "video");
    if !has_video {
        return Err("File has no video stream".into());
    }

    // Extract metadata...
    Ok(VideoMetadata { /* ... */ })
}
```

### Pattern 5: Weighted Random Seed Generation

**What:** Use `rand::distributions::WeightedIndex` for D-02's weighted operation type selection, `rand::Rng` for random parameters within safety-constrained ranges.

**When to use:** SEED-01 (generate seed), SEED-02 (7 types), SEED-03 (operation chain), SEED-04 (safety constraints).

**Example:**
```rust
// Source: rand crate docs (0.9.x) [VERIFIED: crates.io]

use rand::prelude::*;
use rand::distributions::WeightedIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationType {
    MathOverlay,    // ~30% weight — ripple/stripes/concentric
    PixelShift,     // even distribution among remainder
    FrameDrop,
    GopModify,
    MetadataErase,
    AudioTweak,
    Remux,
}

fn generate_seed() -> Seed {
    let mut rng = rand::rng();

    // D-02: Math overlay ~30%, remaining 6 types evenly split ~11.7% each
    let weights = [30, 12, 12, 12, 12, 11, 11]; // sum to 100
    let dist = WeightedIndex::new(&weights).unwrap();

    // D-03: 3-7 random steps
    let step_count = rng.random_range(3..=7);

    let mut operations = Vec::with_capacity(step_count);
    for _ in 0..step_count {
        let op_type = match dist.sample(&mut rng) {
            0 => OperationType::MathOverlay,
            1 => OperationType::PixelShift,
            // ... etc
            _ => unreachable!(),
        };

        let op = generate_operation(&mut rng, op_type);
        operations.push(op);
    }

    // D-04: Auto-alias with timestamp
    let alias = chrono::Utc::now().format("seed_%Y%m%d_%H%M%S").to_string();

    Seed {
        id: uuid::Uuid::new_v4().to_string(),
        alias,
        operations,
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

fn generate_operation(rng: &mut impl Rng, op_type: OperationType) -> Operation {
    // SEED-04: Safety-constrained parameters
    match op_type {
        OperationType::MathOverlay => {
            let pattern = match rng.random_range(0..3) {
                0 => "ripple",
                1 => "stripes",
                _ => "concentric",
            };
            let opacity = rng.random_range(0.03..=0.15); // SEED-04: ≤ 0.15
            let frequency = rng.random_range(20..=200);
            Operation {
                op_type,
                start_frame: 0,
                duration_frames: 0, // entire video
                params: serde_json::json!({
                    "pattern": pattern,
                    "opacity": opacity,
                    "frequency": frequency,
                }),
            }
        }
        OperationType::PixelShift => {
            let dx = rng.random_range(-3i32..=3); // SEED-04: ≤ |3px|
            let dy = rng.random_range(-3i32..=3);
            Operation {
                op_type,
                start_frame: 0,
                duration_frames: 0,
                params: serde_json::json!({ "dx": dx, "dy": dy }),
            }
        }
        OperationType::FrameDrop => {
            let interval = rng.random_range(15..=60); // SEED-04: ≥ 15
            Operation {
                op_type,
                start_frame: rng.random_range(0..100),
                duration_frames: rng.random_range(30..300),
                params: serde_json::json!({ "interval": interval }),
            }
        }
        // ... all 7 types
    }
}
```

### Anti-Patterns to Avoid

- **Storing `FfmpegChild` in managed state without `Arc<Mutex<>>`:** `FfmpegChild` is not `Send + Sync` by default. Wrap in `Mutex` or use `tokio::sync::Mutex` if accessed across await points.
- **Serializing each seed/queue item as an individual store key:** `app.store().set("seed_1", ...)`, `app.store().set("seed_2", ...)` -- does not scale and makes atomic batch updates impossible. Use a single key holding the serialized `Vec<T>`.
- **Loading store on every command invocation:** Read the store once on startup (in `setup()`) into managed state, then write-through on mutations. Avoid `app.store()` in hot-path commands.
- **Using `std::sync::Mutex` across await points:** Tauri commands are async. If you hold a `std::sync::Mutex` lock and then `.await`, you risk deadlock. Use `tokio::sync::Mutex` for state accessed across await boundaries, or scope the lock to sync-only sections.
- **Building FFmpeg filter strings via string concatenation:** Use a dedicated filter builder to ensure correct escaping of filter graph syntax (colons must be escaped as `\:` in filter arguments).
- **Assuming FFmpeg is in PATH:** Phase 1 may have downloaded FFmpeg to a custom directory. Always use `FfmpegCommand::new_with_path(stored_ffmpeg_path)` — never `new()` which only checks PATH.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Random number generation | Custom RNG or `fastrand` with manual weighting | `rand` 0.9.x with `WeightedIndex` | WeightedIndex handles edge cases (zero weights, floating precision), provides statistically sound sampling |
| FFmpeg process management | `std::process::Command` raw + manual stderr parsing | `ffmpeg-sidecar::FfmpegCommand` | Structured progress parsing (`FfmpegProgress`), cross-platform binary detection, stdin control (`quit`/`kill`), log parsing. 1098 documented snippets. |
| JSON serialization for store | Manual JSON string building | `serde_json::to_value()` + `store.set()` | Handles nested objects, escaping, Unicode; composable with `#[derive(Serialize)]` |
| File path collision detection | Manual existence check with race condition | Loop `while path.exists() { increment_suffix() }` then create | The approach is simple enough to self-implement, but use `std::fs::exists` (or `Path::try_exists()` on nightly) to avoid TOCTOU races between check and write |
| Video metadata extraction | Grepping ffprobe text output | `ffprobe -print_format json -show_format -show_streams` + `serde_json` | JSON output is stable, structured, and includes all needed fields. Parsing text output is fragile across ffprobe versions. |
| Concurrency limiting | Hand-rolled semaphore | `tokio::sync::Semaphore` | Standard, well-tested, integrates with tokio runtime |

**Key insight:** FFmpeg process management is the most dangerous thing to hand-roll. ffmpeg-sidecar handles: cross-platform binary path resolution, stderr parsing into typed events (progress, logs, errors), stdin control for graceful quit, platform-specific signal handling, and avoiding stderr pipe deadlocks. Building this from `std::process::Command` would require hundreds of lines of error-prone code and would miss edge cases on Windows.

## Common Pitfalls

### Pitfall 1: FFmpeg stderr Pipe Deadlock

**What goes wrong:** When spawning FFmpeg via `std::process::Command`, if stderr is piped but never consumed, FFmpeg blocks when the OS pipe buffer fills (~64KB), hanging indefinitely. This is the most common FFmpeg integration bug.

**Why it happens:** FFmpeg writes verbose progress to stderr. Without a reader consuming stderr, the pipe buffer fills, blocking the FFmpeg process.

**How to avoid:** Always use `ffmpeg-sidecar`'s `.iter()` method which continuously drains stderr and parses events. If using raw `std::process::Command`, spawn a tokio task to read stderr to completion.

**Warning signs:** FFmpeg process starts and immediately hangs, CPU at 0%, progress never updates.

### Pitfall 2: tauri-plugin-store JSON Value Nesting

**What goes wrong:** `store.set("key", serde_json::Value::Array(...))` followed by `store.get("key")` returns a `serde_json::Value` that may be deeply nested and require manual traversal. Attempting to store complex objects as nested keys (e.g., `store.set("seeds.0.alias", ...)`) doesn't work because the store API only supports top-level keys.

**Why it happens:** tauri-plugin-store is a flat key-value store. It stores `serde_json::Value` values but doesn't support dotted key paths.

**How to avoid:** Serialize entire collections as a single JSON array under one key (e.g., key "seeds" → `Value::Array([...])`), deserialize with `serde_json::from_value::<Vec<Seed>>(value)`.

**Warning signs:** Runtime errors about missing keys; data not appearing after save/load cycle.

### Pitfall 3: Mutex Poisoning from FFmpeg Panics

**What goes wrong:** If FFmpeg execution panics (e.g., unwrap on unexpected stderr format) while holding a `std::sync::Mutex` lock, the mutex becomes poisoned. All subsequent lock attempts fail, rendering the state inaccessible for the rest of the app lifetime.

**Why it happens:** `std::sync::Mutex` uses poisoning to signal that invariants may have been violated during a panic.

**How to avoid:** (a) Use `tokio::sync::Mutex` for state accessed across await points (no poisoning); (b) Never hold a `std::sync::Mutex` lock across FFmpeg spawn/iteration; extract needed data, release lock, then spawn; (c) Use `Mutex::lock().unwrap_or_else(|e| e.into_inner())` to recover from poisoning if using `std::sync::Mutex`.

**Warning signs:** All commands start returning "Lock error: PoisonError" after a batch failure.

### Pitfall 4: FFmpeg Filter String Escaping

**What goes wrong:** FFmpeg filter graphs use `:` as separator and `,` as chain separator. Parameter values containing these characters (like expressions with division `a/b` or complex filter args) must be escaped. Missing escaping causes "No such filter" or "Option not found" errors.

**Why it happens:** The FFmpeg filter graph parser interprets unescaped `:` and `,` as structural delimiters.

**How to avoid:** In complex filter graphs, escape `:` as `\:` and `,` as `\,`. The `geq` expressions are especially prone since they contain operators. Build a small `EscapeFilterArg` helper that escapes these characters. Test filter strings with `--print_command` or `-report` before integration.

**Warning signs:** `ffmpeg` errors like "No such filter: 'geq=lum=128'" (when the expression contains an unescaped colon).

### Pitfall 5: AtomicBool Visibility in Spawned Tasks

**What goes wrong:** The cancel `AtomicBool` is set in one thread (the cancel command) but the batch processing loop in `spawn_blocking` doesn't see the updated value immediately, causing cancellation delay or failure.

**Why it happens:** Using `Ordering::Relaxed` instead of `Ordering::SeqCst` or `Ordering::Acquire` can cause visibility delays on weakly-ordered architectures (ARM). On x86, this is rarely observed due to strong memory ordering, but the app targets macOS ARM (Apple Silicon) where it matters.

**How to avoid:** Always use `Ordering::SeqCst` for both `store` and `load` on cancel flags. This is the pattern already established in `download.rs` (verified: line 83 uses `Ordering::SeqCst`).

**Warning signs:** Cancel command succeeds (no error) but batch continues processing for several more seconds.

## Code Examples

Verified patterns from official sources:

### ffprobe JSON Metadata Extraction

```rust
// Source: ffmpeg-sidecar docs (ffprobe_path, ffprobe_is_installed) [VERIFIED]
// Combined with ffprobe CLI JSON output format [VERIFIED: ffmpeg.org]

use ffmpeg_sidecar::ffprobe::ffprobe_path;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoMetadata {
    pub filename: String,
    pub filepath: String,
    pub duration_secs: f64,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u64,
    pub codec: String,
    pub fps: f32,
}

#[derive(Debug, Deserialize)]
struct RawProbeOutput {
    format: RawFormat,
    streams: Vec<RawStream>,
}

#[derive(Debug, Deserialize)]
struct RawFormat {
    filename: String,
    duration: String,
    size: String,
}

#[derive(Debug, Deserialize)]
struct RawStream {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    #[serde(rename = "r_frame_rate")]
    r_frame_rate: Option<String>, // e.g., "30/1"
}

pub fn extract_metadata(filepath: &str, ffmpeg_path_opt: Option<&str>) -> Result<VideoMetadata, String> {
    let ffprobe_bin = match ffmpeg_path_opt {
        Some(dir) => Path::new(dir).join(if cfg!(windows) { "ffprobe.exe" } else { "ffprobe" }),
        None => ffprobe_path(),
    };

    let output = Command::new(&ffprobe_bin)
        .args([
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            filepath,
        ])
        .output()
        .map_err(|e| format!("ffprobe execution failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("File is not a valid video: {}", stderr.trim()));
    }

    let probe: RawProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let video_stream = probe.streams.iter()
        .find(|s| s.codec_type == "video")
        .ok_or("No video stream found in file")?;

    let duration: f64 = probe.format.duration.parse().unwrap_or(0.0);
    let size: u64 = probe.format.size.parse().unwrap_or(0);
    let width = video_stream.width.unwrap_or(0);
    let height = video_stream.height.unwrap_or(0);
    let codec = video_stream.codec_name.clone().unwrap_or_default();
    let fps = video_stream.r_frame_rate.as_ref()
        .and_then(|r| {
            let parts: Vec<&str> = r.split('/').collect();
            if parts.len() == 2 {
                let num: f32 = parts[0].parse().ok()?;
                let den: f32 = parts[1].parse().ok()?;
                if den > 0.0 { Some(num / den) } else { None }
            } else {
                None
            }
        })
        .unwrap_or(0.0);

    let filepath_owned = filepath.to_string();
    let filename = Path::new(filepath)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(VideoMetadata {
        filename,
        filepath: filepath_owned,
        duration_secs: duration,
        width,
        height,
        size_bytes: size,
        codec,
        fps,
    })
}
```

### Output File Naming with Collision Detection

```rust
// Source: D-16 requirement [LOCKED]
use std::path::{Path, PathBuf};

/// Returns the output path with collision-safe naming.
/// D-16: {original_stem}_{seed_alias}.{ext}
/// If exists, appends -1, -2, etc. before extension.
pub fn make_output_path(
    source_path: &Path,
    seed_alias: &str,
    output_dir: &Path,
) -> PathBuf {
    let stem = source_path.file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| "output".into());
    let ext = source_path.extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_else(|| "mp4".to_string());

    let base_name = format!("{}_{}", stem, seed_alias);
    let mut candidate = output_dir.join(format!("{}.{}", base_name, ext));

    // D-16: Collision detection with numeric suffix
    let mut suffix = 1;
    while candidate.exists() {
        candidate = output_dir.join(format!("{}-{}.{}", base_name, suffix, ext));
        suffix += 1;
    }

    candidate
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 `tauri::api::process::Command` | Tauri v2 `tauri_plugin_shell::ShellExt` | Tauri 2.0 (2024) | Phase 1 already uses v2; no migration needed |
| `std::process::Command` for FFmpeg | `ffmpeg-sidecar::FfmpegCommand` | Phase 1 decision | Already adopted; Phase 2 extends with `.filter_complex()` for operation chains |
| `tauri-plugin-store` v1 API | v2 API with `app.store()` | Tauri 2.0 (2024) | Phase 1 already uses v2; `.get()`/`.set()` with `serde_json::Value` |
| `rand 0.8.x` | `rand 0.9.x` with `rand::rng()` | 2025 | Simplified API; `thread_rng()` replaced by `rand::rng()` |

**Deprecated/outdated:**
- `rand::thread_rng()`: Deprecated in 0.9.x; use `rand::rng()` instead.
- `std::sync::Mutex` for async commands: Still usable but `tokio::sync::Mutex` is recommended for state accessed across `.await` points. See Anti-Patterns section.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A — desktop app, local only |
| V3 Session Management | No | N/A |
| V4 Access Control | No | N/A — single-user desktop app |
| V5 Input Validation | Yes | `serde` deserialization with type validation; file path sanitization; ffprobe binary path must come from stored config, not user input |
| V6 Cryptography | No | N/A — no cryptographic operations |

### Known Threat Patterns for Tauri + FFmpeg

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via video filename | Tampering | Resolve paths with `std::fs::canonicalize()`; verify output path is within output directory; reject `..` segments |
| FFmpeg argument injection via seed parameters | Tampering | Use `FfmpegCommand::new_with_path()` with typed `.arg()` calls, not string interpolation; validate all numeric parameters are within bounds |
| Malformed ffprobe JSON DoS | Denial of Service | Set process timeout on ffprobe (e.g., 30s via `std::process::Child::wait_timeout`); limit JSON parse depth with `serde_json::from_slice` (default depth limit is safe) |
| Disk space exhaustion | Denial of Service | D-13: check available space before batch processing using `fs2::available_space()` |
| Malicious video file (codec exploits) | Elevation of Privilege | FFmpeg process runs with same privilege as app; no sandboxing beyond OS default. Acceptable for desktop app. |
| Store file corruption on crash | Data Loss | Write to temp file, rename atomically (or rely on `store.save()` which uses atomic write). On startup, gracefully handle deserialization failures with empty default state. |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| FFmpeg binary | Batch processing (BATCH-01) | Depends on Phase 1 | >= 4.0 | Error if not configured; frontend gates processing behind FFmpeg status check |
| FFprobe binary | Video import (IMPORT-01/02) | Depends on Phase 1 D-16 | N/A | Error; ffprobe is required for metadata. D-16 guarantees ffprobe is downloaded with ffmpeg |
| Rust toolchain | Compilation | Yes | 1.85+ | -- |
| tauri-plugin-store | Persistence (D-05) | Yes | 2.4.3 | Already in Cargo.toml and initialized in lib.rs |
| rand crate | Seed generation | Not yet added | 0.9.x | Must be added via `cargo add` |
| uuid crate | Seed IDs | Not yet added | 1.x | Must be added via `cargo add` |

**Missing dependencies with no fallback:**
- `rand` and `uuid` crates must be added to Cargo.toml before seed generation commands compile. These are required, not optional.

**Missing dependencies with fallback:**
- None -- all dependencies are resolvable by `cargo add`.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `rand 0.9.x` uses `rand::rng()` instead of `thread_rng()` | Standard Stack | Low — API change detected at compile time; easy fix |
| A2 | ffprobe JSON output format is stable across versions >= 4.0 | Architecture Patterns | Low — ffprobe JSON format has been stable since FFmpeg 3.x; minor fields may vary |
| A3 | `tauri-plugin-store`'s `.save()` is atomic (won't corrupt on crash) | Architecture Patterns | Medium — if non-atomic, crash during save could corrupt store. Mitigation: load with fallback to default on deserialize failure |
| A4 | `geq` filter expressions can reference input pixel values via `lum(X,Y)`, `cb(X,Y)`, `cr(X,Y)` | Code Examples | Medium — if `geq` cannot modify existing pixels (only generates new), the approach needs `geq` on nullsrc + `blend` instead. Testing required. |
| A5 | macOS ARM (Apple Silicon) weak memory ordering affects `AtomicBool` visibility if `Ordering::Relaxed` is used | Common Pitfalls | Low — existing code already uses `Ordering::SeqCst`; safe as long as pattern is followed |
| A6 | `FfmpegChild::kill()` on Windows uses `TerminateProcess` which is immediate but may leave temp files | Architecture Patterns | Low — we explicitly clean up incomplete output files after kill, so temp file leakage is handled |
| A7 | The 7 operation types can all be implemented as FFmpeg filter chains without external tools | State of the Art | Low — all 7 types (overlay, shift, frame drop, GOP, metadata, audio, remux) are standard FFmpeg capabilities; confirmed in FFmpeg documentation |

## Open Questions

1. **Math overlay filter chain construction**
   - What we know: `geq` can generate patterns, `blend` can composite at low opacity. `geq=lum='lum(X,Y)*(1+0.05*sin(...))'` may modify existing pixel values directly.
   - What's unclear: Whether `geq` on an existing video input modifies pixels in-place or generates a new stream. Official FFmpeg docs are ambiguous — `geq` with input uses `lum(X,Y)` to reference input luma but the output behavior needs empirical verification.
   - Recommendation: Test during implementation. Fallback: generate pattern with `geq` on nullsrc, then `blend` with `all_opacity=0.1`. More complex but guaranteed correct.

2. **GOP modification scope**
   - What we know: GOP can be modified via encoding parameters (`-g`, `-keyint_min`) or by re-encoding with different settings.
   - What's unclear: Whether GOP-only modification (without other visual changes) is meaningfully useful for fingerprint modification, since it requires re-encoding which inherently changes fingerprint regardless of GOP.
   - Recommendation: Implement as re-encode with randomized `-g` parameter (range: 12-250). It's in the list by requirement but may have minimal marginal effect beyond what other operations achieve.

3. **tauri-plugin-store concurrent saves**
   - What we know: The store API is synchronous for `.get()`/`.set()` but `.save()` is async.
   - What's unclear: Whether calling `.save()` on two different store files concurrently (e.g., saving seeds and queue simultaneously) causes issues. The plugin documentation doesn't document multi-store concurrency guarantees.
   - Recommendation: Serialize saves — save one store at a time. Since seed count and queue size are small (<1000 items), save latency is negligible.

## Sources

### Primary (HIGH confidence)
- [Context7: ffmpeg-sidecar](https://docs.rs/ffmpeg-sidecar/latest) — FfmpegCommand builder API, FfmpegChild kill/quit/wait, ffprobe_path, FfmpegProgress parsing, FfmpegMetadata, VideoStream struct. 1098 code snippets.
- [Context7: Tauri v2](https://v2.tauri.app/develop/state-management) — AppHandle::manage(), State<'_, T> extractor, Mutex wrapping, setup() hook, Manager trait.
- [Context7: tauri-plugin-store](https://github.com/ferreira-tb/tauri-store) — Store class, app.store() API, get/set/save with serde_json::Value, custom serialization.
- [Context7: Tauri v2 commands](https://v2.tauri.app/develop/calling-rust) — #[tauri::command] async fn, generate_handler![] macro, invoke_handler(), parameters (State, Window, AppHandle).
- [ffmpeg.org Filters Documentation](https://ffmpeg.org/ffmpeg-filters.html) — geq filter (mathematical expressions), blend/composite filters, framestep, metadata, setpts.
- [crates.io](https://crates.io) — rand 0.9.x, uuid 1.x latest versions verified.

### Secondary (MEDIUM confidence)
- [ffmpeg-sidecar docs.rs FfmpegChild](https://docs.rs/ffmpeg-sidecar/latest/ffmpeg_sidecar/child/struct.FfmpegChild.html) — kill(), quit(), wait(), send_stdin_command(), iter(), take_stdin/stdout/stderr. Verified against Context7 results.
- [ffmpeg-sidecar docs.rs FfmpegCommand](https://docs.rs/ffmpeg-sidecar/latest/ffmpeg_sidecar/command/struct.FfmpegCommand.html) — new(), new_with_path(), input(), output(), args(), filter_complex(), spawn(). Verified against Context7 results.
- Existing codebase: `src-tauri/src/commands/ffmpeg.rs`, `src-tauri/src/commands/download.rs`, `src-tauri/src/lib.rs` — store usage pattern, cancel pattern, command registration.

### Tertiary (LOW confidence)
- FFmpeg filter `geq` in-place pixel modification behavior — assumed from general documentation structure, needs empirical verification (see Assumption A4).

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all core dependencies already in Cargo.toml with verified versions; rand and uuid are well-established crates with verified versions
- Architecture: HIGH — Tauri v2 state management, store persistence, and ffmpeg-sidecar APIs are all verified via Context7 and existing codebase patterns
- Pitfalls: HIGH — stderr deadlock, mutex poisoning, and filter escaping are well-documented issues; AtomicBool visibility is architecture-specific but verified against existing codebase patterns

**Research date:** 2026-05-13
**Valid until:** 2026-06-12 (30 days — all core libraries are stable releases)
