# Phase 01: Foundation - Research

**Researched:** 2026-05-13
**Domain:** Tauri 2.x project scaffolding + FFmpeg lifecycle (detection, download, verification)
**Confidence:** HIGH

## Summary

Phase 01 establishes the two non-negotiable prerequisites for the entire project: (1) a Tauri 2.x + Vue 3 + Vite project scaffold that builds and runs, and (2) FFmpeg reliably available on the user's machine with zero-config detection and one-click download.

The scaffold is straightforward: `bun create tauri-app` with Vue + TypeScript template, then manually pin all versions to match CLAUDE.md requirements. Tauri v2 docs explicitly support `bun create tauri-app` and `bun tauri dev`. The scaffolded project uses Vite 8.x for the frontend dev server and Tauri 2.11.x for the desktop shell.

FFmpeg availability is the more nuanced piece. The user wants GitHub Releases as the primary download source (D-21), but `ffmpeg-sidecar` v2.5.1 -- the recommended FFmpeg binary management crate -- bakes third-party URLs at compile time (evermeet.cx for macOS, gyan.dev for Windows, johnvansickle.com for Linux). Critically, BtbN/FFmpeg-Builds (the dominant GitHub Releases source) provides NO macOS builds. This means: for Linux/Windows, GitHub Releases can be the primary source; for macOS, ffmpeg-sidecar's built-in URLs are the established and reliable option. The crate's `download_ffmpeg_package(url, dir)` function accepts custom URLs, enabling a multi-source strategy with mirror fallback.

**Primary recommendation:** Use `bun create tauri-app` for scaffolding with post-generation version pinning. For FFmpeg download, implement a custom download function using `reqwest` streaming (for progress) with GitHub Releases as primary on Linux/Windows, ffmpeg-sidecar's built-in URLs as primary on macOS, and a CN-accessible mirror (jsDelivr CDN) as fallback. Use `unpack_ffmpeg()` from ffmpeg-sidecar for archive extraction. Store FFmpeg path/version/download-time in `tauri-plugin-store` for persistence across restarts.

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** `create-tauri-app` one-click scaffold, manual version pinning post-generation
- **D-02:** All dependencies exact-version pinned (no caret ranges)
- **D-03:** TypeScript strict: true, all strict checks enabled
- **D-04:** Tauri identifier: `com.sandwich.app`
- **D-05:** Package manager: bun
- **D-06:** Atomic CSS: UnoCSS (Vite native integration, zero runtime)
- **D-07:** Git hooks: husky + lint-staged configured in Phase 1
- **D-08:** Rust edition: 2024
- **D-09:** Dev command: `tauri dev` (starts Vite + Tauri simultaneously)
- **D-10:** CI: GitHub Actions (vue-tsc + cargo check + ESLint + clippy + Vitest + cargo test)
- **D-11:** Frontend directory structure: layered (src/components/, src/stores/, src/composables/, src/types/, src/utils/)
- **D-12:** Window: title 'Sandwich', 1200x800 default, resizable, min 900x600
- **D-13:** Internationalization: vue-i18n v11, Chinese + English bilingual
- **D-14:** No Vue Router (Phase 1 is single page)
- **D-15:** Startup detection + user-triggered download (not automatic)
- **D-16:** Download scope: FFmpeg + FFprobe (Phase 2 metadata extraction needs ffprobe)
- **D-17:** Full-screen download page: percentage + downloaded/total size + download speed
- **D-18:** User can choose FFmpeg storage directory (not forced default)
- **D-19:** Detection strategy: PATH first -> prompt download otherwise
- **D-20:** Download failure: show specific error + retry button (3 retries then manual download instructions)
- **D-21:** Download sources: GitHub Releases default + mirror fallback (CN-user friendly)
- **D-22:** Minimum FFmpeg version >= 4.0; below threshold prompts download
- **D-23:** Post-download: auto `ffmpeg -version` verification -> success auto-enters main interface
- **D-24:** Path persistence: tauri-plugin-store (ffmpeg_path, version, download_time)
- **D-25:** Every startup checks GitHub latest release; if newer, prompts optional update (non-blocking)
- **D-26:** Resume interrupted download on next startup
- **D-27:** Download cancelable (cleans temp files, returns to initial state)
- **D-28:** macOS: auto-remove quarantine attribute post-extraction (`xattr -dr com.apple.quarantine`)
- **D-29:** Progress display: percentage + downloaded/total size + speed
- **D-30:** Minimal FFmpeg page -- serves only FFmpeg detection/download, not full UI foundation for Phase 3
- **D-31:** Page elements: status indicator + action button (centered card layout)
- **D-32:** Naive UI dark theme enabled in Phase 1 (NConfigProvider + darkTheme)
- **D-33:** Frontend infrastructure: Naive UI + UnoCSS + vue-i18n + Pinia all installed Phase 1
- **D-34:** Page state flow: detecting -> check result (found/not found) -> downloading (progress+cancel+speed) -> complete (auto-transition)
- **D-35:** After FFmpeg ready: show placeholder homepage (logo + version + "awaiting future features")
- **D-36:** ESLint 9 (flat config) + @typescript-eslint + eslint-plugin-vue + Prettier, all Phase 1
- **D-37:** Vitest + @vue/test-utils + cargo test, test infrastructure Phase 1
- **D-38:** rustfmt + clippy, Phase 1 enabled (CI: `cargo fmt --check` + `cargo clippy -- -D warnings`)
- **D-39:** vue-tsc type checking, Phase 1 configured

### Claude's Discretion

None -- all decisions were explicitly made by user.

### Deferred Ideas (OUT OF SCOPE)

None -- discussion stayed entirely within phase scope.

## Phase Requirements

| ID        | Description                                                                                   | Research Support                                                                                                                                                                                                                                                                                  |
| --------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| FFMPEG-01 | App startup auto-detects FFmpeg in PATH                                                       | ffmpeg-sidecar `ffmpeg_is_installed()` + `ffmpeg_version()` -- checks PATH and common install locations. Returns bool + semver string. Called in Tauri setup hook before window creation.                                                                                                         |
| FFMPEG-02 | When missing, one-click download with progress display and platform-adaptive binary selection | Custom download via `reqwest` streaming + ffmpeg-sidecar `unpack_ffmpeg()`. URLs selected at runtime via platform detection. Progress streamed to frontend via Tauri events. GitHub Releases (BtbN) for Linux/Windows; evermeet.cx/osxexperts.net for macOS. jsDelivr CDN mirror for CN fallback. |
| FFMPEG-03 | Post-download auto-verify FFmpeg is executable                                                | `ffmpeg_version_with_path()` verifies the downloaded binary. On macOS, `xattr -dr com.apple.quarantine` runs first to remove Gatekeeper isolation. Verified path + version persisted to tauri-plugin-store.                                                                                       |

## Architectural Responsibility Map

