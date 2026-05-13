# Architecture Research

**Domain:** Tauri 2.x desktop video batch processing with FFmpeg
**Researched:** 2026-05-12
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          Frontend Layer (Vue 3 + Pinia)                      │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────────┐   ┌──────────────────────┐   ┌──────────────────┐ │
│  │    Seed Panel         │   │   Video Queue Panel   │   │  Preview Panel   │ │
│  │  (seedStore)          │   │  (videoStore)         │   │  (previewStore)  │ │
│  │  - List seeds         │   │  - Drop zone          │   │  - Video player  │ │
│  │  - Generate new       │   │  - Queue list         │   │  - Metadata view │ │
│  │  - Delete / Copy      │   │  - Reorder / Remove   │   │                  │ │
│  └──────────┬───────────┘   └──────────┬───────────┘   └────────┬─────────┘ │
│             │                          │                        │           │
│  ┌──────────┴──────────────────────────┴────────────────────────┴─────────┐ │
│  │                     Processing Controller Panel                          │ │
│  │  - Select seed → apply to queue                                         │ │
│  │  - Progress bar + per-video status                                      │ │
│  │  - Output directory picker                                              │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                          IPC Boundary (Tauri Commands + Events)               │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Commands (invoke):              Events (listen):                            │
│  generate_seed()                 processing-progress { video, pct, fps }     │
│  list_seeds()                    processing-complete { output_path }         │
│  delete_seed(id)                 processing-error { video, error }           │
│  import_videos(paths[])          ffmpeg-download-progress { pct }            │
│  get_video_metadata(path)        ffmpeg-status { found, version }            │
│  start_processing(config)                                                 │
│  cancel_processing()                                                       │
│  check_ffmpeg()                                                             │
│  download_ffmpeg()                                                          │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                        Rust Backend (src-tauri/src/)                         │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐   │
│  │                      Tauri Commands (commands.rs)                     │   │
│  │  #[tauri::command] fn generate_seed(...)                              │   │
│  │  #[tauri::command] fn import_videos(...)                              │   │
│  │  #[tauri::command] fn start_processing(...)  ← spawns async tasks     │   │
│  └──────────────┬───────────────────────────────────────────────────────┘   │
│                 │                                                            │
│  ┌──────────────┴───────────────────────────────────────────────────────┐   │
│  │                         Domain Services                               │   │
│  │                                                                       │   │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────────┐  │   │
│  │  │  Seed Engine     │  │  Video Manager   │  │  FFmpeg Engine       │  │   │
│  │  │  - generate()    │  │  - import()      │  │  - check_ffmpeg()    │  │   │
│  │  │  - validate()    │  │  - probe()       │  │  - download_ffmpeg() │  │   │
│  │  │  - serialize()   │  │  - metadata()    │  │  - build_command()   │  │   │
│  │  └────────┬────────┘  │  └────────┬───────┘  │  - spawn_process()   │  │   │
│  │           │            │           │          │  - parse_progress()  │  │   │
│  └───────────┼────────────┼───────────┼──────────┼─────────────────────┘   │
│              │            │           │          │                           │
│  ┌───────────┴────────────┴───────────┴──────────┴─────────────────────┐   │
│  │                       Managed State (lib.rs)                         │   │
│  │  app.manage(SeedStore(Mutex::new(HashMap)))                          │   │
│  │  app.manage(VideoQueue(Mutex::new(Vec)))                             │   │
│  │  app.manage(ProcessingState(Mutex::new(ProcessingStatus)))           │   │
│  │  app.manage(FfmpegPath(Mutex::new(Option<String>)))                  │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                          External Processes                                  │
│  ┌────────────────────┐  ┌──────────────────────────┐                       │
│  │  FFmpeg (process)  │  │  FFprobe (process)        │                       │
│  │  - Encode videos   │  │  - Extract metadata       │                       │
│  │  - Stream progress │  │  - Resolution, duration    │                       │
│  │  to stderr         │  │  - Codec info             │                       │
│  └────────────────────┘  └──────────────────────────┘                       │
├──────────────────────────────────────────────────────────────────────────────┤
│                          File System                                         │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │  $APPDATA/seeds/          JSON files, one per seed                  │     │
│  │  {user-selected}/output/  Processed video files                     │     │
│  │  $TEMP/sandwich/          Temporary intermediate files             │     │
│  └────────────────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component                 | Responsibility                                                                                              | Typical Implementation                                                                                                       |
| ------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **Seed Engine** (Rust)    | Random seed generation, validation against constraints, JSON serialization/deserialization                  | Pure Rust structs + `rand` crate + `serde_json`. Stateless — input is random seed value, output is `Seed` struct.            |
| **Video Manager** (Rust)  | Import video files, validate formats, extract metadata via FFprobe, queue management                        | Spawns `ffprobe` via `tauri_plugin_shell`, parses JSON output, returns `VideoMeta` structs.                                  |
| **FFmpeg Engine** (Rust)  | FFmpeg detection/download, command building from seed ops, process spawning, progress parsing, cancellation | Uses `tauri_plugin_shell::ShellExt` to spawn FFmpeg, reads `CommandEvent::Stderr` for progress, emits events to frontend.    |
| **Managed State** (Rust)  | In-memory store for seeds, video queue, processing status, FFmpeg path                                      | `tauri::State` with `std::sync::Mutex` (sync commands) or `tokio::sync::Mutex` (async commands). Managed via `app.manage()`. |
| **Tauri Commands** (Rust) | IPC entry points that delegate to domain services                                                           | `#[tauri::command]` functions. Each receives `State<>` for managed data, `AppHandle` for event emission.                     |
| **Pinia Stores** (Vue)    | UI state: seed list display, video queue display, processing progress, selection state, loading/error flags | Setup stores (Composition API) with `ref()`/`computed()`. Stores mirror Rust state fetched via `invoke()` commands.          |
| **Vue Components** (Vue)  | Layout, drag-and-drop zones, seed list, video preview, progress bars, settings                              | `<script setup>` components using composables and Pinia stores. No direct FFmpeg interaction — all through Tauri commands.   |

