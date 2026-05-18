# Roadmap: Sandwich - Video Fingerprint Batch Modification Tool

## Overview

A five-phase build: Foundation (FFmpeg lifecycle + scaffold) establishes the non-negotiable prerequisite. The Rust Backend phase builds all domain logic, FFmpeg execution, and IPC commands as an independently testable layer. The Vue Frontend phase wraps that backend in a production UI. Integration & Polish wires real-time progress streaming and validates the end-to-end workflow. Production Hardening adds cross-platform packaging, GPU acceleration, multi-seed batch processing, and integrity verification.

## Phases

- [x] **Phase 1: Foundation** - FFmpeg detection, one-click download, project scaffold, Tauri plugins (completed 2026-05-13)
- [x] **Phase 2: Rust Backend** - Domain services, FFmpeg command builder, processing pipeline, IPC commands (completed 2026-05-14)
- [x] **Phase 3: Vue Frontend** - Pinia stores, typed API wrappers, dual-panel UI, Naive UI dark theme (completed 2026-05-13)
- [x] **Phase 4: Integration & Polish** - Progress streaming, batch summary, cancel flow wiring, E2E validation (completed 2026-05-14)
- [x] **Phase 5: Production Hardening** - Cross-platform builds, GPU acceleration, multi-seed batch, MD5 integrity verification (completed 2026-05-15)
- [x] **Phase 6: 增强指纹修改** - 20 operation types, 3-tier strength presets, drag-to-reorder, thumbnails, processing log (completed 2026-05-18)

## Phase Details

### Phase 1: Foundation

**Goal**: FFmpeg is reliably available on the user's machine with zero-config detection and one-click download, and the project scaffold (Tauri 2.x + Vue 3 + Vite) builds and runs.
**Depends on**: Nothing (first phase)
**Requirements**: FFMPEG-01, FFMPEG-02, FFMPEG-03
**Success Criteria** (what must be TRUE):

1. User launches the app and immediately sees FFmpeg availability status (version found / missing)
2. When FFmpeg is missing, user can initiate a download with one click and see real-time download progress with platform-appropriate binary selection
3. After download completes, FFmpeg is automatically verified (runs `ffmpeg -version` successfully) and its path persists across app restarts
   **Plans:** 4/4 plans complete

Plans:

- [x] 01-01-PLAN.md — Project scaffold + all dependencies + config files
- [x] 01-02-PLAN.md — Dev tooling (ESLint, Prettier, rustfmt, clippy, husky, GitHub Actions CI)
- [x] 01-03-PLAN.md — Rust FFmpeg backend (detection, download, verification, persistence)
- [x] 01-04-PLAN.md — Vue FFmpeg frontend (status UI, download page, i18n, dark theme)

### Phase 2: Rust Backend

**Goal**: All domain operations -- seed generation, video import with metadata extraction, video queue management, and batch FFmpeg processing with failure isolation and cancellation -- work through typed Tauri IPC commands with Rust-managed authoritative state.
**Depends on**: Phase 1
**Requirements**: SEED-01, SEED-02, SEED-03, SEED-04, SEED-05, SEED-06, IMPORT-01, IMPORT-02, QUEUE-01, QUEUE-02, BATCH-01, BATCH-03, BATCH-04, OUTPUT-01, OUTPUT-02
**Success Criteria** (what must be TRUE):

1. User can generate a random seed with one command; each seed contains 3-7 operations drawn from 7 types with safety-constrained parameters, an alias, and persists as JSON across app restarts
2. User can import videos via drag-and-drop and file picker; each video's metadata (filename, duration, resolution, size) is extracted via ffprobe and stored in the managed queue
3. User can manage the video queue through commands (remove individual, clear all) and view full metadata for each entry
4. User can initiate batch processing through a command: select a seed, set an output directory, and process all queued videos -- output files appear with the `{original}_{seed_alias}.{ext}` naming convention in the specified directory
5. During batch processing, single-file failures are isolated (remaining files continue), and processing can be canceled via a command with graceful FFmpeg process termination
   **Plans**: 4 plans

Plans:
**Wave 1**
- [ ] 02-01-PLAN.md — Foundation: model types (Seed, VideoEntry, BatchConfig), AppState, module scaffolding, Cargo dependencies

**Wave 2** *(blocked on Wave 1 completion)*
- [ ] 02-02-PLAN.md — FFmpeg utilities: ffprobe metadata extraction, filter chain builders for all 7 operation types, executor with progress streaming and cancel support
- [ ] 02-03-PLAN.md — Seed generation and CRUD commands + video queue management commands

