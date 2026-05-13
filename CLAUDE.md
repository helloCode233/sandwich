<!-- GSD:project-start source:PROJECT.md -->

## Project

**视频指纹批量修改工具**

一款基于 Tauri 的桌面端视频指纹批量修改工具。用户管理"种子"（自动生成的多操作链处理配方），拖入视频队列，选择种子后批量处理。处理通过 FFmpeg 执行，包括数学叠加、像素变换、时间轴修改、编码参数调整等操作，使同一素材生成多个指纹不同的视频。

**Core Value:** **一键批量去重** — 自动生成随机化种子配方，将同一视频源产出多个平台无法识别为重复的变体。

### Constraints

- **Tech stack**: Tauri 2.x + Vue 3 + Rust — 必须
- **Bundle size**: FFmpeg 二进制可能较大（~80MB），需考虑下载策略
- **Performance**: 视频处理为 CPU 密集型，必须异步执行避免阻塞 UI
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->

## Technology Stack

## Recommended Stack

### Core Technologies

| Technology | Version        | Purpose                                         | Why Recommended                                                                                                                                                              |
| ---------- | -------------- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tauri      | 2.11.x         | Desktop framework (Rust backend + web frontend) | Only production-ready framework combining Rust backend with web frontend for cross-platform desktop apps. v2 is mature with stable plugin system, sidecar bundling, and IPC. |
| Rust       | stable (1.85+) | Backend language                                | Required by Tauri. Memory-safe, zero-cost abstractions. FFmpeg process management, file I/O, seed generation all benefit from Rust's performance and safety.                 |
| Vue        | 3.5.x          | Frontend framework                              | User-specified. Composition API with `<script setup>` provides concise reactive UI code. Excellent TypeScript support.                                                       |
| Vite       | 8.0.x          | Build tool & dev server                         | Default for Vue 3 projects. Native ESM dev server for instant HMR. Rollup-based production builds produce small bundles for Tauri's webview.                                 |
| TypeScript | 6.0.x          | Type safety                                     | Catches bugs at compile time. Pinia, Naive UI, and Tauri APIs all have first-class TS support.                                                                               |

### Rust Crates

| Crate               | Version | Purpose                              | Why Recommended                                                                                                                                                                                                                                                                                                                                                                                          |
| ------------------- | ------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ffmpeg-sidecar      | 2.5.x   | FFmpeg binary management & execution | **Critical decision.** Provides `auto_download()` to fetch platform-specific FFmpeg binaries on first launch (solves FFMPEG-02 requirement). Iterator API with `filter_progress()` parses FFmpeg stderr to stream structured encoding progress to UI. Wraps `std::process::Command` internally but adds download, version detection, and output parsing. 1098 documented code snippets, HIGH reputation. |
| tauri-plugin-shell  | 2.x     | Execute external processes           | Required for `Command.sidecar()` and `Command.create()` APIs. Enables spawning FFmpeg as a child process from JS or Rust. Replaces Tauri v1's `tauri::api::process`.                                                                                                                                                                                                                                     |
| tauri-plugin-dialog | 2.x     | Native file/directory dialogs        | Open file picker for video import, save dialog for export directory selection. Filters for video file extensions.                                                                                                                                                                                                                                                                                        |
| tauri-plugin-fs     | 2.x     | File system access                   | Read/write video files, manage output directories, config file persistence.                                                                                                                                                                                                                                                                                                                              |
| serde + serde_json  | 1.x     | Serialization                        | Seed recipes serialized as JSON. Tauri command arguments/returns use serde.                                                                                                                                                                                                                                                                                                                              |
| rand                | 0.9.x   | Random seed generation               | Drives random parameter selection for seed recipes (operation types, frame ranges, filter params).                                                                                                                                                                                                                                                                                                       |
| anyhow              | 1.x     | Error handling                       | Ergonomic error propagation in Rust commands. Avoids boilerplate `Result<T, Box<dyn Error>>`.                                                                                                                                                                                                                                                                                                            |
| tokio               | 1.x     | Async runtime                        | Tauri commands can be async. Tokio provides `spawn_blocking` for CPU-bound FFmpeg work without blocking the main thread.                                                                                                                                                                                                                                                                                 |

### Vue 3 Ecosystem