## Recommended Project Structure

```
sandwich/
├── src/                          # Vue 3 frontend
│   ├── components/               # Reusable UI components
│   │   ├── layout/               # AppShell, Sidebar, MainContent
│   │   │   ├── AppShell.vue      # Top-level layout (left/right split)
│   │   │   └── TitleBar.vue      # Custom title bar
│   │   ├── seeds/                # Seed-related components
│   │   │   ├── SeedList.vue      # Scrollable seed list
│   │   │   ├── SeedCard.vue      # Single seed display (name, op count)
│   │   │   └── SeedGenerateBtn.vue  # Generate new seed button
│   │   ├── videos/               # Video-related components
│   │   │   ├── VideoDropZone.vue # Drag-and-drop import area
│   │   │   ├── VideoQueue.vue    # Sortable video list
│   │   │   ├── VideoItem.vue     # Single queue item (thumbnail, name)
│   │   │   └── VideoPreview.vue  # Embedded video player
│   │   ├── processing/           # Processing-related components
│   │   │   ├── ProcessPanel.vue  # Seed selector + start button
│   │   │   ├── ProgressBar.vue   # Overall progress
│   │   │   └── ProcessLog.vue    # Per-video status log
│   │   └── common/               # Shared components
│   │       ├── ConfirmDialog.vue
│   │       └── EmptyState.vue
│   ├── stores/                   # Pinia stores
│   │   ├── seedStore.ts          # Seed CRUD state
│   │   ├── videoStore.ts         # Video queue state
│   │   ├── processingStore.ts    # Processing status + progress
│   │   └── settingsStore.ts      # Output dir, FFmpeg path, preferences
│   ├── composables/              # Reusable Vue composables
│   │   ├── useTauriEvent.ts      # listen() wrapper with cleanup
│   │   ├── useVideoDrop.ts       # Drag-and-drop logic
│   │   └── useFileDialog.ts      # Native file dialog wrapper
│   ├── types/                    # TypeScript type definitions
│   │   ├── seed.ts               # Seed, Operation, OpType, OpParams
│   │   ├── video.ts              # VideoMeta, VideoQueueItem, VideoStatus
│   │   └── processing.ts         # ProcessingConfig, ProgressEvent, ProcessResult
│   ├── tauri-api/                # Typed wrappers around invoke()
│   │   ├── seeds.ts              # generateSeed(), listSeeds(), deleteSeed()
│   │   ├── videos.ts             # importVideos(), getVideoMeta(), removeVideo()
│   │   ├── processing.ts         # startProcessing(), cancelProcessing()
│   │   └── ffmpeg.ts             # checkFfmpeg(), downloadFfmpeg()
│   ├── App.vue
│   └── main.ts
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Entry point (or lib.rs)
│   │   ├── lib.rs                # App builder, state management, plugin registration
│   │   ├── commands/             # Tauri command handlers (IPC layer)
│   │   │   ├── mod.rs
│   │   │   ├── seed_commands.rs  # Seed CRUD commands
│   │   │   ├── video_commands.rs # Video import/meta commands
│   │   │   ├── processing_commands.rs  # Start/cancel processing
│   │   │   └── ffmpeg_commands.rs      # FFmpeg check/download
│   │   ├── services/             # Domain logic (no Tauri dependency)
│   │   │   ├── mod.rs
│   │   │   ├── seed_engine.rs    # Seed generation + validation
│   │   │   ├── video_manager.rs  # Video import + FFprobe metadata
│   │   │   ├── ffmpeg_engine.rs  # FFmpeg detection, command building, execution, progress
│   │   │   └── storage.rs        # File I/O for seeds, config
│   │   ├── models/               # Shared data structures
│   │   │   ├── mod.rs
│   │   │   ├── seed.rs           # Seed, Operation, OpType, constraints
│   │   │   ├── video.rs          # VideoMeta, VideoQueueItem
│   │   │   └── processing.rs     # ProcessingConfig, ProgressPayload
│   │   └── errors.rs             # Unified error types (thiserror)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
├── vite.config.ts
└── tsconfig.json
```