| Capability                        | Primary Tier           | Secondary Tier         | Rationale                                                               |
| --------------------------------- | ---------------------- | ---------------------- | ----------------------------------------------------------------------- |
| FFmpeg PATH detection             | API / Backend (Rust)   | --                     | `ffmpeg_sidecar::command::ffmpeg_is_installed()` runs in Rust process   |
| FFmpeg version reading            | API / Backend (Rust)   | --                     | Executes `ffmpeg -version`, parses output in Rust                       |
| Download URL selection (platform) | API / Backend (Rust)   | --                     | `cfg!` macros + runtime platform detection determine correct binary URL |
| HTTP download with progress       | API / Backend (Rust)   | --                     | `reqwest` streaming download, progress emitted via Tauri events         |
| Archive extraction                | API / Backend (Rust)   | --                     | ffmpeg-sidecar `unpack_ffmpeg()` handles tar/zip extraction             |
| Post-download verification        | API / Backend (Rust)   | --                     | Runs `ffmpeg -version` on downloaded binary                             |
| macOS quarantine removal          | API / Backend (Rust)   | --                     | `std::process::Command` runs `xattr -dr com.apple.quarantine`           |
| Download progress UI              | Browser / Client (Vue) | --                     | Listens to Tauri events, updates Pinia store, renders NProgress + stats |
| FFmpeg status UI                  | Browser / Client (Vue) | --                     | Vue component reads Pinia store, renders status card                    |
| FFmpeg path persistence           | API / Backend (Rust)   | --                     | tauri-plugin-store writes to app_data_dir                               |
| Directory selection dialog        | Browser / Client (Vue) | --                     | @tauri-apps/plugin-dialog `open()` triggers native directory picker     |
| User-selected download path       | API / Backend (Rust)   | Browser / Client (Vue) | Frontend passes path from dialog to Rust download command               |
| Dark theme                        | Browser / Client (Vue) | --                     | Naive UI NConfigProvider + darkTheme import                             |
| i18n (zh/en)                      | Browser / Client (Vue) | --                     | vue-i18n `createI18n()` with locale message files                       |
| Project build/dev                 | Frontend Server (Vite) | --                     | Vite dev server + Tauri Rust compilation                                |
| Tauri window config               | API / Backend (Rust)   | --                     | tauri.conf.json `app.windows[]`                                         |
| CI checks                         | CI (GitHub Actions)    | --                     | ubuntu-latest runner with bun + Rust toolchains                         |

## Standard Stack

### Core

| Library                           | Version | Purpose                                    | Why Standard                                                                                                                                                      |
| --------------------------------- | ------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| tauri (crate)                     | 2.11.1  | Desktop framework (Rust backend + webview) | Only production-ready Rust+web hybrid desktop framework. v2 stable with unified plugin system.                                                                    |
| @tauri-apps/cli                   | 2.11.1  | Tauri CLI (`tauri dev`, `tauri build`)     | Must match tauri crate minor version. Provides dev server orchestration.                                                                                          |
| @tauri-apps/api                   | 2.11.0  | Frontend Tauri API (`invoke()`, events)    | Core JS bridge to Rust commands. Version must match tauri crate minor.                                                                                            |
| vue                               | 3.5.34  | Frontend framework                         | User-specified. Composition API with `<script setup>`.                                                                                                            |
| vite                              | 8.0.12  | Build tool & dev server                    | Default for Vue 3. Native ESM HMR. Required by Tauri v2 frontend.                                                                                                 |
| @vitejs/plugin-vue                | 6.0.6   | Vue SFC compilation in Vite                | Official Vue plugin for Vite. Required for `.vue` file support.                                                                                                   |
| typescript                        | 6.0.3   | Type safety                                | User-specified strict mode. All frontend code typed.                                                                                                              |
| ffmpeg-sidecar (crate)            | 2.5.1   | FFmpeg binary management                   | Detection (`ffmpeg_is_installed`), download helpers, archive extraction. Provides `unpack_ffmpeg()` for cross-platform tar/zip handling.                          |
| Naive UI                          | 2.44.1  | Vue 3 component library                    | User-chosen over Element Plus. Tree-shakeable, built-in dark theme, TypeScript-first, compact desktop density.                                                    |
| Pinia                             | 3.0.4   | State management                           | Official Vue store. Composition API setup stores align with `<script setup>`.                                                                                     |
| vue-i18n                          | 11.4.2  | Internationalization                       | User-required bilingual (zh-CN + en). Composition API mode.                                                                                                       |
| UnoCSS                            | 66.6.8  | Atomic CSS                                 | User-chosen. Vite-native, zero runtime, tree-shakeable.                                                                                                           |
| tauri-plugin-store (crate + npm)  | 2.4.3   | Persistent key-value storage               | FFmpeg path/version/download_time persistence. Auto-save with debounce.                                                                                           |
| tauri-plugin-shell (crate + npm)  | 2.3.5   | Shell command execution                    | Required for `Command.sidecar()` and `Command.create()` -- though ffmpeg-sidecar handles process spawning, shell plugin enables xattr and other shell operations. |
| tauri-plugin-dialog (crate + npm) | 2.7.1   | Native file/directory dialogs              | Directory picker for user-selected FFmpeg storage location (D-18).                                                                                                |
| tauri-plugin-fs (crate + npm)     | 2.5.1   | File system access                         | Read/write config files, manage temp download files.                                                                                                              |

### Supporting

| Library                         | Version | Purpose            | When to Use                                                                             |
| ------------------------------- | ------- | ------------------ | --------------------------------------------------------------------------------------- |
| tokio (crate)                   | 1.52.3  | Async runtime      | `spawn_blocking` for CPU-bound FFmpeg work. Async Tauri commands.                       |
| serde + serde_json (crate)      | 1.0.149 | Serialization      | Tauri command arguments/returns. Store values as JSON.                                  |
| anyhow (crate)                  | 1.0.102 | Error handling     | Ergonomic `Result<T>` in Rust commands.                                                 |
| rand (crate)                    | 0.10.1  | Random generation  | Not directly used in Phase 1, but included in Cargo.toml for future phases.             |
| reqwest (crate)                 | 0.12.x  | HTTP client        | Custom FFmpeg download with streaming progress. ffmpeg-sidecar uses reqwest internally. |
| @tauri-apps/plugin-store (npm)  | 2.4.3   | Store JS bindings  | `load()`, `set()`, `get()` from frontend for reading persisted FFmpeg info.             |
| @tauri-apps/plugin-dialog (npm) | 2.7.1   | Dialog JS bindings | `open()` with directory option for user path selection.                                 |
| @tauri-apps/plugin-shell (npm)  | 2.3.5   | Shell JS bindings  | `Command.create()` for shell operations.                                                |

### Dev Dependencies

| Library           | Version | Purpose                       | Why Standard                                                                                |
| ----------------- | ------- | ----------------------------- | ------------------------------------------------------------------------------------------- |
| eslint            | 9.39.4  | JS/TS linting                 | Latest 9.x (user specifies ESLint 9 flat config). Flat config format (`eslint.config.mjs`). |
| typescript-eslint | 8.59.3  | TypeScript ESLint integration | `tseslint.config()` helper for flat config. Provides TS-aware lint rules.                   |
| eslint-plugin-vue | 10.9.1  | Vue SFC linting               | v10 required for ESLint 9 flat config compatibility. `flat/recommended` config.             |
| vue-eslint-parser | 10.4.0  | Vue SFC parser for ESLint     | Required by eslint-plugin-vue. Delegates `<script lang="ts">` to typescript-eslint parser.  |
| @eslint/js        | 9.39.4  | ESLint core JS rules          | `js.configs.recommended` in flat config.                                                    |
| prettier          | 3.8.3   | Code formatting               | Consistent formatting. Separate from ESLint (no eslint-plugin-prettier).                    |
| vitest            | 4.1.6   | Frontend testing              | Vite-native. Jest-compatible API. happy-dom for DOM simulation.                             |
| @vue/test-utils   | 2.4.10  | Vue component mounting        | `mount()` for isolated component tests.                                                     |
| vue-tsc           | 3.2.8   | Vue type checking             | CLI type checker for `.vue` files. Run as `vue-tsc -b`.                                     |
| husky             | 9.1.7   | Git hooks                     | Pre-commit hook for lint-staged. Configured per D-07.                                       |
| lint-staged       | 17.0.4  | Staged file linting           | Runs ESLint + Prettier on staged files before commit.                                       |
| rstest (crate)    | 0.26.1  | Parameterized Rust tests      | `#[rstest]` macro for table-driven tests.                                                   |