| Library                   | Version | Purpose                 | Why Recommended                                                                                                                                                                                                                                                                                                                 |
| ------------------------- | ------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pinia                     | 3.0.x   | State management        | Official Vue store. Composition API setup stores align with `<script setup>` style. Type-safe. Devtools integration. Stores: seed list, video queue, batch processing state, FFmpeg status.                                                                                                                                     |
| Naive UI                  | 2.44.x  | Component library       | **Chosen over Element Plus.** Tree-shakeable (import only used components), built-in dark theme (critical for video tool UI), TypeScript-first, lighter bundle. Desktop-friendly compact theme. Components needed: NLayout, NMenu, NButton, NModal, NInput, NDataTable, NTag, NProgress, NTree, NCard, NSpace, NConfigProvider. |
| @tauri-apps/api           | 2.11.x  | Tauri frontend API      | `invoke()` for calling Rust commands, `event` system for progress streaming. Core Tauri JS bridge.                                                                                                                                                                                                                              |
| @tauri-apps/plugin-shell  | 2.3.x   | Shell command execution | `Command.create()` for spawning FFmpeg with arguments. Streams stdout/stderr for progress.                                                                                                                                                                                                                                      |
| @tauri-apps/plugin-dialog | 2.7.x   | Native file dialogs     | `open()` for video file selection with extension filters. `save()` for export directory.                                                                                                                                                                                                                                        |
| @tauri-apps/plugin-fs     | 2.5.x   | File system access      | Read video metadata, write processed files, manage config/seeds JSON.                                                                                                                                                                                                                                                           |
| @vitejs/plugin-vue        | 6.0.x   | Vue SFC compilation     | Required Vite plugin for `.vue` single-file component support.                                                                                                                                                                                                                                                                  |

### Development Tools

| Tool             | Version    | Purpose                     | Notes                                                                                                                |
| ---------------- | ---------- | --------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Vitest           | 4.1.x      | Frontend testing            | Vite-native test runner. Jest-compatible API. Uses happy-dom for DOM simulation. Faster than Jest for Vite projects. |
| @vue/test-utils  | 2.4.x      | Vue component mounting      | `mount()`, `shallowMount()` for isolated component tests. Works with Vitest.                                         |
| vue-tsc          | 3.x        | Vue type checking           | CLI type checker for `.vue` files. Run in CI or before build (`vue-tsc -b && vite build`).                           |
| cargo test       | (built-in) | Rust unit/integration tests | Standard Rust test harness. `#[cfg(test)]` modules for unit tests, `tests/` directory for integration tests.         |
| rstest           | 0.23+      | Parameterized Rust tests    | `#[rstest]` macro for table-driven tests. Useful for testing seed generation with multiple random seeds.             |
| ESLint           | 9.x        | JS/TS linting               | With `@typescript-eslint` and `eslint-plugin-vue` for `.vue` file linting.                                           |
| Prettier         | 3.x        | Code formatting             | Consistent formatting across Rust (`rustfmt`) and frontend code.                                                     |
| create-tauri-app | 4.6.x      | Project scaffolding         | `npm create tauri-app@latest` scaffolds Tauri + Vue + Vite + TypeScript in one command.                              |

## Installation

# Scaffold the project (run once)

# Frontend dependencies

# Tauri plugins (frontend JS bindings)

# Dev dependencies

# Rust dependencies (add to src-tauri/Cargo.toml)

# Tauri plugins (Rust side)

# Dev dependencies

## Alternatives Considered

| Recommended                       | Alternative                | When to Use Alternative                                                                                                                                                                                                                                       |
| --------------------------------- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ffmpeg-sidecar (binary wrapper)   | ffmpeg-next (C bindings)   | Use ffmpeg-next if you need programmatic frame-level access without CLI. For this project, we construct FFmpeg filter commands — CLI is the natural interface. ffmpeg-next requires FFmpeg dev libraries installed (painful for cross-platform distribution). |
| ffmpeg-sidecar auto_download      | Tauri sidecar bundling     | Use sidecar bundling if offline-first operation is critical and 80MB installer is acceptable. auto_download is better for MVP: small installer, download on first launch.                                                                                     |
| Naive UI                          | Element Plus               | Use Element Plus if you need more components out of the box (date pickers, complex tables) or prefer a Material Design aesthetic. Naive UI wins on bundle size, dark theme, and desktop density.                                                              |
| Pinia (Composition API stores)    | Pinia (Options API stores) | Options API if team prefers Vue 2-style stores. Composition API is more ergonomic with `<script setup>` and composables.                                                                                                                                      |
| No router (conditional rendering) | Vue Router                 | Add Vue Router if the app grows beyond two-panel layout to need distinct pages (settings, about, logs). For MVP, conditional `v-if` / dynamic `<component>` is simpler.                                                                                       |
| Vitest                            | Jest                       | Jest if you need legacy support or specific Jest plugins not ported to Vitest. Vitest is faster, Vite-native, and shares the same transform pipeline.                                                                                                         |

## What NOT to Use