### Structure Rationale

- **`src/stores/`:** Pinia stores are the single source of truth for UI state. Each store mirrors a domain (seeds, videos, processing). Stores call `invoke()` through the typed `src/tauri-api/` wrappers, never directly — this keeps IPC calls discoverable and typed.
- **`src/composables/`:** Reusable Vue 3 composition functions. `useTauriEvent` encapsulates `listen()` with automatic cleanup on component unmount, preventing memory leaks from event listeners.
- **`src/tauri-api/`:** Thin typed wrappers around `invoke()`. Each function maps to exactly one Tauri command. This provides autocomplete, type safety, and a single place to update if command signatures change.
- **`src-tauri/src/commands/`:** Each file corresponds to a domain, contains only `#[tauri::command]` functions. These are thin — they extract `State`, delegate to services, and format responses. No business logic in commands.
- **`src-tauri/src/services/`:** Pure domain logic with no Tauri dependency. Each service is stateless or receives state as parameters. This makes services unit-testable without Tauri runtime.
- **`src-tauri/src/models/`:** Shared structs with `Serialize`/`Deserialize`. These are the contract between Rust and TypeScript (via `ts-rs` or manual type mirroring in `src/types/`).

## Architectural Patterns

### Pattern 1: Command-Event Split for Long-Running Operations

**What:** Use Tauri commands (`invoke`/`#[tauri::command]`) for request-response operations (CRUD, queries). Use Tauri events (`emit`/`listen`) for streaming progress from long-running async tasks.

**When to use:** Any operation that takes more than ~500ms or produces incremental results. Processing videos takes minutes — commands would time out, events stream freely.

**Trade-offs:** Events are fire-and-forget with no built-in acknowledgment. If the frontend misses an event, it won't be resent. For this app, progress events are informational (not critical), so this is acceptable.

**Example:**

