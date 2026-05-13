# Project Research Summary

**Project:** Sandwich -- Desktop Video Fingerprint Batch Modification Tool
**Domain:** Desktop video batch processing (Tauri 2.x + Vue 3 + FFmpeg)
**Researched:** 2026-05-12
**Confidence:** MEDIUM-HIGH

## Executive Summary

Sandwich is a desktop tool that automates video fingerprint modification through randomized multi-operation seed recipes applied via FFmpeg. Users generate a "seed" (a chain of 3-7 visual operations with randomly determined parameters within visual-safety bounds), load videos into a queue, and batch-process all videos with one click. The output is multiple visually-indistinguishable variants of the same source video that fingerprinting systems cannot identify as duplicates.

The research-consensus approach is: Tauri 2.11.x as the desktop framework (Rust backend for FFmpeg process management and seed generation; Vue 3.5 frontend for the UI), ffmpeg-sidecar for FFmpeg binary auto-detection and download (avoids bundling ~80MB in the installer), Naive UI as the component library (chosen over Element Plus for smaller bundle size, native dark theme, and desktop-friendly compact density), and Pinia 3 for reactive state management. The architecture follows three proven patterns: Command-Event Split (commands for CRUD, events for streaming progress), Thin Commands / Fat Services (commands are IPC glue, all logic lives in testable Rust service modules), and Rust Owns Truth / Pinia Mirrors (Rust managed state is authoritative, Pinia stores cache a local copy for UI rendering).

The two highest risks are FFmpeg download reliability (the app is non-functional without FFmpeg; the auto-download must handle network failures, platform detection, and binary verification robustly) and seed generation quality (randomized operations must produce varied enough output to defeat fingerprinting while staying within safety constraints that prevent visible degradation). Research also uncovered 10 critical domain pitfalls -- the most dangerous being orphaned FFmpeg processes surviving app close (data corruption), brittle stderr progress parsing (silent failures across platforms/versions), and Vue reactivity freezes on large video queues (UI jank during batch processing). Mitigation: all three are addressed through architectural patterns (cancellation tokens, `-progress pipe:1` structured parsing, `shallowRef()` + `markRaw()` for queue state) that must be established in Phase 1 before any feature code is written.

## Key Findings

### Recommended Stack

**Source:** [STACK.md](./STACK.md) -- HIGH confidence, all decisions verified against Context7 official docs and current package registries.

The stack is anchored on Tauri 2.11.x (Rust backend) + Vue 3.5.x (web frontend), the user-specified constraint. The critical architectural decision is using **ffmpeg-sidecar** (a Rust crate that handles platform-specific FFmpeg binary download, version detection, and structured progress parsing) rather than ffmpeg-next (C bindings requiring system FFmpeg dev libraries) or raw `std::process::Command` (no auto-download, no progress parsing). This directly satisfies the FFMPEG-01 and FFMPEG-02 requirements from PROJECT.md.

**Core technologies:**

- **Tauri 2.11.x**: Production-grade Rust+web desktop framework with mature plugin system, sidecar bundling, and IPC. v2 is required (v1 is EOL).
- **Vue 3.5.x + Vite 8.x + TypeScript 6.x**: Modern frontend stack. Composition API with `<script setup>` for concise reactive code. Vite for instant HMR and optimized production builds for Tauri's webview.
- **ffmpeg-sidecar 2.5.x**: FFmpeg binary management with `auto_download()`. Handles platform detection, download from official builds, extraction, and caching. Also provides `filter_progress()` for parsing FFmpeg stderr.
- **Naive UI 2.44.x**: Tree-shakeable component library with native dark theme (critical for video tools) and compact density theme. Lighter bundle than Element Plus.
- **Pinia 3.0.x**: Official Vue state management. Composition API setup stores. Four stores by domain: seeds, videos, processing, settings.
- **tauri-plugin-shell / dialog / fs**: Official Tauri v2 plugins for spawning FFmpeg processes, native file pickers, and file system access.