| Avoid                              | Why                                                                                                                                                                                       | Use Instead             |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| ffmpeg-next (Rust FFmpeg bindings) | Requires system-installed FFmpeg dev libraries (libavcodec, libavformat, etc.). Massive compile times. Cross-platform distribution nightmare. Overkill for CLI-based filter construction. | ffmpeg-sidecar          |
| std::process::Command (raw)        | No auto-download. No structured progress parsing. Must manually parse FFmpeg stderr for progress info. Reinventing ffmpeg-sidecar's features.                                             | ffmpeg-sidecar          |
| Element Plus                       | ~30% larger bundle than Naive UI. No built-in compact theme for desktop density. Dark theme support is bolted-on, not native.                                                             | Naive UI                |
| Vuex                               | Deprecated in favor of Pinia. Official Vue recommendation for new projects. No Composition API support.                                                                                   | Pinia                   |
| Vue 2 / Options API                | Vue 2 is EOL (Dec 2023). Options API still works in Vue 3 but `<script setup>` + Composition API is the modern standard with better TypeScript inference.                                 | Vue 3 + Composition API |
| Tauri v1                           | EOL. Plugin system in v2 is unified. v1 APIs like `tauri::api::dialog` are removed in v2.                                                                                                 | Tauri 2.11.x            |
| Electron                           | 3x larger bundle (~150MB+). Slower startup. Heavier memory footprint. No Rust backend (Node.js only).                                                                                     | Tauri                   |
| Webpack / vue-cli                  | Deprecated Vue tooling. Slower builds, worse DX. Vite is the official recommendation.                                                                                                     | Vite                    |

## FFmpeg Distribution Strategy

- PROJECT.md specifies "FFMPEG-02: FFmpeg 缺失时自动下载" — auto-download is a hard requirement
- ffmpeg-sidecar handles platform detection, download, extraction, and verification
- Binaries cached in app data directory — persists across app restarts
- No Tauri sidecar bundling (avoids 80MB in installer, meets "Bundle size" constraint)
- `KEEP_ONLY_FFMPEG=1` env var skips ffplay/ffprobe download (save ~30MB)

## IPC Patterns

### Pattern 1: Request/Response (Rust Command)

#[tauri::command]

### Pattern 2: Event Streaming (Progress)

### Pattern 3: Shell Command (FFmpeg Execution)

## Stack Patterns by Variant

- Spawn FFmpeg in a separate thread via `tokio::task::spawn_blocking`
- Emit progress events from Rust to Vue via Tauri event system
- Because: avoids blocking the Tauri main thread and keeps UI responsive
- Stream FFmpeg output directly to disk (no in-memory buffering)
- Use FFmpeg's `-progress` flag for structured progress output
- Because: prevents memory exhaustion from buffering video frames
- Switch to Tauri sidecar bundling: add FFmpeg binary to `src-tauri/binaries/`
- Register in `tauri.conf.json` under `bundle.externalBin`
- Because: bundles FFmpeg with installer, no download needed

## Version Compatibility

| Package                  | Compatible With                  | Notes                                                             |
| ------------------------ | -------------------------------- | ----------------------------------------------------------------- |
| tauri 2.11.x             | @tauri-apps/api 2.11.x           | Must match minor versions. Plugins follow independent versioning. |
| vue 3.5.x                | vite 8.x, @vitejs/plugin-vue 6.x | Vue 3.5 is the stable line. Vite 7+ dropped CJS; all ESM.         |
| pinia 3.x                | vue 3.5.x                        | Pinia 3 is the current major for Vue 3.                           |
| naive-ui 2.44.x          | vue 3.5.x                        | Actively maintained. All components support latest Vue 3.         |
| vitest 4.x               | vite 8.x                         | Vitest 4 requires Vite >= 7.                                      |
| ffmpeg-sidecar 2.5.x     | tauri 2.x                        | Standalone — doesn't link to Tauri. Binary management only.       |
| tauri-plugin-shell 2.3.x | tauri 2.11.x                     | Tauri 2 plugin — uses v2 plugin API.                              |

## Sources

- [Context7: Tauri v2 docs](/websites/v2_tauri_app) — sidecar bundling, IPC, shell plugin, dialog plugin, project scaffolding. HIGH confidence.
- [Context7: ffmpeg-sidecar](/nathanbabcock/ffmpeg-sidecar) — auto_download, FfmpegCommand, progress iteration, version detection. HIGH confidence.
- [Context7: ffmpeg-next](/websites/rs_ffmpeg-next) — C binding approach, complexity assessment, transcode examples. HIGH confidence.
- [Context7: Naive UI](/tusen-ai/naive-ui) — installation, dark theme, tree-shaking, component imports. HIGH confidence.
- [Context7: Pinia](/vuejs/pinia) — Composition API stores, TypeScript integration, devtools. HIGH confidence.
- [Context7: Vite](/vitejs/vite) — Vue + TypeScript template, plugin-vue setup, build config. HIGH confidence.
- [Context7: Vitest](/vitest-dev/vitest) — Vue component testing, browser mode, jsdom/happy-dom environments. HIGH confidence.
- [npm registry](https://www.npmjs.com/) — latest versions: vue@3.5.34, pinia@3.0.4, vite@8.0.12, naive-ui@2.44.1, vitest@4.1.6, @tauri-apps/api@2.11.0. HIGH confidence.
- [crates.io](https://crates.io/) — latest versions: tauri@2.11.1, ffmpeg-sidecar@2.5.1, ffmpeg-next@8.1.0. HIGH confidence.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->

## Conventions

Conventions not yet established. Will populate as patterns emerge during development.

<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->

## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.

<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->

## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.

<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->

## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:

- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.

<!-- GSD:workflow-end -->

<!-- GSD:profile-start -->

## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.

<!-- GSD:profile-end -->