```rust
// Rust: command starts processing, spawns async task that emits events
#[tauri::command]
async fn start_processing(
    app: AppHandle,
    config_state: State<'_, Mutex<ProcessingConfig>>,
    queue_state: State<'_, Mutex<VideoQueue>>,
) -> Result<(), String> {
    let config = config_state.lock().unwrap().clone();
    let queue = queue_state.lock().unwrap().clone();

    tauri::async_runtime::spawn(async move {
        for video in queue.videos {
            app.emit("processing-progress", ProgressPayload {
                video: video.name.clone(),
                stage: "starting".into(),
                percent: 0,
                fps: 0.0,
            }).ok();

            let result = ffmpeg_engine::process_video(&config, &video, |pct, fps| {
                app.emit("processing-progress", ProgressPayload {
                    video: video.name.clone(),
                    stage: "encoding".into(),
                    percent: pct,
                    fps,
                }).ok();
            }).await;

            match result {
                Ok(path) => app.emit("processing-complete", CompletePayload {
                    video: video.name.clone(),
                    output_path: path,
                }).ok(),
                Err(e) => app.emit("processing-error", ErrorPayload {
                    video: video.name.clone(),
                    error: e.to_string(),
                }).ok(),
            }
        }
    });

    Ok(())
}
```

```typescript
// Frontend: listen to events, update Pinia store
import { listen } from '@tauri-apps/api/event';
import { useProcessingStore } from '@/stores/processingStore';

export function useTauriEvent() {
  const processingStore = useProcessingStore();

  onMounted(async () => {
    const unlisten = await listen<ProgressPayload>('processing-progress', (event) => {
      processingStore.updateProgress(event.payload);
    });
    onUnmounted(() => unlisten());
  });
}
```

### Pattern 2: Thin Commands, Fat Services

**What:** Tauri command handlers extract state and delegate to services. Services contain all logic and are Tauri-agnostic. This makes services testable without `tauri::test` and swappable if the IPC layer changes.

**When to use:** Always. Commands are the API contract — they should not contain business logic.

**Trade-offs:** Slightly more indirection (one extra function call). Negligible for this app's scale.

**Example:**

```rust
// commands/seed_commands.rs — thin, only IPC concerns
#[tauri::command]
fn generate_seed(
    state: State<'_, Mutex<SeedStore>>,
    alias: Option<String>,
) -> Result<Seed, String> {
    let seed = seed_engine::generate_random(alias);
    let mut store = state.lock().map_err(|e| e.to_string())?;
    store.insert(seed.id.clone(), seed.clone());
    storage::save_seeds(&store)?;
    Ok(seed)
}

// services/seed_engine.rs — pure logic, no Tauri imports
pub fn generate_random(alias: Option<String>) -> Seed {
    let mut rng = rand::thread_rng();
    let op_count = rng.gen_range(2..6);
    let operations = (0..op_count).map(|_| random_operation(&mut rng)).collect();
    let id = Uuid::new_v4().to_string();
    Seed {
        id,
        alias: alias.unwrap_or_else(|| format!("Seed-{}", &id[..8])),
        operations,
        created_at: chrono::Utc::now(),
    }
}
```

### Pattern 3: State Split — Rust Owns Truth, Pinia Mirrors

**What:** Rust managed state (`app.manage()`) is the authoritative source for seeds, video queue, and processing state. Pinia stores hold a local cache (mirror) for UI rendering. All mutations go through Tauri commands (Rust mutates, returns fresh state, Pinia updates).

**When to use:** When state must survive page reloads, when multiple windows might exist, or when backend needs to perform actions based on state (like canceling processing). Also prevents stale UI state.

**Trade-offs:** Every mutation requires an IPC round-trip. For lists of seeds and videos (small data), this is fast (<1ms). For video metadata queries (FFprobe), the latency is dominated by the external process, not IPC.

**Example:**

```typescript
// Pinia store: mirrors Rust state, mutations go through invoke()
export const useSeedStore = defineStore('seeds', () => {
  const seeds = ref<Seed[]>([]);
  const loading = ref(false);

  async function loadSeeds() {
    loading.value = true;
    seeds.value = await seedsApi.listSeeds(); // calls invoke('list_seeds')
    loading.value = false;
  }

  async function generateSeed(alias?: string) {
    const newSeed = await seedsApi.generateSeed(alias);
    seeds.value.push(newSeed);
    return newSeed;
  }

  async function deleteSeed(id: string) {
    await seedsApi.deleteSeed(id);
    seeds.value = seeds.value.filter((s) => s.id !== id);
  }

  return { seeds, loading, loadSeeds, generateSeed, deleteSeed };
});
```

## Data Flow

### Request Flow (Seed Generation)