**Critical version requirements:** Tauri 2.11.x and @tauri-apps/api 2.11.x must match minor versions. Plugins follow independent versioning but require Tauri 2.x.

### Expected Features

**Source:** [FEATURES.md](./FEATURES.md) -- MEDIUM confidence. Table-stakes features verified against competitive landscape; differentiators derived from PROJECT.md requirements.

**Must have for launch (P1 -- 9 features):**

- FFmpeg detection + one-click download when missing -- zero-config prerequisite
- Seed generation (auto-randomized multi-operation chains) -- the core value proposition
- Seed list management (view, rename, delete, duplicate) -- users build a seed library
- Video import (drag-and-drop + file picker fallback) -- getting content into the tool
- Video queue with basic management (remove, clear) -- visibility into what's being processed
- Batch processing (one seed applied to all queued videos) -- the fundamental workflow
- Progress tracking (per-file progress bars via FFmpeg stderr parsing) -- visibility during encodes
- Output directory configuration + file naming convention -- predictable output management
- Error handling (per-file failures, batch continues) -- robustness

**Should have post-validation (P2 -- 6 features):**

- Video preview thumbnails -- visual identification of queued files
- Queue reordering -- control over processing sequence
- Processing log and history -- reviewable record of what was done
- Seed export/import -- share effective seeds as portable JSON
- Batch processing summary with diff hints -- confidence that modifications happened
- Processing cancellation polish -- clean state after abort

**Defer to v2+ (P3 -- 5 features):**

- Different seeds per video in a batch -- explodes state machine complexity
- Project files / workspace persistence -- adds file format design and backward compat burden
- Minimize to tray / background processing -- significant system tray integration
- Platform-optimized seed presets -- depends on community knowledge accumulation
- Queue import from CSV/text file -- useful only at scale (50+ videos)

**Anti-features explicitly avoided (8 total):** Real-time processing preview, manual filter chain editor, video editor timeline, cloud encoding, audio-only mode, plugin system, AI scoring, background processing. Each would add months of development for near-zero v1 value.

### Architecture Approach

**Source:** [ARCHITECTURE.md](./ARCHITECTURE.md) -- HIGH confidence, patterns verified against Tauri v2 official docs via Context7.

Three-layer split: Vue 3 frontend (components + Pinia stores) communicates with the Rust backend via Tauri's IPC boundary (commands for request-response, events for progress streaming). Rust backend organized into services (pure domain logic, no Tauri dependency, fully unit-testable) and commands (thin IPC handlers that extract state and delegate to services). External processes (FFmpeg, FFprobe) spawned via `tauri-plugin-shell`, progress communicated through Rust-emitted events.

**Major components:**

1. **Seed Engine** (Rust service) -- Random seed generation, operation chain construction, safety constraint validation, JSON serialization. Stateless and pure -- core IP.
2. **Video Manager** (Rust service) -- Video file import validation, FFprobe metadata extraction, queue item creation. Spawns FFprobe as short-lived child process.
3. **FFmpeg Engine** (Rust service) -- FFmpeg detection, auto-download, command building (seed ops to filtergraph args), async process spawning with stderr progress parsing, cancellation.
4. **Managed State** (Rust) -- In-memory authoritative state for seeds, video queue, processing status, FFmpeg path. Seeds persisted to JSON in `$APPDATA/seeds/`. Video queue is memory-only.
5. **Pinia Stores** (Vue) -- UI mirror of Rust state. Four domain-specific stores. All mutations go through `invoke()` to Rust.
6. **Vue Components** (Vue) -- Left panel (seeds), right panel (videos), drag-and-drop import, progress bars, output settings.

### Critical Pitfalls

**Source:** [PITFALLS.md](./PITFALLS.md) -- MEDIUM confidence. 10 critical pitfalls identified, plus technical debt patterns, integration gotchas, performance traps, and a "looks done but isn't" checklist.