### Rust Dev Tooling

| Tool    | Version   | Purpose                                          |
| ------- | --------- | ------------------------------------------------ |
| rustfmt | (bundled) | Rust code formatting. CI: `cargo fmt --check`.   |
| clippy  | (bundled) | Rust linting. CI: `cargo clippy -- -D warnings`. |

### Alternatives Considered

| Instead of                       | Could Use                                | Tradeoff                                                                                                                            |
| -------------------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| ffmpeg-sidecar default URLs      | GitHub Releases (BtbN) for Linux/Windows | GitHub Releases has higher availability confidence. But BtbN provides NO macOS builds -- macOS must use evermeet.cx/osxexperts.net. |
| ffmpeg-sidecar `auto_download()` | Custom reqwest download with progress    | `auto_download()` handles everything but uses baked-in URLs. Custom reqwest gives us URL control + progress streaming to UI.        |
| Naive UI                         | Element Plus                             | Naive UI wins on bundle size, dark theme native support, desktop density. User explicitly chose Naive UI.                           |
| Pinia (Composition API)          | Pinia (Options API)                      | Composition API stores align with `<script setup>` and composables.                                                                 |
| No router (D-14)                 | Vue Router                               | Phase 1 is single-page; no routes needed. Router added in later phases if needed.                                                   |
| Vitest                           | Jest                                     | Vitest is Vite-native, faster, shares transform pipeline. User chose Vitest.                                                        |
| Prettier integrated with ESLint  | Separate Prettier + ESLint               | Separate is current best practice (avoids plugin compatibility issues). ESLint 9 flat config makes this cleaner.                    |

**Installation:**

```bash
# Scaffold (once)
bun create tauri-app -- --template vue-ts sandwich
cd sandwich

# Frontend dependencies
bun add vue@3.5.34 @tauri-apps/api@2.11.0 naive-ui@2.44.1 pinia@3.0.4 vue-i18n@11.4.2
bun add -D unocss@66.6.8 @unocss/vite@66.6.8

# Tauri plugins (frontend JS bindings)
bun add @tauri-apps/plugin-store@2.4.3 @tauri-apps/plugin-shell@2.3.5 @tauri-apps/plugin-dialog@2.7.1 @tauri-apps/plugin-fs@2.5.1

# Dev dependencies
bun add -D eslint@9.39.4 typescript-eslint@8.59.3 eslint-plugin-vue@10.9.1 vue-eslint-parser@10.4.0 @eslint/js@9.39.4
bun add -D prettier@3.8.3 vitest@4.1.6 @vue/test-utils@2.4.10 vue-tsc@3.2.8
bun add -D husky@9.1.7 lint-staged@17.0.4
bun add -D @tauri-apps/cli@2.11.1

# Rust dependencies (add to src-tauri/Cargo.toml)
cargo add tauri@2.11.1
cargo add ffmpeg-sidecar@2.5.1 --features download_ffmpeg
cargo add tokio@1.52.3 --features full
cargo add serde@1.0.149 serde_json@1.0.149
cargo add anyhow@1.0.102
cargo add reqwest@0.12 --features stream
cargo add tauri-plugin-store@2.4.3
cargo add tauri-plugin-shell@2.3.5
cargo add tauri-plugin-dialog@2.7.1
cargo add tauri-plugin-fs@2.5.1
cargo add rstest@0.26.1 --dev
```

**Version verification:** All versions confirmed against npm registry and crates.io as of 2026-05-13. [VERIFIED: npm registry] [VERIFIED: crates.io]

## Architecture Patterns

### System Architecture: FFmpeg Detection and Download Flow

```
User Launches App
       │
       ▼
  ┌──────────────────────────────────────────────────────────────┐
  │  Tauri Setup Hook (Rust)                                     │
  │  1. Check tauri-plugin-store for cached ffmpeg_path          │
  │  2. If cached path exists → verify binary → emit status      │
  │  3. If no cache → ffmpeg_is_installed() → ffmpeg_version()   │
  │  4. Check version >= 4.0 (D-22)                              │
  │  5. Check GitHub latest release (D-25, non-blocking)         │
  │  6. Emit initial status event to frontend                     │
  └──────────────────────────┬───────────────────────────────────┘
                             │
                             ▼
  ┌──────────────────────────────────────────────────────────────┐
  │  Frontend (Vue) — FFmpegStatus Page                          │
  │  ┌──────────────────────────────────────────────────┐        │
  │  │  Pinia: useFfmpegStore                           │        │
  │  │  - status: 'detecting'|'found'|'missing'|...    │        │
  │  │  - version: string|null                          │        │
  │  │  - downloadProgress: {pct, downloaded, total,    │        │
  │  │    speed}                                        │        │
  │  └──────────────────────────────────────────────────┘        │
  │                                                              │
  │  State Machine (D-34):                                       │
  │  ┌──────────┐   found    ┌──────────┐                       │
  │  │detecting │───────────►│  found   │──► auto-transition    │
  │  └────┬─────┘            └──────────┘    to placeholder pg  │
  │       │ not found                                            │
  │       ▼                                                      │
  │  ┌──────────┐   user click   ┌──────────┐                   │
  │  │ missing  │───────────────►│ select   │                   │
  │  └──────────┘                │  dir     │                   │
  │                              └────┬─────┘                   │
  │                                   │ confirm                  │
  │                                   ▼                          │
  │  ┌────────────────────────────────────────────┐              │
  │  │ downloading (full-screen D-17)              │              │
  │  │  - NProgress: percentage                    │              │
  │  │  - Downloaded/Total size                    │              │
  │  │  - Download speed                           │              │
  │  │  - Cancel button (D-27)                     │              │
  │  │  - Retry button (D-20, after failure)       │              │
  │  └──────────────────┬─────────────────────────┘              │
  │                     │ success                                 │
  │                     ▼                                         │
  │  ┌──────────────────────────┐                                │
  │  │ verifying → verified ✓   │──► auto-transition             │
  │  └──────────────────────────┘    to placeholder pg           │
  └──────────────────────────────────────────────────────────────┘

  Download Architecture (Rust Backend):
  ┌──────────────────────────────────────────────────────────────┐
  │  tauri::command("start_download")                            │
  │  1. Receive target_dir from frontend (D-18)                  │
  │  2. Select download URL:                                     │
  │     - Linux/Windows: BtbN GitHub Releases (primary)          │
  │     - macOS aarch64: osxexperts.net (primary)                │
  │     - macOS x86_64: evermeet.cx (primary)                    │
  │     - Fallback: jsDelivr CDN mirror (CN-friendly)            │
  │     - Last resort: ffmpeg-sidecar ffmpeg_download_url()      │
  │  3. reqwest streaming GET with progress callbacks            │
  │  4. Emit 'ffmpeg-download-progress' events to frontend       │
  │  5. Save archive to temp dir                                 │
  │  6. unpack_ffmpeg(archive, target_dir)                       │
  │  7. macOS: run xattr -dr com.apple.quarantine target_dir/*   │
  │  8. ffmpeg_version_with_path(target_dir) → verify            │
  │  9. Persist to store: {ffmpeg_path, version, download_time}  │
  │  10. Emit 'ffmpeg-ready' event                               │
  └──────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
sandwich/
├── index.html                    # Tauri entry HTML
├── package.json                  # Exact-version pinned (D-02)
├── eslint.config.mjs             # ESLint 9 flat config (D-36)
├── prettier.config.mjs           # Prettier config
├── uno.config.ts                 # UnoCSS presets + theme
├── tsconfig.json                 # TypeScript strict config (D-03)
├── vite.config.ts                # Vite + Vue + UnoCSS plugins
├── src/
│   ├── main.ts                   # Vue app entry + plugins
│   ├── App.vue                   # Root: NConfigProvider + i18n
│   ├── components/
│   │   ├── FfmpegStatus.vue      # Status indicator card
│   │   ├── FfmpegDownload.vue    # Full-screen download page
│   │   └── PlaceholderHome.vue   # Post-FFmpeg placeholder (D-35)
│   ├── stores/
│   │   └── ffmpeg.ts             # Pinia: useFfmpegStore
│   ├── composables/
│   │   └── useFfmpeg.ts          # FFmpeg detection/invoke logic
│   ├── types/
│   │   └── ffmpeg.ts             # FFmpegStatus, DownloadProgress types
│   ├── utils/
│   │   └── i18n.ts               # vue-i18n setup + locale imports
│   └── locales/
│       ├── zh-CN.json            # Chinese translations
│       └── en.json               # English translations
├── src-tauri/
│   ├── Cargo.toml                # Rust dependencies (edition 2024)
│   ├── build.rs
│   ├── tauri.conf.json           # Window config, identifier, plugins
│   ├── capabilities/
│   │   └── default.json          # Permissions for store, shell, dialog, fs
│   └── src/
│       ├── main.rs               # Tauri entry
│       ├── lib.rs                # Plugin registration + setup
│       └── commands/
│           └── ffmpeg.rs         # detect_ffmpeg, start_download, cancel, verify
└── tests/
    └── ffmpeg.rs                 # Rust integration tests (rstest)
```