```
User clicks "Generate Seed"
    ↓
Vue Component → seedStore.generateSeed("my-alias")
    ↓
Pinia Action → seedsApi.generateSeed("my-alias")
    ↓
invoke('generate_seed', { alias: "my-alias" })
    ↓  (IPC)
Tauri Command: commands/seed_commands.rs
    ↓
Service: seed_engine::generate_random(alias)
    ↓
Rust struct: Seed { id, alias, operations, created_at }
    ↓
Managed State: SeedStore.insert(id, seed)
    ↓
Storage: storage::save_seeds() → $APPDATA/seeds/{id}.json
    ↓
Return: Result<Seed, String>
    ↓  (IPC)
Pinia Action: seeds.value.push(newSeed)
    ↓
Vue Component re-renders SeedList
```

### Request Flow (Video Import)

```
User drops files onto VideoDropZone
    ↓
Vue Component → videoStore.importVideos(filePaths)
    ↓
invoke('import_videos', { paths: [...] })
    ↓  (IPC)
Tauri Command: commands/video_commands.rs
    ↓
Service: video_manager::import(paths)
    │
    ├── Spawn ffprobe for each file:
    │   shell().command("ffprobe")
    │     .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", path])
    │     .output()  ← blocking for metadata (fast: ~50ms per file)
    │   Parse JSON → VideoMeta { path, duration, resolution, codec, size }
    │
    ↓
Validate: reject unsupported formats
    ↓
Managed State: VideoQueue.push(VideoQueueItem { meta, status: Pending })
    ↓
Return: Vec<VideoQueueItem>
    ↓  (IPC)
Pinia: videoStore.queue = result
    ↓
Vue Component re-renders VideoQueue
```

### Processing Flow (The Critical Path)

```
User selects seed + clicks "Start Processing"
    ↓
Vue → processingStore.startProcessing(config)
    ↓
invoke('start_processing', { seed_id, output_dir })
    ↓  (IPC)
Tauri Command: commands/processing_commands.rs
    ↓
Validate: seed exists, queue not empty, FFmpeg available
    ↓
Mark all videos as "processing"
    ↓
tauri::async_runtime::spawn(async move {   ← NON-BLOCKING
    │
    │  Command returns Ok(()) immediately
    │  Frontend now receives events
    │
    ├── For each video in queue:
    │   │
    │   ├── 1. Build FFmpeg command:
    │   │   ffmpeg_engine::build_command(&seed, &video, &output_dir)
    │   │   │
    │   │   │   For each operation in seed.operations:
    │   │   │     match op.op_type {
    │   │   │       Overlay => ["-filter_complex", overlay_filter(op)],
    │   │   │       PixelShift => ["-vf", shift_filter(op)],
    │   │   │       DropFrames => ["-vf", drop_filter(op)],
    │   │   │       ...etc
    │   │   │     }
    │   │   │
    │   │   │   Plus: codec args, output path, -y (overwrite)
    │   │   │
    │   │   ├── 2. Spawn FFmpeg process:
    │   │   │   let (mut rx, child) = handle.shell()
    │   │   │     .command("ffmpeg")
    │   │   │     .args(command_args)
    │   │   │     .spawn()?;
    │   │   │
    │   │   ├── 3. Parse progress from stderr:
    │   │   │   while let Some(CommandEvent::Stderr(line)) = rx.recv().await {
    │   │   │       if line contains "frame=" {
    │   │   │           parse frame, fps, time, bitrate
    │   │   │           calculate percent from time / total_duration
    │   │   │           app.emit("processing-progress", ProgressPayload { ... })
    │   │   │       }
    │   │   │   }
    │   │   │
    │   │   └── 4. On process exit:
    │   │       if exit_code == 0:
    │   │           app.emit("processing-complete", { video, output_path })
    │   │       else:
    │   │           app.emit("processing-error", { video, error })
    │
    └── Queue done → app.emit("processing-all-complete")
})

Return: Ok(())
```

### State Management Architecture