Top 5 pitfalls by impact + likelihood:

1. **Orphaned FFmpeg Processes Survive App Close** -- Closing the app while processing leaves FFmpeg encoding in the background. Output files locked or corrupt on next launch. SIGKILL produces broken MP4 files missing MOOV atoms. **Prevention:** SIGTERM first with 5s grace period, SIGKILL only as last resort. Store active `Child` handles in `Mutex<Vec<Child>>` for shutdown iteration. Register `onCloseRequested` handler that warns user and gracefully terminates all children.

2. **Brittle Stderr Progress Parsing** -- FFmpeg progress format varies between versions, OSes, and codecs. Parsing the human-readable `time=` pattern fails on Windows (`\r\n` vs `\n`), misses `progress=end` sentinel, and breaks with non-English locales. **Prevention:** Always use `-progress pipe:1 -nostats` for structured key=value output. Parse `out_time_us=` for microsecond-precise progress. Parse `progress=continue/end` sentinel keys. Build progress parsing as a dedicated, tested Rust module before any UI.

3. **Vue Reactivity Freezes on Large Queues** -- Storing video queue as `reactive([...])` wraps every item property in a Proxy. At 50+ items with 2Hz progress updates, recursive dependency tracking freezes the UI for hundreds of milliseconds. **Prevention:** Use `shallowRef<VideoItem[]>([])` for the queue array. `markRaw()` individual items. Immutable update pattern: replace item at index. Use `v-memo` with narrow dependency keys. This data model decision must be made in Phase 1.

4. **FFmpeg Auto-Download Failure States** -- Network errors, platform misdetection, or binary corruption during download leave the app non-functional with no recovery path. **Prevention:** Validate with checksum verification and `ffmpeg -version` test. Provide clear error messaging and retry button. Test on all three platforms. Verify sidecar binary in production builds (works in `tauri dev`, fails silently in `tauri build` if naming convention mismatches).

5. **Corrupt Output Not Detected** -- FFmpeg exits with code 0 but output is truncated (missing MOOV atom), has empty streams, or contains green/black corruption. App marks it "success" and user discovers the problem later. **Prevention:** Post-encode validation with ffprobe (verify duration > 0, streams > 0, file size > 10KB). Write to temp file, validate, then atomically rename. Use `-movflags +faststart` for MOOV atom relocation. Use `-abort_on empty_output` and `-xerror` flags.

Full pitfall catalog in PITFALLS.md includes: Tauri event listener leaks (5), FFmpeg license non-compliance (6), UI blocking during probe import (7), inadequate progress granularity (8), and cross-platform path handling (10). Each mapped to prevention phase with verification criteria.

## Implications for Roadmap

Based on the dependency graph from ARCHITECTURE.md, the priority matrix from FEATURES.md, and the pitfall-to-phase mapping from PITFALLS.md, the research suggests 5 roadmap phases.

### Phase 1: Foundation (FFmpeg + Seed Engine + Data Model)

**Rationale:** FFmpeg detection/download is the foundation everything depends on. The Seed Engine is pure Rust with no external dependencies. Critically, PITFALLS.md identifies three Phase 1 decisions that must be locked before any feature work: (a) LGPL-only FFmpeg build to avoid licensing catastrophe, (b) `shallowRef()` + `markRaw()` data model for the video queue to avoid a reactivity rewrite later, and (c) the `useTauriEvent()` composable pattern to prevent listener leak bugs. Building these first validates the highest-risk components and establishes architectural invariants.

**Delivers:** FFmpeg auto-detection on launch, one-click download with progress and failure recovery, LGPL license verification. Seed generation producing randomized operation chains (3-7 operations from 7 types), safety constraint enforcement at Rust type level, JSON serialization, persistence. `useTauriEvent()` composable. `shallowRef` data model contract documented.

