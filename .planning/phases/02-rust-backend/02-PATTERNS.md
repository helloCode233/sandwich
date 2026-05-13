# Phase 02: Rust Backend - Pattern Map

**Mapped:** 2026-05-13
**Files analyzed:** 16 (13 new, 3 modified)
**Analogs found:** 16 / 16

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/commands/seed.rs` | controller | CRUD + event-driven | `src-tauri/src/commands/ffmpeg.rs` | role-match |
| `src-tauri/src/commands/import.rs` | controller | request-response + external-process | `src-tauri/src/commands/ffmpeg.rs` | role-match |
| `src-tauri/src/commands/queue.rs` | controller | CRUD | `src-tauri/src/commands/ffmpeg.rs` | role-match |
| `src-tauri/src/commands/batch.rs` | controller | streaming + event-driven | `src-tauri/src/commands/download.rs` | exact (cancel, emit, spawn_blocking) |
| `src-tauri/src/state.rs` | model | N/A (state container) | `download.rs` GlobalDownloadState (lines 37-44) | structural-match |
| `src-tauri/src/models/mod.rs` | utility | N/A (re-export) | `src-tauri/src/commands/mod.rs` | exact |
| `src-tauri/src/models/seed.rs` | model | N/A (data types) | `src-tauri/src/commands/ffmpeg.rs` FfmpegInfo/FfmpegConfig | exact (serde structs) |
| `src-tauri/src/models/video.rs` | model | N/A (data types) | `src-tauri/src/commands/ffmpeg.rs` FfmpegInfo/FfmpegConfig | exact (serde structs) |
| `src-tauri/src/models/batch.rs` | model | N/A (data types) | `src-tauri/src/commands/ffmpeg.rs` FfmpegInfo/FfmpegConfig | exact (serde structs) |
| `src-tauri/src/ffmpeg/mod.rs` | utility | N/A (re-export) | `src-tauri/src/commands/mod.rs` | exact |
| `src-tauri/src/ffmpeg/filters.rs` | utility | transform | `download.rs` free functions (e.g., `select_download_urls`) | structural-match |
| `src-tauri/src/ffmpeg/probe.rs` | utility | external-process | `ffmpeg.rs` `detect_ffmpeg_internal` + `verify_ffmpeg` | data-flow-match |
| `src-tauri/src/ffmpeg/executor.rs` | utility | external-process + streaming | `download.rs` `download_single` (progress, cancel, fs check) | exact |
| `src-tauri/src/lib.rs` (modified) | config | N/A (setup/registration) | itself | exact |
| `src-tauri/src/commands/mod.rs` (modified) | utility | N/A (re-export) | itself | exact |
| `src-tauri/Cargo.toml` (modified) | config | N/A (dependencies) | itself | exact |

## Pattern Assignments

### `src-tauri/src/commands/seed.rs` (controller, CRUD + event-driven)

**Analog:** `src-tauri/src/commands/ffmpeg.rs`

**Imports pattern** (lines 1-6 of ffmpeg.rs):
```rust
use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::version::{ffmpeg_version, ffmpeg_version_with_path};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;
```

**Struct/Type declaration pattern** (lines 10-18 of ffmpeg.rs):
```rust
/// Returned to the frontend after FFmpeg detection.
/// Maps to the `FfmpegInfo` TypeScript interface in src/types/ffmpeg.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegInfo {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub outdated: bool,
    pub needs_download: bool,
}
```

**Command signature pattern** (lines 100-101 of ffmpeg.rs):
```rust
#[tauri::command]
pub async fn detect_ffmpeg(app: AppHandle) -> Result<FfmpegInfo, String> {
```

**Store access + persistence pattern** (lines 103-128 of ffmpeg.rs):
```rust
    if let Ok(store) = app.store("ffmpeg-config.json")
        && let Some(cached_path) = store.get("ffmpeg_path")
        && let Some(path_str) = cached_path.as_str()
    {
        let cached_path = PathBuf::from(path_str);
        if cached_path.join("ffmpeg").exists() || cached_path.join("ffmpeg.exe").exists() {
            // ... read from store, fall back to fresh check on error
        }
    }
```

**Store write-through persistence** (lines 185-191 of ffmpeg.rs):
```rust
    let store =
        app.store("ffmpeg-config.json").map_err(|e| format!("Failed to open store: {}", e))?;
    store.set("ffmpeg_path", serde_json::Value::String(path.clone()));
    store.set("version", serde_json::Value::String(version_str.clone()));
    store.set("download_time", serde_json::Value::String(now));
    store.save().map_err(|e| format!("Failed to save store: {}", e))?;
```

**Error handling pattern** (throughout ffmpeg.rs):
```rust
    // Pattern: always return Result<T, String> from commands
    // Use .map_err(|e| format!("context: {}", e))? for error conversion
    // Never unwrap(); always propagate with context
```

**Event emission pattern** (line 201 of ffmpeg.rs):
```rust
    let _ = app.emit("ffmpeg-ready", info.clone());
```

**For this file specifically:**
- Use `rand::prelude::*` + `rand::distributions::WeightedIndex` for weighted random selection (see RESEARCH.md Pattern 5, lines 453-498)
- Use `uuid::Uuid::new_v4().to_string()` for seed IDs
- Use `chrono::Utc::now().format(...)` for timestamp aliases
- Persist to store using `app.store("seeds.json")` (see RESEARCH.md Pattern 2, lines 286-304)
- Serialize the entire `Vec<Seed>` as a single store value under key `"seeds"`
- Emit `"seeds-updated"` event after any mutation to keep frontend in sync

---

### `src-tauri/src/commands/import.rs` (controller, request-response + external-process)

**Analog:** `src-tauri/src/commands/ffmpeg.rs`

**Imports pattern** -- same as seed.rs above (copy from ffmpeg.rs lines 1-6).

**Command signature pattern** -- copy from ffmpeg.rs lines 100-101:
```rust
#[tauri::command]
pub async fn import_video(app: AppHandle, path: String) -> Result<VideoMetadata, String> {
```

**Store access pattern** (ffmpeg.rs lines 103-106):
```rust
    if let Ok(store) = app.store("ffmpeg-config.json")
        && let Some(cached_path) = store.get("ffmpeg_path")
        && let Some(path_str) = cached_path.as_str()
    {
```

**Error handling pattern** -- copy from ffmpeg.rs (always `Result<T, String>`, use `.map_err(|e| format!("context: {}", e))?`):
```rust
    // D-14: ffprobe validation on import — reject invalid files with specific error
    let output = Command::new(&ffprobe_bin)
        .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", &path])
        .output()
        .map_err(|e| format!("ffprobe execution failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("File is not a valid video: {}", stderr.trim()));
    }
```

**Event emission pattern** (ffmpeg.rs line 201):
```rust
    let _ = app.emit("video-imported", metadata.clone());
```

**For this file specifically:**
- Use `ffmpeg_sidecar::ffprobe::ffprobe_path()` to get ffprobe binary
- Use `std::process::Command` for ffprobe (not ffmpeg-sidecar's `FfmpegCommand` -- ffprobe is CLI-only)
- Parse JSON output with `serde_json::from_slice::<RawProbeOutput>(&output.stdout)`
- See RESEARCH.md Pattern 4 (lines 386-440) for full ffprobe extraction pattern
- Verify at least one video stream present (D-14)
- Use `#[serde(rename_all = "camelCase")]` on `VideoMetadata` to match frontend TypeScript interface

---

### `src-tauri/src/commands/queue.rs` (controller, CRUD)

**Analog:** `src-tauri/src/commands/ffmpeg.rs`

**Imports pattern** -- same as ffmpeg.rs lines 1-6.

**Command signature pattern** -- same as ffmpeg.rs line 100-101:
```rust
#[tauri::command]
pub async fn remove_from_queue(app: AppHandle, index: usize) -> Result<(), String> {
```

**Store persistence pattern** (ffmpeg.rs lines 185-191):
```rust
    let store = app.store("queue.json").map_err(|e| format!("Failed to open queue store: {}", e))?;
    let json = serde_json::to_value(&queue).map_err(|e| format!("Serialization error: {}", e))?;
    store.set("queue", json);
    store.save().map_err(|e| format!("Failed to save queue: {}", e))?;
```

**Error return pattern** (throughout ffmpeg.rs):
```rust
    // Always Result<T, String>, no panics in command functions
```

**For this file specifically:**
- CRUD operations: `get_queue`, `remove_from_queue`, `clear_queue`
- Each command accesses managed state via `state: tauri::State<'_, Mutex<AppState>>`
- On mutation, write-through to `app.store("queue.json")` with serialized `Vec<VideoEntry>` under key `"queue"`
- D-06: Path validity check -- on get, verify each entry's path exists; mark invalid entries
- Emit `"queue-updated"` event after any mutation

---

### `src-tauri/src/commands/batch.rs` (controller, streaming + event-driven)

**Analog:** `src-tauri/src/commands/download.rs` -- exact match for cancel pattern, event streaming, and progress emission.

**Imports pattern** (download.rs lines 1-8):
```rust
use ffmpeg_sidecar::download::unpack_ffmpeg;
use reqwest::header::{HeaderValue, RANGE};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
```

**Cancel flag declaration + accessor pattern** (download.rs lines 37-57):
```rust
/// Global download state for cancellation and status tracking.
/// Uses OnceLock for lazy initialization, Mutex for safe concurrent access.
struct GlobalDownloadState {
    cancel_flag: AtomicBool,
    temp_file_path: Option<PathBuf>,
    downloaded_bytes: u64,
    retry_count: u32,
    error_message: Option<String>,
}

static DOWNLOAD_STATE: OnceLock<Mutex<GlobalDownloadState>> = OnceLock::new();

fn get_download_state() -> &'static Mutex<GlobalDownloadState> {
    DOWNLOAD_STATE.get_or_init(|| {
        Mutex::new(GlobalDownloadState {
            cancel_flag: AtomicBool::new(false),
            temp_file_path: None,
            downloaded_bytes: 0,
            retry_count: 0,
            error_message: None,
        })
    })
}
```

**Cancel flag store/load pattern** (download.rs lines 83 and 356-363):
```rust
    // Store (set cancel -- from cancel_batch command):
    state.cancel_flag.store(true, Ordering::SeqCst);

    // Load (check cancel -- in processing loop):
    {
        let state = get_download_state().lock().await;
        if state.cancel_flag.load(Ordering::SeqCst) {
            return Err("Download cancelled".to_string());
        }
    }
```
CRITICAL: Always use `Ordering::SeqCst` for both `store` and `load`. This is the pattern verified in download.rs line 83 and documented in RESEARCH.md Pitfall 5 (macOS ARM weak memory ordering).

**Progress emission pattern** (download.rs lines 389-398):
```rust
            let _ = app.emit(
                "ffmpeg-download-progress",
                DownloadProgress {
                    percent,
                    downloaded_bytes: downloaded,
                    total_bytes,
                    speed_bytes_per_sec: speed,
                    stage: DownloadStage::Downloading,
                },
            );
```

**Spawn blocking pattern** (download.rs lines 338-401, but batch uses `tokio::task::spawn_blocking`):
```rust
    let result = tokio::task::spawn_blocking(move || {
        // ... FFmpeg execution in blocking context ...
    }).await.map_err(|e| format!("Join error: {}", e))?;
```

**Token-level cancellation check** (download.rs lines 357-364):
```rust
        // Check cancellation between files
        {
            let state = get_download_state().lock().await;
            if state.cancel_flag.load(Ordering::SeqCst) {
                return Err("Cancelled".to_string());
            }
        }
```

**Per-file failure isolation pattern** (D-11, RESEARCH.md lines 366-370):
```rust
        .map_err(|e| {
            // D-11: Single-file failure isolation -- log error, continue
            let _ = app.emit("batch-file-error", FileError { file: entry.name.clone(), error: e });
            // Continue to next file
        });
```

**For this file specifically:**
- Cancel flag stored in managed `BatchState` within `AppState`
- Between each file: check cancel flag; if set, kill current `FfmpegChild`, clean up incomplete output file, stop loop
- D-10: On cancel, call `child.kill()` on the active `FfmpegChild`, delete in-progress output file
- D-11: Wrap each file's FFmpeg execution in its own Result handling; on Err, emit `"batch-file-error"` and continue
- Use `FfmpegCommand::new_with_path(stored_ffmpeg_path)` -- never `FfmpegCommand::new()` (per Anti-Patterns)
- Emit `"batch-progress"` with percent/total/succeeded/failed per entry completed
- Emit `"batch-complete"` with summary when all files done (or cancelled)
- Use `tokio::sync::Semaphore` or a counter for concurrency limiting (D-08: 1-4, default 1)
- Progress events: throttle to at most every 100ms (copy pattern from download.rs line 374)

---

### `src-tauri/src/state.rs` (model, state container)

**Analog:** `src-tauri/src/commands/download.rs` GlobalDownloadState (lines 37-44)

**Core pattern** (download.rs lines 37-44):
```rust
struct GlobalDownloadState {
    cancel_flag: AtomicBool,
    temp_file_path: Option<PathBuf>,
    downloaded_bytes: u64,
    retry_count: u32,
    error_message: Option<String>,
}
```

**For this file specifically:**
```rust
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use crate::models::seed::Seed;
use crate::models::video::VideoEntry;
use crate::models::batch::BatchProgress;

pub struct AppState {
    pub seeds: Mutex<Vec<Seed>>,
    pub queue: Mutex<Vec<VideoEntry>>,
    pub batch_state: Mutex<BatchState>,
}

pub struct BatchState {
    pub cancel_flag: AtomicBool,
    pub status: BatchStatus,
    pub progress: BatchProgress,
}

pub enum BatchStatus {
    Idle,
    Running,
    Cancelling,
}
```

Note: `BatchProgress` struct defined in `src-tauri/src/models/batch.rs`.

---

### `src-tauri/src/models/mod.rs` (utility, re-export)

**Analog:** `src-tauri/src/commands/mod.rs` (lines 1-2)

**Exact pattern to copy** (commands/mod.rs lines 1-2):
```rust
pub mod seed;
pub mod video;
pub mod batch;
```

---

### `src-tauri/src/models/seed.rs` (model, data types)

**Analog:** `src-tauri/src/commands/ffmpeg.rs` FfmpegInfo and FfmpegConfig structs (lines 10-28)

**Core serde struct pattern** (ffmpeg.rs lines 10-18):
```rust
/// Returned to the frontend after FFmpeg detection.
/// Maps to the `FfmpegInfo` TypeScript interface in src/types/ffmpeg.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegInfo {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub outdated: bool,
    pub needs_download: bool,
}
```

**Enum pattern** (download.rs lines 24-33):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStage {
    Connecting,
    Downloading,
    Extracting,
    Verifying,
    Complete,
    Error,
}
```

**For this file specifically:**
- `Seed` struct: id (String/Uuid), alias (String), operations (Vec<Operation>), created_at (String -- ISO 8601)
- `Operation` struct: op_type (OperationType), start_frame (u32), duration_frames (u32), params (serde_json::Value)
- `OperationType` enum: MathOverlay, PixelShift, FrameDrop, GopModify, MetadataErase, AudioTweak, Remux
- ALL structs and enums must use `#[serde(rename_all = "camelCase")]` for TypeScript interop
- ALL enums must derive `Serialize, Deserialize, Clone, Debug`
- ALL structs must derive `Debug, Clone, Serialize, Deserialize`

---

### `src-tauri/src/models/video.rs` (model, data types)

**Analog:** `src-tauri/src/commands/ffmpeg.rs` FfmpegInfo struct (lines 10-18)

**Same serde pattern as seed.rs above.**

**For this file specifically:**
- `VideoEntry` struct: filename (String), filepath (String), metadata (VideoMetadata), status (VideoStatus)
- `VideoMetadata` struct: duration_secs (f64), width (u32), height (u32), size_bytes (u64), codec (String), fps (f32)
- `VideoStatus` enum: Valid, Invalid (preserves metadata per D-06)
- All use `#[serde(rename_all = "camelCase")]`
- `VideoMetadata` mirrors the frontend TS interface; see RESEARCH.md lines 636-647 for the exact fields

---

### `src-tauri/src/models/batch.rs` (model, data types)

**Analog:** `src-tauri/src/commands/ffmpeg.rs` FfmpegInfo/FfmpegUpdateInfo structs (lines 10-18, 32-38)

**Same serde pattern as seed.rs and video.rs above.**

**For this file specifically:**
- `BatchConfig` struct: seed_id (String), output_dir (String), concurrency (u32 -- 1-4)
- `BatchProgress` struct: total (usize), completed (usize), succeeded (usize), failed (usize), current_file (Option<String>)
- `BatchResult` struct: succeeded (Vec<String>), failed (Vec<FileResult>)
- `FileResult` struct: file (String), seed (String), error (String)
- All use `#[serde(rename_all = "camelCase")]`

---

### `src-tauri/src/ffmpeg/mod.rs` (utility, re-export)

**Analog:** `src-tauri/src/commands/mod.rs` (lines 1-2)

**Exact pattern to copy** (commands/mod.rs lines 1-2):
```rust
pub mod filters;
pub mod probe;
pub mod executor;
```

---

### `src-tauri/src/ffmpeg/filters.rs` (utility, transform)

**Analog:** `src-tauri/src/commands/download.rs` free functions `select_download_urls` (lines 478-539) and `get_manual_download_url` (lines 543-556)

**Free function pattern** (download.rs lines 478-479):
```rust
fn select_download_urls() -> Vec<Vec<String>> {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return vec![...];
    }
```

**Error handling pattern for utility functions** (download.rs lines 425-431):
```rust
    unpack_ffmpeg(&archive_path, target_dir).map_err(|e| {
        format!(
            "Extraction failed: {}. The archive may be corrupted. Try deleting {} and retrying.",
            e,
            archive_path.display()
        )
    })?;
```

**For this file specifically:**
- Pure functions with no AppHandle dependency -- takes `&Operation`, returns `Vec<String>` (FFmpeg arg list)
- 7 functions, one per `OperationType`, each constructing FFmpeg filter arguments
- Each function returns `Result<Vec<String>, String>` for safety
- Math overlay: build `geq` or `blend` filter chain with safety-constrained params (opacity <= 0.15 per SEED-04)
- Pixel shift: `crop` + `pad` with dx/dy ranging -3..=3 (SEED-04)
- Frame drop: `framestep` with interval >= 15 (SEED-04)
- GOP modify: `-g` encoding parameter, range 12-250
- Metadata erase: `-map_metadata -1`
- Audio tweak: `volume`, `atempo`, or `aecho` with modest parameters
- Remux: change container format via output extension
- Escaping: colons must be escaped as `\:` in filter arguments (RESEARCH.md Pitfall 4)

---

### `src-tauri/src/ffmpeg/probe.rs` (utility, external-process)

**Analog:** `src-tauri/src/commands/ffmpeg.rs` `detect_ffmpeg_internal` (lines 42-84) -- same role of running external binary and parsing output.

**Core external process pattern** (ffmpeg.rs lines 42-84):
```rust
pub async fn detect_ffmpeg_internal() -> FfmpegInfo {
    if ffmpeg_is_installed() {
        match ffmpeg_version() {
            Ok(version_str) => {
                let major_version = extract_major_version(&version_str);
                if major_version >= 4 {
                    FfmpegInfo {
                        found: true,
                        path: std::env::var("PATH").ok(),
                        version: Some(version_str),
                        outdated: false,
                        needs_download: false,
                    }
                } else {
                    FfmpegInfo { ... }
                }
            }
            Err(_) => FfmpegInfo { ... },
        }
    } else {
        FfmpegInfo { ... }
    }
}
```

**For this file specifically:**
- Function signature: `pub fn probe_video(filepath: &str, ffmpeg_dir: &str) -> Result<VideoMetadata, String>`
- Takes the stored ffmpeg directory (from Phase 1 config) to locate ffprobe binary
- Runs `std::process::Command` with `ffprobe -v quiet -print_format json -show_format -show_streams`
- Parses JSON output with `serde_json::from_slice` into typed structs
- Validates at least one video stream present (D-14)
- Extracts: duration, width, height, size_bytes, codec, fps (from `r_frame_rate` "30/1" format)
- See RESEARCH.md lines 631-735 for the full implementation pattern with FPS parsing

---

### `src-tauri/src/ffmpeg/executor.rs` (utility, external-process + streaming)

**Analog:** `src-tauri/src/commands/download.rs` `download_single` function (lines 252-442) -- exact match for progress streaming, cancel checking, file I/O patterns.

**Progress streaming pattern** (download.rs lines 389-398):
```rust
            let _ = app.emit(
                "ffmpeg-download-progress",
                DownloadProgress {
                    percent,
                    downloaded_bytes: downloaded,
                    total_bytes,
                    speed_bytes_per_sec: speed,
                    stage: DownloadStage::Downloading,
                },
            );
```

**Cancel check in iteration loop** (download.rs lines 357-364):
```rust
        // Check cancellation
        {
            let state = get_download_state().lock().await;
            if state.cancel_flag.load(Ordering::SeqCst) {
                return Err("Download cancelled".to_string());
            }
        }
```

**Spawn blocking pattern** (conceptual from download.rs async pattern + RESEARCH.md lines 344-365):
```rust
    tokio::task::spawn_blocking(move || {
        let mut child = FfmpegCommand::new_with_path(&ffmpeg_path)
            .input(&entry.path)
            .args(&filter_args)
            .output(&output_path)
            .spawn()
            .map_err(|e| format!("FFmpeg spawn failed: {}", e))?;

        for event in child.iter().map_err(|e| e.to_string())? {
            if cancel_flag.load(Ordering::SeqCst) {
                child.kill().ok();
                return Err("Cancelled".into());
            }
            // Parse progress from event and emit
        }
        child.wait().map_err(|e| format!("FFmpeg wait error: {}", e))?;
        Ok(())
    }).await.map_err(|e| format!("Join error: {}", e))?;
```

**File cleanup on cancel** (download.rs lines 452-457):
```rust
    if let Some(ref temp_path) = state.temp_file_path {
        let _ = std::fs::remove_file(temp_path);
    }
    let temp_dir = std::env::temp_dir().join("sandwich-ffmpeg-download");
    let _ = std::fs::remove_dir_all(&temp_dir);
```

**For this file specifically:**
- Function signature: `pub fn execute_batch_entry(...) -> Result<(), String>` -- executes one FFmpeg command for one queue entry + seed
- Uses `FfmpegCommand::new_with_path(ffmpeg_path)` -- never `new()` (per Anti-Patterns)
- Accepts cancel `AtomicBool` reference for inter-iteration cancellation checks
- Accepts `AppHandle` for progress emission
- Uses `child.iter()` to stream structured events and `child.kill()` on cancel (D-10)
- Progress from `FfmpegProgress` event parsed and emitted as `"batch-progress"` event
- Returns `Result` -- caller in `batch.rs` handles D-11 failure isolation
- Uses `tokio::task::spawn_blocking` for CPU-bound FFmpeg work (caller in batch.rs dispatches)

---

### `src-tauri/src/lib.rs` (modified, config/setup)

**Analog:** itself -- existing `lib.rs` (lines 1-45)

**Module declaration pattern** (line 1):
```rust
mod commands;
```

New modules to declare at top:
```rust
mod commands;
mod state;
mod models;
mod ffmpeg;
```

**Import and invoke_handler pattern** (lines 3-6, 16-22):
```rust
use commands::download::{cancel_download, start_download};
use commands::ffmpeg::{...};

.invoke_handler(tauri::generate_handler![
    detect_ffmpeg,
    get_ffmpeg_status,
    start_download,
    cancel_download,
    verify_ffmpeg,
])
```

**For this file specifically:**
- Add `mod state;`, `mod models;`, `mod ffmpeg;` after `mod commands;` (line 1)
- Add imports for all new commands: `commands::seed::*`, `commands::import::*`, `commands::queue::*`, `commands::batch::*`
- Register all new commands in `generate_handler![]` -- add `generate_seed`, `rename_seed`, `delete_seed`, `copy_seed`, `list_seeds`, `import_video`, `remove_from_queue`, `clear_queue`, `get_queue`, `start_batch`, `cancel_batch`, `get_batch_status`
- In `setup()`, initialize managed state: `app.manage(Mutex::new(AppState::default()))`
- In `setup()`, load persisted seeds and queue from store, populate into managed state
- In `setup()`, emit initial states to frontend via events

**Managed state init in setup** (NEW pattern, modeled after Tauri v2 docs + existing setup pattern lines 23-40):
```rust
    .setup(|app| {
        // Initialize managed state
        use std::sync::Mutex;
        app.manage(Mutex::new(state::AppState::default()));

        // Load persisted state from store into managed state
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            // Load seeds
            if let Ok(store) = handle.store("seeds.json") {
                if let Some(value) = store.get("seeds") {
                    if let Ok(seeds) = serde_json::from_value::<Vec<models::seed::Seed>>(value) {
                        if let Ok(state) = handle.state::<Mutex<state::AppState>>().lock() {
                            *state.seeds.lock().unwrap() = seeds;
                        }
                    }
                }
            }
            // Load queue similarly from "queue.json"
            // Emit initial states
            let _ = handle.emit("seeds-loaded", /* ... */);
            let _ = handle.emit("queue-loaded", /* ... */);
        });

        // ... existing FFmpeg detection code (lines 26-39 unchanged) ...

        Ok(())
    })
```

---

### `src-tauri/src/commands/mod.rs` (modified, re-export)

**Analog:** itself -- existing `commands/mod.rs` (lines 1-2)

**Exact pattern** (lines 1-2):
```rust
pub mod download;
pub mod ffmpeg;
```

**For this file specifically:**
Add four new module declarations:
```rust
pub mod download;
pub mod ffmpeg;
pub mod seed;
pub mod import;
pub mod queue;
pub mod batch;
```

---

### `src-tauri/Cargo.toml` (modified, dependencies)

**Analog:** itself -- existing `Cargo.toml` (lines 1-32)

**Add these new dependencies** (matching the exact formatting style of existing entries):
```toml
[dependencies]
# ... existing dependencies unchanged ...
rand = "0.9"
uuid = { version = "1", features = ["v4"] }
```

The existing Cargo.toml line 22 (reqwest) and line 27 (chrono) already have the `{ version = "...", features = [...] }` syntax to follow.

---

## Shared Patterns

### Authentication / Authorization
**Not applicable** -- desktop app, local-only, single-user. No auth middleware needed.

### Error Handling
**Source:** `src-tauri/src/commands/ffmpeg.rs` (throughout) + `src-tauri/src/commands/download.rs` (throughout)

**Apply to:** ALL new command modules and utility functions

```rust
// Pattern 1: Command return type (ffmpeg.rs line 101, download.rs line 79):
#[tauri::command]
pub async fn command_name(app: AppHandle, ...) -> Result<T, String> {
    // Never panic; always return Err(String)

// Pattern 2: Error conversion with context (ffmpeg.rs line 187):
    operation.map_err(|e| format!("Failed to do X: {}", e))?;

// Pattern 3: Store access error handling (ffmpeg.rs line 187):
    let store = app.store("config.json")
        .map_err(|e| format!("Failed to open store: {}", e))?;

// Pattern 4: Mutex lock recovery from poisoning (RESEARCH.md Pitfall 3):
    let state = app.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
```

CRITICAL: NEVER use `unwrap()` in command functions. Always propagate with `?` or `map_err(|e| format!(...))`. The `Result<T, String>` pattern is universal across all existing Phase 1 commands.

### Event Emission
**Source:** `src-tauri/src/commands/ffmpeg.rs` (line 201) + `src-tauri/src/commands/download.rs` (lines 389-398)

**Apply to:** `commands/batch.rs`, `commands/seed.rs`, `commands/import.rs`, `commands/queue.rs`

```rust
// Pattern: fire-and-forget event emission (ffmpeg.rs line 201):
let _ = app.emit("event-name", payload);

// Pattern: structured progress event (download.rs lines 389-398):
let _ = app.emit(
    "event-name",
    ProgressPayload {
        percent,
        stage: Stage::InProgress,
        ...other_fields,
    },
);
```

Event naming conventions from existing codebase:
- `"ffmpeg-status"` -- initial state on startup
- `"ffmpeg-ready"` -- ready notification
- `"ffmpeg-download-progress"` -- streaming progress

New event names for Phase 2 (following same kebab-case convention):
- `"seeds-updated"` -- after any seed mutation
- `"video-imported"` -- after successful import
- `"queue-updated"` -- after any queue mutation
- `"batch-progress"` -- per-file progress during batch
- `"batch-file-error"` -- single-file failure (D-11 isolation)
- `"batch-complete"` -- batch finished or cancelled

### Persistence (tauri-plugin-store)
**Source:** `src-tauri/src/commands/ffmpeg.rs` (lines 185-191)

**Apply to:** `commands/seed.rs`, `commands/queue.rs`, `commands/batch.rs`

```rust
// Pattern: write-through persistence (exact from ffmpeg.rs lines 186-191):
let store = app.store("seeds.json")
    .map_err(|e| format!("Failed to open store: {}", e))?;
store.set("seeds", serde_json::to_value(&seeds)
    .map_err(|e| format!("Serialization error: {}", e))?);
store.save()
    .map_err(|e| format!("Failed to save: {}", e))?;
```

CRITICAL per RESEARCH.md Anti-Patterns:
- Serialize full `Vec<T>` as a single JSON value under one key -- NOT individual keys per item
- Load from store ONCE on startup (in `setup()`), write-through on mutations
- Use `serde_json::to_value()` / `serde_json::from_value()` for store serialization

### Serde camelCase Interop
**Source:** `src-tauri/src/commands/ffmpeg.rs` (lines 11, 14, 24, 33)

**Apply to:** ALL model structs in `models/`, ALL event payload types, ALL command return types

```rust
// Pattern (ffmpeg.rs line 11):
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructName {
    pub rust_field: Type,  // Maps to `rustField` in TypeScript
}
```

### Cancel Pattern
**Source:** `src-tauri/src/commands/download.rs` (lines 37-57, 83, 357-364)

**Apply to:** `commands/batch.rs`, `ffmpeg/executor.rs`

```rust
// Pattern: static cancel flag with OnceLock + Mutex (download.rs lines 46-57):
static DOWNLOAD_STATE: OnceLock<Mutex<GlobalDownloadState>> = OnceLock::new();

fn get_download_state() -> &'static Mutex<GlobalDownloadState> {
    DOWNLOAD_STATE.get_or_init(|| {
        Mutex::new(GlobalDownloadState {
            cancel_flag: AtomicBool::new(false),
            // ...other fields...
        })
    })
}

// Pattern: store cancel (download.rs line 83):
state.cancel_flag.store(true, Ordering::SeqCst);

// Pattern: check cancel (download.rs lines 357-360):
let state = get_download_state().lock().await;
if state.cancel_flag.load(Ordering::SeqCst) {
    return Err("Cancelled".to_string());
}
```

For Phase 2, the cancel flag is stored inside managed `BatchState` (accessed via `app.state::<Mutex<AppState>>()`), NOT as a global static. The access mechanism differs but the `AtomicBool` + `Ordering::SeqCst` pattern is identical.

### FFmpeg Binary Path
**Source:** `src-tauri/src/commands/ffmpeg.rs` (lines 103-106)

**Apply to:** `ffmpeg/executor.rs`, `ffmpeg/probe.rs`, `commands/batch.rs`

```rust
// Pattern: read stored ffmpeg path (ffmpeg.rs lines 103-106):
if let Ok(store) = app.store("ffmpeg-config.json")
    && let Some(cached_path) = store.get("ffmpeg_path")
    && let Some(path_str) = cached_path.as_str()
{
    // use path_str with FfmpegCommand::new_with_path(path_str)
    // or Path::new(path_str).join("ffprobe") for ffprobe
}
```

Per RESEARCH.md Anti-Patterns: Always use `FfmpegCommand::new_with_path(stored_path)`, never `FfmpegCommand::new()` which only checks PATH.

### Module Organization
**Source:** `src-tauri/src/commands/mod.rs` (lines 1-2) + `src-tauri/src/lib.rs` (line 1)

**Apply to:** all new modules

```rust
// lib.rs: declare top-level modules in canonical order (system → domain → commands):
mod commands;  // existing
mod state;     // NEW
mod models;    // NEW
mod ffmpeg;    // NEW

// commands/mod.rs: declare command sub-modules in dependency order:
pub mod download;   // existing
pub mod ffmpeg;     // existing
pub mod seed;       // NEW -- depends on models::seed
pub mod import;     // NEW -- depends on ffmpeg::probe, models::video
pub mod queue;      // NEW -- depends on models::video, state
pub mod batch;      // NEW -- depends on ffmpeg::executor, ffmpeg::filters, state

// models/mod.rs:
pub mod seed;
pub mod video;
pub mod batch;

// ffmpeg/mod.rs:
pub mod filters;
pub mod probe;
pub mod executor;
```

## No Analog Found

None -- all 16 files have direct analogs in the existing codebase:

| File | Analog | Basis |
|------|--------|-------|
| `commands/seed.rs` | `commands/ffmpeg.rs` | Same role (Tauri command module), adapted data flow |
| `commands/import.rs` | `commands/ffmpeg.rs` | Same role, adapted data flow |
| `commands/queue.rs` | `commands/ffmpeg.rs` | Same role, adapted data flow |
| `commands/batch.rs` | `commands/download.rs` | Exact match for cancel + emit + spawn patterns |
| `state.rs` | `download.rs` lines 37-44 | Same structural pattern for state encapsulation |
| All 3 `models/*.rs` | `ffmpeg.rs` lines 10-28 | Exact serde struct pattern |
| All 3 `ffmpeg/*.rs` | `download.rs` free functions | Same utility function patterns |
| `lib.rs` (modified) | itself | Same file, extended |
| `commands/mod.rs` (modified) | itself | Same pattern, extended |
| `Cargo.toml` (modified) | itself | Same format, extended |

## Metadata

**Analog search scope:** `src-tauri/src/` (all .rs files)
**Files scanned:** 5 existing Rust source files + 1 Cargo.toml + 2 frontend type files
**Pattern extraction date:** 2026-05-13
**Codebase snapshot:** Phase 1 complete -- FFmpeg detection, download, verification commands registered. 4 Tauri plugins initialized (store, shell, dialog, fs).