```
┌─────────────────────────────────────────────────────┐
│                  Rust Managed State                   │
│  (authoritative — survives reloads, safe from UI)    │
│                                                       │
│  SeedStore: HashMap<String, Seed>                     │
│  VideoQueue: Vec<VideoQueueItem>                     │
│  ProcessingState: { is_processing, current_video, .. }│
│  FfmpegPath: Option<String>                          │
│  Config: { output_dir, ffmpeg_path, threads }        │
│                                                       │
│  Backed by:                                          │
│  - $APPDATA/seeds/*.json (persisted)                 │
│  - $APPDATA/config.json (persisted)                  │
│  - VideoQueue: memory only (resets on app restart)   │
└──────────────────────┬──────────────────────────────┘
                       │  invoke() returns fresh state
                       ↓
┌─────────────────────────────────────────────────────┐
│                  Pinia Stores (Vue)                   │
│  (UI cache — mirrors Rust state for rendering)       │
│                                                       │
│  seedStore:                                           │
│    seeds: Ref<Seed[]>      ← loaded from Rust         │
│    selectedId: Ref<string> ← UI-only, not in Rust     │
│                                                       │
│  videoStore:                                          │
│    queue: Ref<VideoQueueItem[]>  ← loaded from Rust   │
│    selectedVideo: Ref<string>    ← UI-only            │
│    previewSrc: Computed<string>  ← derived from state │
│                                                       │
│  processingStore:                                     │
│    isProcessing: Ref<boolean>    ← from Rust events   │
│    progress: Map<string, ProgressPayload> ← events    │
│    logs: Ref<LogEntry[]>         ← UI-only log        │
│                                                       │
│  settingsStore:                                       │
│    outputDir: Ref<string>        ← from Rust Config   │
│    ffmpegFound: Ref<boolean>     ← from Rust          │
└─────────────────────────────────────────────────────┘
```

### Key Data Flows

1. **Seed flow:** Generator → Seed struct → JSON file + Managed State → invoke response → Pinia seedStore → Vue components
2. **Video import flow:** File paths → FFprobe → VideoMeta → Managed State → invoke response → Pinia videoStore
3. **Processing flow:** Seed operations → FFmpeg command args → spawned process → stderr parsing → emit events → Pinia processingStore
4. **FFmpeg detection flow:** App startup → shell command `ffmpeg -version` → parse output → Managed State → startup event → Pinia settingsStore
5. **Progress flow:** FFmpeg stderr line → parse frame/time → compute percent → emit event → Pinia store → reactive UI update

## Scaling Considerations

| Scale                | Architecture Adjustments                                                                                                                       |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| 1-10 videos/session  | Monolith is fine. Single FFmpeg process per video, sequential. Async runtime handles it.                                                       |
| 10-50 videos/session | Add parallel processing (spawn N FFmpeg processes, bounded by CPU cores). Add queue pause/resume. Progress tracking gets more complex.         |
| 50+ videos/session   | Worker pool pattern. Tokio semaphore limiting concurrent FFmpeg processes. Disk I/O becomes bottleneck. Memory pressure from multiple encodes. |

### Scaling Priorities

1. **First bottleneck:** Sequential processing. Fix with configurable parallelism (process N videos at once, where N = CPU cores - 1). Requires making the processing loop work on chunks, not one-at-a-time.
2. **Second bottleneck:** FFmpeg progress parsing granularity (stderr output every ~0.5s). Sufficient for UI. If batch is huge, add aggregated progress stats.
3. **Third bottleneck:** Seed store as JSON files. Works fine for thousands of seeds. If millions needed, switch to SQLite.

## Anti-Patterns

### Anti-Pattern 1: Blocking the Main Thread with FFmpeg

**What people do:** Call `Command::output()` or `std::process::Command::output()` in a Tauri command without spawning to async runtime. This blocks the Tauri event loop.

**Why it's wrong:** The entire app freezes — UI unresponsive, no IPC works, OS may show "Not Responding". FFmpeg encodes can take minutes.

**Do this instead:** Always use `tauri::async_runtime::spawn()` for FFmpeg processes. The Tauri command returns immediately after spawning. Progress communicated via events.

### Anti-Pattern 2: Building FFmpeg Commands in the Frontend

**What people do:** Construct FFmpeg argument arrays in JavaScript/Vue and pass them to Rust.

**Why it's wrong:** Exposes command injection surface. Frontend knows FFmpeg internals. Breaks separation of concerns. Makes validation harder.

**Do this instead:** Frontend sends seed ID + output directory. Rust builds the FFmpeg command from the seed's operations. The frontend never sees FFmpeg arguments.