### Pattern 1: Tauri IPC Request/Response (Detection)

**What:** Synchronous request from frontend to Rust backend via `invoke()`. Frontend calls, Rust processes, returns value.
**When to use:** FFmpeg detection, version retrieval, store operations.
**Example:**

```typescript
// Frontend (src/composables/useFfmpeg.ts)
// Source: @tauri-apps/api invoke() [VERIFIED: Tauri v2 docs]
import { invoke } from '@tauri-apps/api/core';

export interface FfmpegInfo {
  found: boolean;
  path: string | null;
  version: string | null;
}

export async function detectFfmpeg(): Promise<FfmpegInfo> {
  return invoke<FfmpegInfo>('detect_ffmpeg');
}
```

```rust
// Backend (src-tauri/src/commands/ffmpeg.rs)
// Source: ffmpeg-sidecar docs [VERIFIED: Context7 /nathanbabcock/ffmpeg-sidecar]
use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::version::ffmpeg_version;

#[tauri::command]
async fn detect_ffmpeg() -> Result<FfmpegInfo, String> {
    if ffmpeg_is_installed() {
        let version = ffmpeg_version().map_err(|e| e.to_string())?;
        Ok(FfmpegInfo {
            found: true,
            path: Some(std::env::var("PATH").unwrap_or_default()),
            version: Some(version),
        })
    } else {
        Ok(FfmpegInfo {
            found: false,
            path: None,
            version: None,
        })
    }
}
```

### Pattern 2: Tauri Event Streaming (Download Progress)

**What:** Rust emits events to the frontend during long-running operations. Frontend listens and updates UI reactively.
**When to use:** FFmpeg download progress, extraction progress.
**Example:**

```rust
// Backend — emit progress from download function
// Source: Tauri v2 event system [VERIFIED: Context7 /websites/v2_tauri_app]
use tauri::Emitter;

#[tauri::command]
async fn start_download(
    app: tauri::AppHandle,
    target_dir: String,
) -> Result<(), String> {
    // ... download logic with reqwest streaming ...
    let _ = app.emit("ffmpeg-download-progress", DownloadProgress {
        percent: (downloaded as f64 / total as f64 * 100.0),
        downloaded_bytes: downloaded,
        total_bytes: total,
        speed_bytes_per_sec: speed,
    });
    // ...
}
```

```typescript
// Frontend — listen for events
// Source: @tauri-apps/api event [VERIFIED: Tauri v2 docs]
import { listen } from '@tauri-apps/api/event';
import type { DownloadProgress } from '@/types/ffmpeg';

export function subscribeDownloadProgress(
  callback: (progress: DownloadProgress) => void,
): Promise<() => void> {
  return listen<DownloadProgress>('ffmpeg-download-progress', (event) => {
    callback(event.payload);
  });
}
```

### Pattern 3: Pinia Composition API Store (FFmpeg State)

**What:** Reactive state managed in a Pinia setup store, consumed by Vue components.
**When to use:** FFmpeg status, download progress, version info.
**Example:**

```typescript
// src/stores/ffmpeg.ts
// Source: Pinia Composition API [VERIFIED: Context7 /vuejs/pinia]
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export type FfmpegStatus =
  | 'detecting'
  | 'found'
  | 'missing'
  | 'outdated'
  | 'downloading'
  | 'verifying'
  | 'verified'
  | 'error';

export const useFfmpegStore = defineStore('ffmpeg', () => {
  const status = ref<FfmpegStatus>('detecting');
  const version = ref<string | null>(null);
  const path = ref<string | null>(null);
  const downloadProgress = ref({
    percent: 0,
    downloadedBytes: 0,
    totalBytes: 0,
    speedBytesPerSec: 0,
  });
  const downloadError = ref<string | null>(null);
  const retryCount = ref(0);

  const isReady = computed(() => status.value === 'found' || status.value === 'verified');
  const needsDownload = computed(() => status.value === 'missing' || status.value === 'outdated');

  function setDownloadProgress(p: {
    percent: number;
    downloadedBytes: number;
    totalBytes: number;
    speedBytesPerSec: number;
  }) {
    downloadProgress.value = p;
  }

  return {
    status,
    version,
    path,
    downloadProgress,
    downloadError,
    retryCount,
    isReady,
    needsDownload,
    setDownloadProgress,
  };
});
```

### Pattern 4: Naive UI Dark Theme + i18n Root Wrapper

**What:** Root `App.vue` wraps content in `NConfigProvider` with dark theme and locale.
**When to use:** D-32, D-33 — dark theme from Phase 1, Naive UI locale for component text.
**Example:**

```html
<!-- src/App.vue -->
<!-- Source: Naive UI docs [VERIFIED: Context7 /tusen-ai/naive-ui] -->
<template>
  <n-config-provider :theme="darkTheme" :locale="naiveLocale">
    <n-global-style />
    <FfmpegStatus v-if="!ffmpegStore.isReady" />
    <PlaceholderHome v-else />
  </n-config-provider>
</template>

<script setup lang="ts">
  import { darkTheme, zhCN, enUS, dateZhCN, dateEnUS } from 'naive-ui';
  import { useFfmpegStore } from '@/stores/ffmpeg';
  import { useI18n } from 'vue-i18n';
  import { computed } from 'vue';

  const ffmpegStore = useFfmpegStore();
  const { locale } = useI18n();
  const naiveLocale = computed(() => (locale.value === 'zh-CN' ? zhCN : enUS));
</script>
```