**Addresses:** FFMPEG-01, FFMPEG-02, SEED-01, SEED-02, OP-01, OP-02

**Avoids:** P3 (sidecar naming mismatch in production), P6 (GPL license violation), P4/P5 (reactivity rewrite and listener leak -- handled by establishing patterns early)

### Phase 2: Backend Pipeline (Video Manager + Command Builder + Processing)

**Rationale:** With seeds and FFmpeg available, build the video import pipeline and FFmpeg command construction. PITFALLS.md flags Phase 2 specifically for progress parsing (P2) and cross-platform path handling (P10) -- these must be dedicated, tested modules, not inline string matching. The processing loop is the critical path and must be testable end-to-end in Rust before any frontend exists.

**Delivers:** Video import with concurrent FFprobe metadata extraction (not sequential -- avoids P7). FFmpeg command builder translating each operation type into correct filtergraph arguments. Async processing loop with three-level progress (batch/file/operation), cancellation via stored Child handles (P1 prevention), cross-platform path normalization (P10 prevention). Structured progress parsing via `-progress pipe:1` (P2 prevention). Post-encode validation (P9 prevention).

**Addresses:** SEED-03, VIDEO-01, VIDEO-02, BATCH-01, BATCH-02, BATCH-03

**Avoids:** P1 (orphaned processes -- graceful shutdown), P2 (brittle parsing -- structured pipe protocol), P7 (sequential probe freeze -- concurrent import), P9 (corrupt output -- post-validation), P10 (path handling -- canonicalization + quoting)

### Phase 3: Frontend Shell (Pinia Stores + UI Components)

**Rationale:** With the entire backend pipeline working and testable from Rust, build the UI layer. Pinia stores connect to Tauri commands through typed API wrappers. The `shallowRef` data model and `useTauriEvent` composable established in Phase 1 make this straightforward. Components render store state reactively against real data.

**Delivers:** Four Pinia stores (seeds, videos, processing, settings) using `shallowRef` for queue. Typed `src/tauri-api/` wrappers. Vue component tree (AppShell, SeedList with CRUD, VideoDropZone, VideoQueue, ProcessPanel, output settings). Naive UI dark theme + compact density. Video preview thumbnails.

**Addresses:** UI-01, UI-02, VIDEO-03

**Uses:** Tauri 2.11.x, Vue 3.5.x, Naive UI 2.44.x, Pinia 3.x, @tauri-apps/api 2.11.x

**Avoids:** P4 (reactivity freeze -- shallowRef already in place), P5 (listener leaks -- useTauriEvent composable already in place)

### Phase 4: Integration + Progress (Event Wiring + E2E Workflow)

**Rationale:** Wire Tauri events into Pinia stores. Progress bars, per-video status indicators, and the full workflow become functional end-to-end. This is where PITFALLS.md's "inadequate progress granularity" (P8) gets addressed -- three-level progress (batch, per-file, per-operation) with always-visible cancel button, per-file skip, and timeout handling.

**Delivers:** Event listener wiring via `useTauriEvent()` composable. Live progress bars per video (not spinners -- P8). Processing-complete/error events updating video status. Cancellation flow (frontend cancel -> Rust graceful shutdown -> clean state reset). Queue reordering (drag-to-reorder).

**Addresses:** BATCH-02 (progress display makes it real)

**Implements:** Command-Event Split pattern, P8 prevention (three-level progress + cancel everywhere)

**Avoids:** P5 (listener leaks already prevented by composable from Phase 1)

### Phase 5: Polish + Ship (Logs, Export, Testing, Packaging)

**Rationale:** Secondary features that build user trust and production readiness. The "Looks Done But Isn't" checklist from PITFALLS.md drives verification: sidecar in production build, cancel with cleanup, 100-video queue test, restart resilience, license inclusion.