### Anti-Pattern 3: Sidecar for User-Installed FFmpeg

**What people do:** Configure FFmpeg as a Tauri sidecar in `tauri.conf.json` with `externalBin`.

**Why it's wrong:** Sidecar expects a bundled binary at a known relative path. The requirement is auto-download FFmpeg to a user-writable location. Sidecar paths are fixed at build time. Using `tauri_plugin_shell` with an absolute path to the downloaded binary is more flexible.

**Do this instead:** Use `tauri_plugin_shell` to spawn FFmpeg as a regular child process from its downloaded location. Store the FFmpeg path in managed state after detection/download.

### Anti-Pattern 4: Storing Video Files in Managed State

**What people do:** Load video file bytes into Rust state or pass them through IPC.

**Why it's wrong:** Video files are hundreds of MB. IPC serialization would be catastrophic. Memory usage would explode.

**Do this instead:** Only store file paths and metadata in Rust state. Videos are read directly from disk by FFmpeg. The frontend uses `tauri://localhost` or `convertFileSrc()` to access video files for preview.

### Anti-Pattern 5: Single Pinia Store for Everything

**What people do:** Create one monolithic Pinia store with all state (seeds, videos, processing, settings).

**Why it's wrong:** Tight coupling, hard to test, any component using the store re-renders on any state change. Difficult to reason about.

**Do this instead:** Separate Pinia stores per domain (`seedStore`, `videoStore`, `processingStore`, `settingsStore`). Each store owns its slice. Processing store reads from seedStore and videoStore but doesn't mutate them.

## Integration Points

### External Services

| Service     | Integration Pattern                                                                                                            | Notes                                                                                                                                |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ |
| FFmpeg      | `tauri_plugin_shell` child process. Spawned from Rust async task. Progress parsed from stderr line-by-line.                    | Detect at startup via `ffmpeg -version`. Download to `$APPDATA/ffmpeg/` if missing. Use absolute path from managed state.            |
| FFprobe     | `tauri_plugin_shell` child process with `output()` (blocking, fast). Output format: JSON (`-print_format json`).               | Used for video metadata extraction. Runs synchronously in Tauri command (50ms typical). Much simpler to parse JSON than stderr.      |
| File System | Tauri `path` API for app directories. `tauri-plugin-fs` for JS-side file reads (preview). Rust `std::fs` for backend file I/O. | Seeds stored as JSON in `BaseDirectory.AppData/seeds/`. Config in `BaseDirectory.AppConfig/`. Video output in user-chosen directory. |

### Internal Boundaries

| Boundary                      | Communication                                                                                                      | Notes                                                                                                                                             |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| Vue Components ↔ Pinia Stores | Direct imports. Components call store actions, read store state reactively (`storeToRefs`).                        | Standard Vue/Pinia pattern. Components never call `invoke()` directly — always through store actions.                                             |
| Pinia Stores ↔ Rust Backend   | Tauri `invoke()` for commands, `listen()` for events. Typed through `src/tauri-api/` wrappers.                     | Stores are the single point of IPC in the frontend. Components are IPC-unaware.                                                                   |
| Tauri Commands ↔ Services     | Direct function calls. Commands extract `State<>` and pass to service functions.                                   | Services are pure Rust with no Tauri dependency. This is the most important internal boundary — it makes services testable.                       |
| Services ↔ Managed State      | `std::sync::Mutex` for sync commands, `tokio::sync::Mutex` for async. Services receive `&Mutex<T>` or cloned data. | Services should not hold lock references across `.await` points (deadlock risk). Clone data before async work.                                    |
| FFmpeg Engine ↔ Frontend      | Rust `app.emit()` → frontend `listen()`. One-way, fire-and-forget.                                                 | Progress events are informational. If the frontend is reloaded mid-processing, the backend continues but the UI loses context. Acceptable for v1. |
| Seed Engine ↔ Storage         | Service calls `storage::save_seeds()` / `storage::load_seeds()`.                                                   | Storage is a thin I/O layer. Seeds serialized as JSON. One file per seed avoids read-modify-write races.                                          |

## Build Order (Dependency Graph)