### Pattern 5: ESLint 9 Flat Config (Vue + TypeScript)

**What:** Single `eslint.config.mjs` using `tseslint.config()` with Vue plugin.
**When to use:** D-36 — ESLint 9 flat config for all linting.
**Example:**

```js
// eslint.config.mjs
// Source: typescript-eslint docs + eslint-plugin-vue v10 [CITED: typescript-eslint.io/getting-started]
import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';
import vueParser from 'vue-eslint-parser';

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...pluginVue.configs['flat/recommended'],
  {
    files: ['**/*.vue'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: ['.vue'],
        sourceType: 'module',
      },
    },
  },
  {
    ignores: ['src-tauri/**', 'dist/**', 'node_modules/**'],
  },
);
```

### Anti-Patterns to Avoid

- **Using `auto_download()` directly:** Uses baked-in third-party URLs with no progress. Contradicts D-17 (progress display) and D-21 (GitHub Releases). Use custom reqwest download + `unpack_ffmpeg()` instead.
- **Setting `KEEP_ONLY_FFMPEG=1`:** Skips ffprobe download. Contradicts D-16 (Phase 2 needs ffprobe for metadata extraction).
- **Blocking the Tauri main thread during download:** Must use `tokio::task::spawn_blocking` or async `reqwest` for network I/O. Blocking freezes the webview.
- **Storing FFmpeg in Tauri's sidecar directory:** ffmpeg-sidecar places binaries next to the Rust executable by default, but the user may choose a custom directory (D-18). Don't force the default.
- **Downloading without Content-Length check:** Some mirrors don't report file size. Must handle the case where `total_bytes` is 0 (show indeterminate progress, not 0%).
- **Resuming without temp file tracking:** D-26 requires tracking partially downloaded files. Must store the temp path and downloaded byte offset.
- **Running `xattr` on non-macOS:** Use `#[cfg(target_os = "macos")]` — the command does not exist on Linux/Windows.
- **Over-engineering the UI:** D-30 explicitly says "minimal FFmpeg page" — do NOT build a full app shell, router, or layout system. Just the FFmpeg status/download card.

## Don't Hand-Roll

| Problem                          | Don't Build                          | Use Instead                                                           | Why                                                                                              |
| -------------------------------- | ------------------------------------ | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| FFmpeg binary detection          | Manual `which ffmpeg` + parsing      | `ffmpeg_sidecar::command::ffmpeg_is_installed()` + `ffmpeg_version()` | Handles PATH search, Windows PATHEXT, version string parsing, edge cases across OS               |
| Archive extraction (tar.xz, zip) | Custom tar/zip extraction            | `ffmpeg_sidecar::download::unpack_ffmpeg()`                           | Handles macOS quarantine flags, permissions, nested directory structures, Windows tar since 1803 |
| HTTP download with resume        | Custom HTTP Range request logic      | `reqwest` with `Range` header + streaming                             | Handles TLS, redirects, connection pooling, platform cert stores                                 |
| Persistent key-value storage     | Manual JSON file read/write          | `tauri-plugin-store`                                                  | Handles app_data_dir path, auto-save with debounce, atomic writes, JS/Rust interop               |
| Native directory picker          | Custom file browser UI               | `tauri-plugin-dialog` `open({directory: true})`                       | Native OS dialog, platform-appropriate UX, accessibility                                         |
| Progress display component       | Custom progress bar                  | Naive UI `NProgress`                                                  | Accessible, animated, themed, consistent with rest of UI                                         |
| State management for download    | Manual reactive state                | Pinia `defineStore`                                                   | Devtools integration, hot module replacement, TypeScript inference                               |
| i18n string management           | Custom JSON loader + reactive locale | `vue-i18n` `createI18n()`                                             | Pluralization, interpolation, lazy loading, Composition API support                              |
| Git hooks                        | Custom shell scripts                 | `husky` + `lint-staged`                                               | Cross-platform, staged-file-only, configurable                                                   |

**Key insight:** FFmpeg binary management seems simple (download a zip, extract, run) but has significant hidden complexity: platform-specific URL selection, macOS code signing quarantine, tar vs zip format differences across platforms, binary permission bits, version string parsing across FFmpeg build variants, and temp file cleanup. ffmpeg-sidecar handles most of this. Only the URL source selection and progress streaming need custom implementation.

## Runtime State Inventory

> Omitted — Phase 01 is a greenfield phase. No existing runtime state exists. The project repo is empty (no code yet).

## Common Pitfalls

### Pitfall 1: ffmpeg-sidecar URL Source Mismatch

**What goes wrong:** User expects GitHub Releases as download source (D-21), but ffmpeg-sidecar bakes third-party URLs at compile time. Direct use of `auto_download()` or `auto_download_with_progress()` will use evermeet.cx/osxexperts.net/gyan.dev/johnvansickle.com, not GitHub.
**Why it happens:** ffmpeg-sidecar's `ffmpeg_download_url()` function uses `cfg!` macros to select known-good static build hosts. GitHub Releases (BtbN) provides NO macOS builds at all, so the crate authors chose reliable per-platform sources.
**How to avoid:** Implement custom download using `reqwest` with platform-specific URL selection. Use `unpack_ffmpeg()` for extraction (it works with any archive containing ffmpeg/ffprobe binaries). GitHub Releases for Linux/Windows, evermeet.cx/osxexperts.net for macOS. Use jsDelivr as CDN mirror.
**Warning signs:** ESLint/clippy catches direct `auto_download()` calls. Review should verify download URLs in the Rust source.

### Pitfall 2: macOS Code Signing Quarantine

**What goes wrong:** Downloaded FFmpeg binary is quarantined by Gatekeeper. Running `ffmpeg -version` fails with "cannot be opened because the developer cannot be verified" or "operation not permitted."
**Why it happens:** macOS applies `com.apple.quarantine` extended attribute to all files downloaded from the internet. Quarantined binaries are blocked from execution.
**How to avoid:** After extraction, run `xattr -dr com.apple.quarantine <ffmpeg_path>` and `xattr -dr com.apple.quarantine <ffprobe_path>`. This is OS-specific: wrap in `#[cfg(target_os = "macos")]`. [ASSUMED: xattr is always available on macOS — it ships with the OS since 10.4]
**Warning signs:** `ffmpeg_version_with_path()` returns an error after successful download/extraction on macOS. The error message will mention "cannot execute" or permissions.

### Pitfall 3: create-tauri-app Version Drift

**What goes wrong:** `create-tauri-app` v4.6.2 generates a project with dependency versions that differ from CLAUDE.md pins. Newer template versions may include updated defaults (e.g., ESLint 10 instead of 9, different Vite version).
**Why it happens:** The template is maintained separately from the library versions. Scaffolding fetches latest template contents at generation time.
**How to avoid:** After scaffolding, manually verify all versions against CLAUDE.md. Replace caret ranges with exact versions (D-02). Check `tauri.conf.json` for correct `identifier` and `app.windows` configuration. Run `bun install` after version corrections.
**Warning signs:** `package.json` contains version ranges (`^` prefix) instead of exact versions. Dependency versions differ from CLAUDE.md table.