**Delivers:** Processing log viewer, batch summary, seed export/import. Comprehensive tests. Linting/type checking in CI. Packaging validation (verify `tauri build` produces working binary on clean machine). "Looks Done But Isn't" checklist fully passed.

**Addresses:** SEED-03 (export/import), BATCH-02 (summary)

**Avoids:** All P3 features explicitly deferred per FEATURES.md anti-features.

### Phase Ordering Rationale

- **FFmpeg before everything** -- ARCHITECTURE.md's build order + PITFALLS.md P3 (sidecar naming) and P6 (license) are Phase 1 decisions with high rework cost if deferred.
- **Data model decisions before feature code** -- PITFALLS.md P4 (reactivity) and P5 (listener leaks) are Phase 1 architectural invariants. Retrofitting shallowRef after building with reactive() touches every component and store.
- **Backend before Frontend** -- Building UI first means mocking commands. Building commands first means UI works against real data. ARCHITECTURE.md anti-pattern #2: never build FFmpeg commands in the frontend.
- **Progress parsing as dedicated module** -- PITFALLS.md P2 identifies this as the single most integration-critical component. It must be a tested Rust module (Phase 2) before any UI progress bar (Phase 3-4).
- **P2 features after validation** -- FEATURES.md identifies 6 P2 features for post-launch. Grouping them into Phase 5 keeps v1 scope tight.

### Research Flags

**Phases likely needing deeper research during planning:**

- **Phase 1 (Foundation):** FFmpeg LGPL build availability for all three platforms needs verification. The specific LGPL build source, codec support, and download URL stability must be confirmed. Sidecar naming convention behavior in production builds must be tested on a clean VM early.

- **Phase 2 (Backend Pipeline):** The 7 operation types each need validated FFmpeg filtergraph syntax against a real FFmpeg binary. Stderr progress parsing format must be validated against the specific FFmpeg version that will be downloaded. The `-progress pipe:1` protocol needs verification against the downloaded build's output format.

**Phases with standard patterns (skip dedicated research-phase):**

- **Phase 3 (Frontend Shell):** Vue 3 + Pinia + Naive UI follows well-documented patterns. The two-panel layout is simple. All component patterns have established solutions.

- **Phase 4 (Integration + Progress):** Tauri event wiring is standard. Command-Event Split is a documented best practice. The `useTauriEvent` composable established in Phase 1 eliminates listener leak risk.

- **Phase 5 (Polish + Ship):** Testing, linting, CI setup follow established conventions.

## Confidence Assessment

| Area         | Confidence | Notes                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------ | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Stack        | HIGH       | All technology decisions verified against Context7 official docs. Current versions confirmed via npm/crates.io. No speculative choices.                                                                                                                                                                                                                                                                                                                                                        |
| Features     | MEDIUM     | Table-stakes validated against competitive landscape. Competitor analysis partially based on training knowledge (HandBrake, Shutter Encoder -- not verified via official sources). Differentiators from PROJECT.md requirements. P1/P2/P3 prioritization is sound.                                                                                                                                                                                                                             |
| Architecture | HIGH       | Three-layer pattern, Command-Event Split, Thin Commands/Fat Services, Rust Owns Truth/Pinia Mirrors all verified against Tauri v2 official docs. Project structure and build order follow documented best practices.                                                                                                                                                                                                                                                                           |
| Pitfalls     | MEDIUM     | 10 critical pitfalls identified with prevention strategies, verification criteria, and phase mapping. Technical debt patterns, integration gotchas, and performance traps catalogued. Source quality varies: Tauri sidecar/event system is HIGH (Context7 official docs), FFmpeg progress protocol is HIGH (Context7 FFmpeg docs), Vue reactivity performance is MEDIUM (ecosystem knowledge), FFmpeg licensing is MEDIUM (build config is HIGH, legal interpretation requires lawyer review). |

**Overall confidence:** MEDIUM-HIGH (HIGH on stack and architecture, MEDIUM on features and pitfalls)