**Wave 3** *(blocked on Wave 2 completion)*
- [ ] 02-04-PLAN.md — Video import with ffprobe validation and disk space check + batch processing with global static cancel flag and failure isolation + final lib.rs command wiring and state initialization

### Phase 3: Vue Frontend

**Goal**: Users interact with all features through a production-quality dual-panel dark-themed UI built with Naive UI components and Pinia stores that mirror the Rust backend state.
**Depends on**: Phase 2
**Requirements**: UI-01, UI-02
**Success Criteria** (what must be TRUE):

1. User sees a dual-panel layout: left panel for seed management (list, generate, rename, delete, duplicate), right panel for video queue (drag-drop import zone, queue list, per-video actions)
2. The entire application renders in a consistent Naive UI dark theme across all components, dialogs, and empty states
3. User can perform the complete static workflow through the UI: generate seeds, import videos via drag-and-drop and file picker, manage the queue, select a seed and output directory, and initiate processing
   **Plans**: 7 plans

Plans:
**Wave 1**
- [x] 03-01-PLAN.md — TypeScript type definitions (Seed, VideoEntry, BatchProgress) + i18n key extensions for both locales

**Wave 2** *(blocked on Wave 1 completion)*
- [x] 03-02-PLAN.md — Pinia Composition API stores (useSeedStore, useQueueStore, useBatchStore)

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 03-03-PLAN.md — Composables wrapping Tauri IPC (useSeed, useQueue, useBatch) with event subscriptions

**Wave 4** *(blocked on Wave 3 completion)*
- [x] 03-04-PLAN.md — App.vue provider wrappers + MainLayout.vue dual-panel resizable layout
- [x] 03-05-PLAN.md — Seed components (SeedCard.vue, SeedList.vue) with hover actions and empty state
- [x] 03-06-PLAN.md — Queue components (ImportZone.vue drag-drop, QueueList.vue with metadata and clear confirmation)
- [x] 03-07-PLAN.md — Batch components (BatchControls.vue with seed/concurrency/output dir, BatchBanner.vue progress)
   **UI hint**: yes

### Phase 4: Integration & Polish

**Goal**: Users experience live per-video progress feedback during processing, a clear batch completion summary, responsive cancellation from the UI, and a reliable end-to-end workflow with no broken states.
**Depends on**: Phase 3
**Requirements**: BATCH-02, BATCH-05
**Success Criteria** (what must be TRUE):

1. During batch processing, user sees real-time per-video progress bars showing percentage complete, current frame, and estimated remaining time
2. After processing finishes, user sees a completion summary panel showing how many files succeeded, how many failed, and per-file output paths
3. User can cancel an in-progress batch from the UI; FFmpeg processes terminate gracefully and the app returns to a clean, ready-to-process state
4. The full end-to-end workflow operates without breaking: generate seed -> drag in videos -> select seed -> click process -> watch live progress per file -> review completion summary
   **Plans:** 8 plans (5 execution + 3 gap closure, 8/8 complete)

Plans:
**Wave 1**
- [x] 04-01-PLAN.md — Rust: PerFileProgress struct + enriched executor emitting batch-file-progress event

**Wave 2**
- [x] 04-02-PLAN.md — Frontend contracts: PerFileProgress TypeScript type + 17 new i18n keys in both locales

**Wave 3** *(blocked on Wave 2 completion)*
- [x] 04-03-PLAN.md — Store + composable: perFileProgress map, cancelling state, 6 event listeners

**Wave 4** *(blocked on Wave 3 completion)*
- [x] 04-04-PLAN.md — UI: BatchBanner multi-state + BatchControls cancel confirmation + ImportZone disabled during processing
- [x] 04-05-PLAN.md — UI: QueueList per-file progress bars + BatchSummary completion summary + MainLayout conditional rendering

**Gap Closure** *(post-verification fixes)*
- [x] 04-06-PLAN.md — Fix parse_time_to_seconds MM:SS bug (progress stuck at 0% for short videos)
- [x] 04-07-PLAN.md — Fix BatchBanner wrong title for completed-with-failures (showed "Cancelled")
- [x] 04-08-PLAN.md — Fix missing initial batch-progress event (progress showed "0/1" during first file)

### Phase 5: Production Hardening