### Pitfall 4: Bun Lockfile and CI

**What goes wrong:** `bun install` generates `bun.lock` (binary format). GitHub Actions `oven-sh/setup-bun@v2` must use the same bun version as local development to avoid lockfile compatibility issues.
**Why it happens:** Bun lockfile format evolves across major versions. CI with a different bun version may fail to parse the lockfile or produce different dependency resolutions.
**How to avoid:** Pin bun version in both local development (via `.tool-versions` or `package.json.engines`) and CI (`bun-version: 1.3.2` in setup-bun action). Use `bun install --frozen-lockfile` in CI.
**Warning signs:** CI fails with lockfile parsing errors. Different `node_modules` tree between local and CI.

### Pitfall 5: Tauri Plugin Permission Gaps

**What goes wrong:** Frontend `invoke()` calls fail with "command not allowed" or "permission denied" because plugin capabilities aren't correctly configured.
**Why it happens:** Tauri v2 defaults to deny-all for plugin commands. Each command must be explicitly enabled in `src-tauri/capabilities/default.json`.
**How to avoid:** Add permission sets for each plugin: `"store:default"`, `"shell:default"`, `"dialog:default"`, `"fs:default"`. For shell commands (xattr), add explicit `shell:allow-execute` with the specific command. Test each IPC call after configuration.
**Warning signs:** Console errors in Tauri dev tools referencing "not allowed" or "unknown command". Frontend components show error states instead of data.

### Pitfall 6: Download Progress with Unknown File Size

**What goes wrong:** Some download sources don't report `Content-Length` header. The progress calculation divides by zero or shows "NaN%".
**Why it happens:** Certain CDN/mirror configurations strip the Content-Length header. GitHub Releases always reports it, but mirrors may not.
**How to avoid:** Handle `total_bytes == 0` case: show indeterminate progress (animated bar, "Downloading..." without percentage) instead of percentage. Update to determinate progress once/if Content-Length becomes available.
**Warning signs:** Progress bar stuck at 0% or showing NaN. Network tab shows missing Content-Length header.

### Pitfall 7: FFmpeg Minimum Version (4.0) Enforcement

**What goes wrong:** User has FFmpeg installed but it's an ancient version (e.g., 2.8 or 3.x). Seed operations in Phase 2+ require features from FFmpeg >= 4.0.
**Why it happens:** Many Linux distributions ship older FFmpeg versions. macOS Homebrew always provides latest, but manual installations may be stale.
**How to avoid:** Parse the version string from `ffmpeg_version()` output. Compare major version: if < 4, treat as "outdated" and prompt download. Store the version check result in the Pinia store so the UI can inform the user why their existing FFmpeg is insufficient.
**Warning signs:** User reports "FFmpeg found but features don't work." Phase 2 tests fail with "unknown filter" errors.

## Code Examples

Verified patterns from official sources:

### FFmpeg Detection (Rust)

```rust
// Source: ffmpeg-sidecar docs [VERIFIED: Context7 /nathanbabcock/ffmpeg-sidecar]
use ffmpeg_sidecar::command::ffmpeg_is_installed;
use ffmpeg_sidecar::version::ffmpeg_version;

fn check_ffmpeg() -> anyhow::Result<()> {
    if ffmpeg_is_installed() {
        let version = ffmpeg_version()?;
        println!("FFmpeg installed, version: {version}");
    } else {
        println!("FFmpeg not found — run auto_download() to install.");
    }
    Ok(())
}
```

### FFmpeg Download with Progress (Rust)

```rust
// Source: ffmpeg-sidecar docs [VERIFIED: Context7 /nathanbabcock/ffmpeg-sidecar]
// Note: auto_download_with_progress uses baked-in URLs. For custom URLs, use
// reqwest streaming + download_ffmpeg_package + unpack_ffmpeg.
use ffmpeg_sidecar::download::{auto_download_with_progress, FfmpegDownloadProgressEvent};

auto_download_with_progress(|event| match event {
    FfmpegDownloadProgressEvent::Starting => println!("Starting download..."),
    FfmpegDownloadProgressEvent::Downloading { total_bytes, downloaded_bytes } => {
        let pct = if total_bytes > 0 {
            (downloaded_bytes * 100) / total_bytes
        } else {
            0
        };
        println!("\rDownloading... {pct}%");
    }
    FfmpegDownloadProgressEvent::UnpackingArchive => println!("\nUnpacking..."),
    FfmpegDownloadProgressEvent::Done => println!("Done!"),
})?;
```

### Custom Download + Unpack (Rust)

```rust
// Source: ffmpeg-sidecar docs [VERIFIED: docs.rs/ffmpeg-sidecar]
use ffmpeg_sidecar::download::{download_ffmpeg_package, unpack_ffmpeg};
use ffmpeg_sidecar::paths::sidecar_dir;

let download_url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl-shared.tar.xz";
let destination = sidecar_dir()?;  // or user-selected path
let archive_path = download_ffmpeg_package(download_url, &destination)?;
unpack_ffmpeg(&archive_path, &destination)?;
```

### Tauri Store Persistence (Rust Side)

```rust
// Source: Tauri v2 store plugin [VERIFIED: Context7 /websites/v2_tauri_app]
use tauri::Wry;
use tauri_plugin_store::StoreExt;
use serde_json::json;

// In setup:
let store = app.store("ffmpeg-config.json")?;
store.set("ffmpeg_path", json!("/path/to/ffmpeg"));
store.set("version", json!("6.1"));
store.set("download_time", json!("2026-05-13T12:00:00Z"));
```

### Tauri Store Persistence (JS Side)

```typescript
// Source: Tauri v2 store plugin [VERIFIED: Context7 /websites/v2_tauri_app]
import { load } from '@tauri-apps/plugin-store';

const store = await load('ffmpeg-config.json', { autoSave: 100 });
await store.set('ffmpeg_path', '/path/to/ffmpeg');
const path = await store.get<string>('ffmpeg_path');
```

### Naive UI Dark Theme + NProgress

```html
<!-- Source: Naive UI docs [VERIFIED: Context7 /tusen-ai/naive-ui] -->
<template>
  <n-config-provider :theme="darkTheme" :locale="zhCN">
    <n-global-style />
    <n-space vertical align="center">
      <n-progress
        type="line"
        :percentage="downloadProgress.percent"
        :indicator-placement="'inside'"
        :height="24"
      />
      <n-text>{{ downloadedSize }} / {{ totalSize }}</n-text>
      <n-text>{{ downloadSpeed }}</n-text>
    </n-space>
  </n-config-provider>
</template>
```

### Vue I18n Setup (Composition API)

```typescript
// Source: vue-i18n docs [VERIFIED: Context7 /intlify/vue-i18n]
import { createI18n } from 'vue-i18n';
import zhCN from './locales/zh-CN.json';
import en from './locales/en.json';

const i18n = createI18n({
  legacy: false, // Composition API mode
  locale: 'zh-CN',
  fallbackLocale: 'en',
  messages: {
    'zh-CN': zhCN,
    en: en,
  },
});
```

### UnoCSS Vite Integration

```ts
// vite.config.ts
// Source: unocss.dev [CITED: unocss.dev/integrations/vite]
import UnoCSS from 'unocss/vite';
import Vue from '@vitejs/plugin-vue';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [Vue(), UnoCSS()],
});

// uno.config.ts
import { defineConfig, presetUno } from 'unocss';
export default defineConfig({
  presets: [presetUno()],
});

// src/main.ts
import 'virtual:uno.css';
```