### Gaps to Address

- **FFmpeg filtergraph validation per operation type:** The 7 operation types each need working FFmpeg command strings validated against a real binary. **Handle:** Phase 2 planning includes a spike to validate each operation type's FFmpeg args with known test video. Document exact arguments and expected output.

- **Cross-platform FFmpeg LGPL binary availability:** ffmpeg-sidecar's `auto_download()` sources need verification for all target platforms. **Handle:** Validate during Phase 1. Test on macOS (Intel + Apple Silicon), Windows, Linux. Document exact build source and codec coverage.

- **Seed diversity and fingerprinting effectiveness:** Safety constraints are established but real-world fingerprinting system behavior is a black box. **Handle:** Focus measurable goal on "producing varied outputs" rather than specific anti-fingerprinting claims. Add disclaimer that platform algorithms change constantly.

- **Drag-and-drop cross-platform reliability:** Platform differences (macOS sandboxing, Windows UAC, Linux Wayland/X11) may cause inconsistent behavior. **Handle:** Include drag-and-drop testing in Phase 3. Keep file picker fallback equally prominent.

- **FFmpeg licensing legal review:** LGPL build identification is technically verified, but legal interpretation for proprietary desktop app distribution needs lawyer review if the app will be commercially distributed. **Handle:** Document LGPL strategy. Include license text in bundle. Seek legal review before commercial distribution.

## Sources

### Primary (HIGH confidence)

- [Context7: Tauri v2 docs](/websites/v2_tauri_app) - Sidecar bundling, IPC patterns, shell/dialog/fs plugins, state management, project scaffolding, event system, window close guard, async commands, resource paths
- [Context7: ffmpeg-sidecar](/nathanbabcock/ffmpeg-sidecar) - auto_download, FfmpegCommand, progress iteration, version detection
- [Context7: ffmpeg-next](/websites/rs_ffmpeg-next) - C binding approach (evaluated and rejected)
- [Context7: Naive UI](/tusen-ai/naive-ui) - Installation, dark theme, tree-shaking, component imports
- [Context7: Pinia](/vuejs/pinia) - Composition API stores, TypeScript integration, devtools
- [Context7: Vite](/vitejs/vite) - Vue + TypeScript template, plugin-vue setup, build config
- [Context7: Vitest](/vitest-dev/vitest) - Vue component testing, browser mode, DOM environments
- [Context7: FFmpeg docs](/websites/ffmpeg_ffmpeg-all) - Progress protocol (`-progress pipe:1`), error detection flags (`-xerror`, `-abort_on empty_output`), select/aselect filter, filtergraph chaining
- [PROJECT.md](/Users/ghost/Code/sandwich/.planning/PROJECT.md) - Requirements, scope, constraints
- [npm registry](https://www.npmjs.com/) - Latest versions for all JS dependencies
- [crates.io](https://crates.io/) - Latest versions for all Rust dependencies

### Secondary (MEDIUM confidence)

- [Vue 3 Reactivity docs](/vuejs/vue) - `shallowRef()`, `markRaw()`, `shallowReactive()` (specific large-list performance numbers from ecosystem knowledge)
- [FFmpeg Licensing](https://ffmpeg.org/legal.html) - LGPL vs GPL builds, `--enable-gpl` implications (build configuration is HIGH, legal interpretation requires lawyer review)
- Training knowledge: HandBrake, Shutter Encoder feature sets - Used for competitive positioning only

### Tertiary (LOW confidence)

- Training knowledge: Video fingerprinting techniques (six-class method from PROJECT.md context) - Safety constraint values from PROJECT.md, not independently verified
- Training knowledge: FFmpeg stderr progress parsing format - Validated against Tauri shell plugin API but not tested against specific downloaded FFmpeg version

---

_Research completed: 2026-05-12_
_Ready for roadmap: yes_
_Research files: STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md all complete_