**Goal**: The app ships as installable packages for Windows and Linux with CI build matrix; batch processing leverages GPU hardware acceleration and optimized scheduling; users can apply multiple seeds per video in a single batch; every output file is verified to differ from its input via MD5 checksum comparison.
**Depends on**: Phase 4
**Requirements**: CROSS-01, CROSS-02, CROSS-03, PERF-01, PERF-02, MULTI-01, MULTI-02, MD5-01, MD5-02
**Success Criteria** (what must be TRUE):

1. User can download and install the app on Windows (.msi/.exe) and Linux (.AppImage/.deb) via CI-built artifacts
2. User on a GPU-equipped machine sees automatically accelerated encoding (NVENC/VideoToolbox/VAAPI) with measurable throughput improvement over CPU encoding
3. User can select multiple seeds (not just one) and each video in the queue produces one output per selected seed ({original}_{seed_alias}.{ext})
4. User sees MD5 checksums before and after processing for every file in the batch summary, with clear pass/fail indication that the file was actually modified
5. All existing v1 functionality continues to work — this phase is additive hardening, not a rewrite
   **Plans:** 4/6 plans complete

Plans:
**Wave 1** *(cross-platform — parallel)*
- [x] 05-01-PLAN.md — Tauri build config for Windows (.msi/.exe) + Linux (.AppImage/.deb) targets
- [x] 05-02-PLAN.md — GitHub Actions CI matrix build (macOS/Windows/Linux) with artifact upload

**Wave 2** *(GPU — blocked on Wave 1 completion)*
- [x] 05-03-PLAN.md — GPU encoder detection (NVENC/VideoToolbox/VAAPI) + auto-select in executor

**Wave 3** *(pipeline — blocked on Wave 2 completion)*
- [x] 05-04-PLAN.md — GPU wiring into batch.rs + Mutex lock frequency reduction

**Wave 4** *(multi-seed — blocked on Wave 3 completion)*
- [x] 05-05-PLAN.md — Multi-seed selection UI + Rust batch command accepting Vec<SeedId>

**Wave 5** *(MD5 — blocked on Wave 4 completion)*
- [x] 05-06-PLAN.md — MD5 checksum recording (pre-process) + comparison (post-process) + summary integration

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5

| Phase                      | Plans Complete | Status      | Completed  |
| -------------------------- | -------------- | ----------- | ---------- |
| 1. Foundation              | 4/4            | Complete    | 2026-05-13 |
| 2. Rust Backend            | 4/4            | Complete    | 2026-05-14 |
| 3. Vue Frontend            | 7/7            | Complete    | 2026-05-13 |
| 4. Integration & Polish    | 8/8            | Complete    | 2026-05-14 |
| 5. Production Hardening    | 6/6 | Complete    | 2026-05-16 |

### Phase 6: 增强指纹修改

**Goal:** Systematically enhance fingerprint modification with 13+ new FFmpeg operation types (color/noise/geometric/blend), 3-tier strength presets with intelligent seed generation (5-12 steps, >=70% video coverage), seed JSON export/import, and three v2 deferred features (drag-to-reorder queue, thumbnail preview, processing log history).
**Requirements**: PHASE-06 (20 locked decisions from CONTEXT.md)
**Depends on:** Phase 5
**Plans:** 7/7 plans executed

Plans:
**Wave 1** *(foundation — parallel)*
- [x] 06-01-PLAN.md — Rust model extensions (OperationType 20 variants, StrengthTier, Seed/VideoEntry/batch structs)
- [x] 06-02-PLAN.md — TypeScript type definitions + ~50 i18n keys (both locales)

**Wave 2** *(Rust backend — parallel)*
- [x] 06-03-PLAN.md — 13 new FFmpeg filter builders + dispatch match arms + tests
- [x] 06-04-PLAN.md — Seed generation upgrade (strength tiers, weights, coverage) + seed export/import commands
- [x] 06-05-PLAN.md — Thumbnail extraction + batch-log events + legacy migration + lib.rs wiring + base64 crate

**Wave 3** *(frontend state)*
- [x] 06-06-PLAN.md — Pinia stores (log, seed, queue) + useSeed composable + vue-draggable-plus

**Wave 4** *(frontend UI)*
- [x] 06-07-PLAN.md — SeedCard + BatchControls + QueueList (VueDraggable + thumbnails) + LogPanel + MainLayout NTabs

### Phase 7: 增强视频指纹，修改音频，视频长度，元数据，轻微裁切成为默认

**Goal:** [To be planned]
**Requirements**: TBD
**Depends on:** Phase 6
**Plans:** 0 plans

Plans:
- [ ] TBD (run /gsd-plan-phase 7 to break down)