## State of the Art

| Old Approach                       | Current Approach                         | When Changed    | Impact                                                                         |
| ---------------------------------- | ---------------------------------------- | --------------- | ------------------------------------------------------------------------------ |
| Tauri v1 + `tauri::api::dialog`    | Tauri v2 + `tauri-plugin-dialog`         | Tauri v2 (2024) | Unified plugin system across all platforms. v1 APIs removed.                   |
| ESLint 8 `.eslintrc.js`            | ESLint 9 `eslint.config.mjs` flat config | ESLint 9 (2024) | Flat config is the only format in ESLint 9+. Old format deprecated.            |
| `@tauri-apps/api/store` (v1)       | `@tauri-apps/plugin-store` (v2)          | Tauri v2        | Store is now a separate plugin, not part of core API.                          |
| `create-tauri-app@3`               | `create-tauri-app@4`                     | 2025-2026       | v4 is the current npm version (4.6.2). Interactive CLI with Vue + TS template. |
| `ffmpeg-sidecar` default URLs      | Custom URL selection                     | This phase      | GitHub Releases for Linux/Windows, default sources for macOS, mirror for CN.   |
| `KEEP_ONLY_FFMPEG=1`               | Do NOT set (need ffprobe)                | This phase      | D-16 requires ffprobe for Phase 2 metadata extraction.                         |
| Vue I18n v10 (Options API default) | Vue I18n v11 (`legacy: false`)           | v11 (2025)      | Composition API mode is now the default in v11.                                |

**Deprecated/outdated:**

- `tauri::api::dialog` — removed in Tauri v2. Use `tauri-plugin-dialog`.
- `@tauri-apps/api/dialog` — removed in Tauri v2. Use `@tauri-apps/plugin-dialog`.
- `.eslintrc.*` config files — ESLint 9+ uses flat config only.
- `vue-i18n` `allowComposition: true` option — removed in v11 (Composition API is default with `legacy: false`).
- `create-tauri-app@2` — use v4 (4.6.2) for latest templates and bun support.
- Tauri v1 sidecar bundling via `tauri.conf.json` `bundle.externalBin` — replaced by v2's `tauri-plugin-shell` capabilities-based approach.

## Assumptions Log

| #   | Claim                                                                                                            | Section               | Risk if Wrong                                                                                                                                                      |
| --- | ---------------------------------------------------------------------------------------------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| A1  | `xattr` command is always available on macOS (ships with OS since 10.4)                                          | Common Pitfalls #2    | LOW: `xattr` is part of the base macOS install. Only risk is if Apple removes it in a future OS version.                                                           |
| A2  | BtbN/FFmpeg-Builds GitHub Releases provides ffprobe in the same archive as ffmpeg                                | Code Examples         | MEDIUM: If verified that ffprobe is NOT in the archive, we need a separate ffprobe download step. User reports and the project's README indicate both are bundled. |
| A3  | jsDelivr CDN can proxy GitHub Releases assets for CN mirror fallback                                             | Architecture Patterns | MEDIUM: jsDelivr's China CDN access depends on China's Great Firewall status at any given time. Alternative: ghproxy.com or self-hosted mirror.                    |
| A4  | `reqwest` with `stream` feature provides Content-Length and download progress tracking                           | Standard Stack        | LOW: This is standard HTTP client behavior, well-documented in reqwest.                                                                                            |
| A5  | Tauri v2 + bun integration is stable (Tauri docs explicitly document `bun create tauri-app` and `bun tauri dev`) | Architecture Patterns | LOW: Confirmed by official Tauri v2 docs. Bun is listed as a supported package manager.                                                                            |
| A6  | create-tauri-app v4.6.2 Vue+TS template generates a project with Vite 8.x (not a lower version)                  | Common Pitfalls #3    | LOW: Can be verified in seconds by scaffolding and checking. If wrong, manual version adjustment is trivial.                                                       |

## Open Questions (RESOLVED)

1. **GitHub Releases as primary macOS source?**
   - RESOLVED: Accept ffmpeg-sidecar's macOS URLs as primary for macOS. GitHub Releases primary only for Linux/Windows. BtbN/FFmpeg-Builds does NOT provide macOS builds. evermeet.cx (x86_64) and osxexperts.net (aarch64) are the established macOS distribution channels used by ffmpeg-sidecar. Document this platform constraint clearly.

2. **How to implement download resume (D-26)?**
   - RESOLVED: Bypass `download_ffmpeg_package()` entirely. Implement custom download with reqwest streaming, check temp file existence on startup, send `Range: bytes={downloaded}-` header. Only use `unpack_ffmpeg()` from ffmpeg-sidecar for extraction. Track downloaded byte count and temp file path in the download state.

3. **What is the specific GitHub Releases URL pattern for BtbN builds?**
   - RESOLVED: Use the `/releases/latest` redirect for initial download on Linux/Windows. Parse the redirect URL to extract the version tag (date-based, e.g., `autobuild-2026-05-12-13-59`) for caching and update checks. URL pattern: `https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-{linux64|linuxarm64|win64}-{variant}.{ext}`.

4. **Should `tauri-plugin-shell` be used for xattr or should we use `std::process::Command` directly?**
   - RESOLVED: Execute `xattr` in Rust setup hook via `std::process::Command` after unpack completes. No need for shell plugin specifically for xattr. Install `tauri-plugin-shell` anyway (needed for future phases like Phase 2 FFmpeg process spawning).

5. **How to handle the CN mirror fallback URL construction?**
   - RESOLVED: Implement a fallback chain: Primary URL → jsDelivr CDN proxy → error with manual download link. For Linux/Windows GitHub-hosted builds, use `https://cdn.jsdelivr.net/gh/BtbN/FFmpeg-Builds@latest/...`. For macOS, evermeet.cx and osxexperts.net already serve China. Document which mirrors were tried so users can debug connectivity issues.

## Environment Availability

| Dependency   | Required By                        | Available | Version               | Fallback                     |
| ------------ | ---------------------------------- | --------- | --------------------- | ---------------------------- |
| Node.js      | Frontend dev, Vite, ESLint         | Yes       | v23.11.0              | --                           |
| bun          | Package manager (D-05), dev server | Yes       | 1.3.2                 | --                           |
| Rust (rustc) | Tauri backend compilation          | Yes       | 1.94.1                | --                           |
| Cargo        | Rust dependency management         | Yes       | 1.94.1                | --                           |
| pnpm         | Alternative if bun has issues      | Yes       | 10.33.0               | Use if bun scaffolding fails |
| FFmpeg       | Runtime (detected or downloaded)   | No        | --                    | This phase will install it   |
| FFprobe      | Runtime (Phase 2 metadata)         | No        | --                    | This phase will install it   |
| Git          | Version control, husky             | Yes       | (present)             | --                           |
| macOS        | Target platform (development)      | Yes       | Darwin 25.0.0 (ARM64) | --                           |

**Missing dependencies with no fallback:**

- None. FFmpeg/FFprobe are the purpose of this phase — their absence is expected and will be resolved by the implementation.

**Missing dependencies with fallback:**

- None. All build/runtime tools are available.

**Platform notes:**