```
Phase 1: FFmpeg Detection + Download
├── Depends on: Nothing (foundation)
├── Produces: FfmpegPath managed state, check/download commands
└── Unblocks: Everything else

Phase 2: Seed Engine (Rust Core)
├── Depends on: Nothing (pure Rust logic)
├── Produces: Seed structs, random generation, validation, JSON persistence
└── Unblocks: Seed commands (Phase 4), Command builder (Phase 5)

Phase 3: Video Manager
├── Depends on: FFmpeg detection (Phase 1), for FFprobe
├── Produces: Video import, metadata extraction, queue management
└── Unblocks: Video commands (Phase 4), Processing pipeline (Phase 6)

Phase 4: Tauri Commands (IPC Layer)
├── Depends on: Seed Engine (Phase 2), Video Manager (Phase 3)
├── Produces: All invoke() commands for seed CRUD, video import, FFmpeg status
└── Unblocks: Frontend stores (Phase 7)

Phase 5: FFmpeg Command Builder
├── Depends on: Seed Engine ops (Phase 2), FFmpeg detection (Phase 1)
├── Produces: OpType → FFmpeg args translation, filter graph construction
└── Unblocks: Processing pipeline (Phase 6)

Phase 6: Processing Pipeline
├── Depends on: Command builder (Phase 5), Video queue (Phase 3)
├── Produces: Async processing loop, progress parsing, event emission, cancellation
└── Unblocks: Processing commands (Phase 4), Progress UI (Phase 8)

Phase 7: Vue Frontend + Pinia Stores
├── Depends on: Tauri commands (Phase 4)
├── Produces: All UI components, Pinia stores, typed API wrappers
└── Unblocks: End-to-end integration (Phase 9)

Phase 8: Progress UI + Event Integration
├── Depends on: Processing pipeline events (Phase 6), Frontend scaffold (Phase 7)
├── Produces: Live progress bars, per-video status, logs
└── Unblocks: UX polish (Phase 9)

Phase 9: End-to-End Integration + Polish
├── Depends on: All phases
├── Produces: Full workflow (generate seed → import videos → process → view results)
└── Unblocks: Ship to validate
```

### Why This Order

1. **FFmpeg first** because everything depends on it. If FFmpeg can't be detected or downloaded, the app is non-functional. This is the highest-risk integration point and must be validated earliest.
2. **Seed engine second** because it's pure Rust with no external dependencies — fast to build, establishes data model, and seeds drive the entire processing pipeline.
3. **Command builder before processing pipeline** because building FFmpeg arguments from operations is the translation layer that must be correct. Getting this wrong produces broken videos.
4. **Frontend after commands** because the UI is pure presentation over the IPC layer. Building UI first means mocking all commands — building commands first means the UI works against real data immediately.
5. **Progress integration last** because it's a UX enhancement over a working pipeline. The core workflow (generate, import, process, get output) must work without progress bars.

## Sources

- [Tauri v2 Documentation — Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/) (Context7, HIGH confidence)
- [Tauri v2 Documentation — Inter-Process Communication](https://v2.tauri.app/concept/inter-process-communication/) (Context7, HIGH confidence)
- [Tauri v2 Documentation — State Management](https://v2.tauri.app/develop/state-management/) (Context7, HIGH confidence)
- [Tauri v2 Documentation — Sidecar & Shell Plugin](https://v2.tauri.app/develop/sidecar/) (Context7, HIGH confidence)
- [Tauri v2 Documentation — File System Plugin](https://v2.tauri.app/plugin/file-system/) (Context7, HIGH confidence)
- [Tauri v2 Documentation — Project Structure](https://v2.tauri.app/start/project-structure/) (Context7, HIGH confidence)
- [Pinia Documentation — Core Concepts and Setup Stores](https://pinia.vuejs.org/core-concepts/) (Context7, HIGH confidence)
- FFmpeg progress parsing pattern: FFmpeg outputs `frame=`, `fps=`, `time=`, `bitrate=` to stderr periodically during encoding. Parsed line-by-line via `CommandEvent::Stderr`. (Training data, MEDIUM confidence — validated against Tauri shell plugin API)

---

_Architecture research for: Tauri 2.x desktop video batch processing tool_
_Researched: 2026-05-12_
