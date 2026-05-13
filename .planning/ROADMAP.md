# Roadmap: Sandwich - Video Fingerprint Batch Modification Tool

## Overview

A four-phase horizontal-layers build: Foundation (FFmpeg lifecycle + scaffold) establishes the non-negotiable prerequisite. The Rust Backend phase builds all domain logic, FFmpeg execution, and IPC commands as an independently testable layer. The Vue Frontend phase wraps that backend in a production UI. Integration & Polish wires real-time progress streaming and validates the end-to-end workflow.

## Phases

- [x] **Phase 1: Foundation** - FFmpeg detection, one-click download, project scaffold, Tauri plugins (completed 2026-05-13)
- [ ] **Phase 2: Rust Backend** - Domain services, FFmpeg command builder, processing pipeline, IPC commands
- [ ] **Phase 3: Vue Frontend** - Pinia stores, typed API wrappers, dual-panel UI, Naive UI dark theme
- [ ] **Phase 4: Integration & Polish** - Progress streaming, batch summary, cancel flow wiring, E2E validation

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
- [ ] 03-02-PLAN.md — Pinia Composition API stores (useSeedStore, useQueueStore, useBatchStore)

**Wave 3** *(blocked on Wave 2 completion)*
- [ ] 03-03-PLAN.md — Composables wrapping Tauri IPC (useSeed, useQueue, useBatch) with event subscriptions

**Wave 4** *(blocked on Wave 3 completion)*
- [ ] 03-04-PLAN.md — App.vue provider wrappers + MainLayout.vue dual-panel resizable layout
- [ ] 03-05-PLAN.md — Seed components (SeedCard.vue, SeedList.vue) with hover actions and empty state
- [ ] 03-06-PLAN.md — Queue components (ImportZone.vue drag-drop, QueueList.vue with metadata and clear confirmation)
- [ ] 03-07-PLAN.md — Batch components (BatchControls.vue with seed/concurrency/output dir, BatchBanner.vue progress)
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
   **Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase                   | Plans Complete | Status      | Completed |
| ----------------------- | -------------- | ----------- | --------- |
| 1. Foundation           | 4/4            | Complete    | 2026-05-13 |
| 2. Rust Backend         | 0/4            | Planned     | -         |
| 3. Vue Frontend         | 1/7 | In Progress|  |
| 4. Integration & Polish | 0/TBD          | Not started | -         |