- Development machine: macOS ARM64 (Apple Silicon). ffmpeg-sidecar will select `https://www.osxexperts.net/ffmpeg80arm.zip` as the default macOS aarch64 binary.
- For cross-platform consideration: Linux and Windows builds will need GitHub Actions CI runners. Linux requires `libwebkit2gtk-4.1-dev` and other system deps installed on CI.

## Security Domain

### Applicable ASVS Categories

| ASVS Category         | Applies | Standard Control                                                                                                                                                                                     |
| --------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| V2 Authentication     | No      | N/A — no user accounts in Phase 1                                                                                                                                                                    |
| V3 Session Management | No      | N/A — no sessions                                                                                                                                                                                    |
| V4 Access Control     | No      | N/A — local desktop app                                                                                                                                                                              |
| V5 Input Validation   | Yes     | TypeScript strict mode (D-03) validates FFmpeg version strings, paths. Rust type system validates Tauri command parameters. Path traversal prevention via `tauri-plugin-fs` scope restrictions.      |
| V6 Cryptography       | No      | N/A — no cryptographic operations                                                                                                                                                                    |
| V7 Error Handling     | Yes     | `anyhow` for ergonomic Rust errors. Frontend error boundaries via Pinia store error state. Never expose raw system paths or FFmpeg stderr to user without sanitization.                              |
| V8 Data Protection    | Yes     | FFmpeg binary path, version, download time stored in local app_data_dir via `tauri-plugin-store`. No sensitive data.                                                                                 |
| V9 Communication      | Yes     | All IPC via Tauri's typed `invoke()` and event system. Plugin permissions explicitly allow-listed in capabilities.                                                                                   |
| V10 Malicious Code    | Yes     | Downloaded FFmpeg binary verified post-extraction (`ffmpeg -version` succeeds). Checksum verification recommended for download integrity (can be added for GitHub Releases which publish checksums). |

### Known Threat Patterns for Tauri + FFmpeg Desktop App

| Pattern                                                     | STRIDE                 | Standard Mitigation                                                                                                                      |
| ----------------------------------------------------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Malicious FFmpeg binary substitution (MITM during download) | Tampering              | Use HTTPS (enforced by reqwest). Verify checksum against GitHub Releases published SHA256.                                               |
| Path traversal in user-selected download directory          | Tampering / Elevation  | `tauri-plugin-fs` scope restricts file operations to allowed paths. Sanitize user-provided paths.                                        |
| Command injection via FFmpeg arguments (future phases)      | Elevation              | Not applicable in Phase 1 (only version check and download). For Phase 2+: use argument arrays, never string interpolation.              |
| DLL sideloading on Windows                                  | Elevation              | Verify digital signature of downloaded binaries on Windows. Store FFmpeg in app-local directory (not system PATH).                       |
| Unvalidated binary execution                                | Elevation              | `ffmpeg -version` verification catches corrupt/non-executable files. macOS quarantine removal only after explicit user download consent. |
| Tauri webview CSP bypass                                    | Information Disclosure | Tauri v2 default CSP is restrictive. Plugin capabilities explicitly allow-listed per command.                                            |
| Download temp file leakage                                  | Information Disclosure | Clean temp files on cancel (D-27) and on app exit. Use OS temp directory (`std::env::temp_dir()`).                                       |

## Sources

### Primary (HIGH confidence)

- [Context7: Tauri v2 docs](/websites/v2_tauri_app) — scaffolding (`bun create tauri-app`), project structure, window config, plugin permissions, capabilities, sidecar, store plugin Rust+JS API, shell plugin, dialog plugin. 3033 snippets.
- [Context7: ffmpeg-sidecar](/nathanbabcock/ffmpeg-sidecar) — `ffmpeg_is_installed`, `ffmpeg_version`, `auto_download`, `auto_download_with_progress`, `FfmpegCommand`, progress iteration. 67 snippets.
- [Context7: ffmpeg-sidecar (docs.rs)](/websites/rs_ffmpeg-sidecar) — `ffmpeg_download_url`, `download_ffmpeg_package`, `unpack_ffmpeg`, `sidecar_dir`, `check_latest_version`, `ffmpeg_version_with_path`. 1098 snippets.
- [Context7: Naive UI](/tusen-ai/naive-ui) — `NConfigProvider`, `darkTheme`, `NButton`, `NProgress`, `NSpace`, `NText`, locale, theme customization. 593 snippets.
- [Context7: Pinia](/vuejs/pinia) — Composition API setup stores, `defineStore`, `ref`/`computed` in stores. HIGH confidence.
- [Context7: Vue I18n](/intlify/vue-i18n) — `createI18n`, Composition API mode, locale messages, `useI18n`. 856 snippets.
- [docs.rs/ffmpeg-sidecar](https://docs.rs/ffmpeg-sidecar/latest/ffmpeg_sidecar/download/fn.download_ffmpeg_package.html) — Verified custom URL support in `download_ffmpeg_package(url: &str, download_dir: &Path) -> Result<PathBuf>`.
- [npm registry](https://www.npmjs.com/) — All frontend and dev dependency versions verified. [VERIFIED: npm view]
- [crates.io](https://crates.io/) — All Rust crate versions verified. [VERIFIED: cargo search]

### Secondary (MEDIUM confidence)

- [typescript-eslint.io/getting-started](https://typescript-eslint.io/getting-started/) — `tseslint.config()` setup, flat config integration with ESLint 9. [CITED]
- [bun.sh/guides/runtime/cicd](https://bun.sh/guides/runtime/cicd) — `oven-sh/setup-bun@v2` GitHub Action, caching strategy. [CITED]
- [unocss.dev/integrations/vite](https://unocss.dev/integrations/vite) — Vite plugin setup, `uno.config.ts`, `virtual:uno.css` import. [CITED]
- [GitHub: BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds/releases) — Release structure, platform availability (Linux x86_64, Linux arm64, Windows x64; NO macOS). [CITED]
- [Tauri v2 blog: create-tauri-app v3](https://v2.tauri.app/blog/create-tauri-app-version-3-released) — CLI prompts, template selection, package manager support.

### Tertiary (LOW confidence)

- [ASSUMED] macOS `xattr` command availability on all macOS versions since 10.4.
- [ASSUMED] BtbN/FFmpeg-Builds includes ffprobe in the same archive as ffmpeg (widely reported, not verified by direct archive inspection).

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all versions verified against npm registry and crates.io on 2026-05-13
- Architecture: HIGH — Tauri v2 plugin system, IPC patterns, and ffmpeg-sidecar API confirmed through multiple Context7 sources
- Pitfalls: MEDIUM-HIGH — pitfalls identified from library documentation and known Tauri v2 migration issues; macOS quarantine pitfall is well-documented
- Download URLs for GitHub Releases: MEDIUM — BtbN provides Linux/Windows but not macOS; CN mirror strategy needs runtime validation

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (30 days for stable libraries; ffmpeg-sidecar URLs may change, re-verify if Phase 1 extends beyond this date)

**Note on CLAUDE.md deviations identified:**

- CLAUDE.md recommends `KEEP_ONLY_FFMPEG=1` (skip ffplay/ffprobe to save ~30MB). D-16 overrides this: user requires ffprobe for Phase 2 metadata extraction.
- CLAUDE.md lists `eslint@9.x`, `eslint-plugin-vue@10.x`, `@eslint/js@9.x` — versions confirmed and matched to latest in 9.x/10.x lines (ESLint 10 exists but user pinned to 9).
