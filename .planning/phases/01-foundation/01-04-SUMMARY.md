---
phase: 01-foundation
plan: 04
subsystem: frontend-infra
tags: [vue, pinia, i18n, naive-ui, dark-theme, ffmpeg-status, download-ui, tauri-ipc]
dependency_graph:
  requires: [01-01]
  provides: [ffmpeg-state-management, ffmpeg-detection-ui, ffmpeg-download-ui, dark-theme-app-shell, bilingual-i18n]
  affects: [phase-02-rust-backend, phase-03-vue-frontend]
tech_stack:
  added: [lucide-vue-next@1.0.0, @types/node@25.7.0]
  patterns: [pinia-composition-store, tauri-invoke-ipc, tauri-event-listening, naive-ui-dark-theme, vue-i18n-composition-api, conditional-view-routing]
key_files:
  created:
    - src/types/ffmpeg.ts
    - src/stores/ffmpeg.ts
    - src/composables/useFfmpeg.ts
    - src/components/FFmpegStatus.vue
    - src/components/FFmpegDownload.vue
    - src/components/PlaceholderHome.vue
    - src/locales/zh-CN.json
    - src/locales/en.json
    - src/utils/i18n.ts
  modified:
    - src/main.ts
    - src/App.vue
    - tsconfig.json
    - tsconfig.node.json
    - package.json
    - .gitignore
decisions:
  - "lucide-vue-next@1.0.0 installed instead of plan's 0.507.2 (version does not exist)"
  - "@types/node@25.7.0 installed as devDependency (required by tsconfig.node.json project reference)"
  - "Unused icon imports (Loader, Download) removed for strict noUnusedLocals compliance"
  - "No Vue Router — conditional v-if/v-else-if routing per D-14"
  - "showDownload local ref in App.vue decouples download overlay from store state machine"
metrics:
  duration: "~7 minutes"
  completed_date: "2026-05-13T04:13:18Z"
  tasks: 3
  files_created: 9
  files_modified: 6
---

# Phase 1 Plan 4: Frontend Infrastructure — FFmpeg UI, i18n, and Dark Theme App Shell Summary

Built the complete Vue 3 frontend infrastructure: TypeScript type contracts matching Rust structs, Pinia state machine for FFmpeg lifecycle, Tauri IPC composable for all backend commands, three Vue components (status detection, download progress, placeholder home), bilingual i18n (zh-CN + en), and Naive UI dark theme app shell with state-driven routing. `vue-tsc -b` exits 0 and `bun run build` produces a 408KB production bundle.

## Tasks Completed

### Task 1: Create TypeScript type definitions, Pinia FFmpeg store, Tauri IPC composable, and app entry point (main.ts)

**Commit:** `2afc0bb`

Created the foundational frontend modules that connect Vue components to the Rust backend:

- **src/types/ffmpeg.ts**: `FfmpegInfo`, `DownloadProgress`, `DownloadStage`, `FfmpegStatus` TypeScript interfaces exactly mirroring Rust structs from Plan 03. All camelCase fields match serde's `rename_all = "camelCase"`.
- **src/stores/ffmpeg.ts**: Pinia Composition API store with 9 refs (status, version, path, downloadProgress, downloadError, retryCount, targetDir), 3 computeds (isReady, needsDownload, isDownloading), and 3 actions (setFfmpegInfo, setDownloadProgress, resetDownload). Full state machine covering all 9 FfmpegStatus states.
- **src/composables/useFfmpeg.ts**: Tauri IPC wrappers exposing 8 functions: `detect()` (invokes `detect_ffmpeg`), `subscribeProgress()` / `subscribeStatus()` / `subscribeReady()` (event listeners), `selectDirectory()` (native OS dir picker), `startDownload()` / `cancelDownload()` (command invokers), `unsubscribeAll()` (cleanup).
- **src/main.ts**: Rewritten to register `createPinia()`, `vue-i18n`, and import `virtual:uno.css` before mounting Vue app.

**TypeScript config fixes (Rule 3):**
- Added `ignoreDeprecations: "6.0"` to tsconfig.json for TypeScript 6's deprecated `baseUrl`
- Added `composite: true` to tsconfig.node.json for project references

### Task 2: Create FFmpegStatus, FFmpegDownload, and PlaceholderHome Vue components

**Commit:** `9b05313`

Created three Vue 3 `<script setup lang="ts">` components consuming the Pinia store and Tauri composable:

- **FFmpegStatus.vue**: Centered card with 4 state-driven views:
  - `detecting`: NSpin loading spinner + "Detecting FFmpeg..." text
  - `found`/`verified`: Green checkmark icon, version tag, auto-transition hint
  - `missing`/`outdated`: Warning icon, contextual message, "Download FFmpeg" button (emits `start-download`)
  - `error`: Error icon, message, retry button calling `detect()`
  - Calls `subscribeStatus()`, `subscribeReady()`, and `detect()` in `onMounted`

- **FFmpegDownload.vue**: Full download page with 4 state views:
  - Directory selection: native OS folder picker triggered by "Choose Directory" button
  - Downloading: NProgress bar with percentage, downloaded/total size, speed, cancel button
  - Verifying: NProgress success variant, "Verifying..." text
  - Error: Error message, retry button (up to 3 attempts), then manual download instructions
  - `formatBytes()` and `formatSpeed()` utility functions for human-readable sizes
  - Subscribes to progress events in `onMounted`, unsubscribes in `onUnmounted`

- **PlaceholderHome.vue**: Welcome page per D-35 showing "Sandwich" header, tagline, FFmpeg version, "Ready" status, and "Awaiting future features" text. All in dark theme centered card layout.

**Installation:** `lucide-vue-next@1.0.0` for tree-shakeable SVG icons (plan specified 0.507.2 which does not exist — Rule 3 version fix).

**Bug fixes (Rule 1):**
- Removed unused `Loader` import from FFmpegStatus.vue (NSpin used instead)
- Removed unused `Download` import from FFmpegDownload.vue (FolderOpen used instead)

### Task 3: Create i18n locale files and wire App.vue with Naive UI dark theme + state routing

**Commit:** `d16a8a5`

Created the bilingual i18n system and wired the root app shell:

- **src/locales/zh-CN.json**: 20 translation keys across 3 namespaces (common, ffmpeg, download) — all UI text in Simplified Chinese
- **src/locales/en.json**: Matching 20 translation keys in English
- **src/utils/i18n.ts**: vue-i18n v11 Composition API setup with `legacy: false`, default `zh-CN`, fallback `en`
- **src/App.vue**: Root component — `NConfigProvider` with `darkTheme`, Naive UI locale synchronized to vue-i18n locale, state-driven conditional rendering:
  - `showDownload` → FFmpegDownload (overlay)
  - `ffmpegStore.isReady` → PlaceholderHome
  - Default → FFmpegStatus
  - No Vue Router (D-14) — `v-if`/`v-else-if` chain

**Config fixes:**
- Added `resolveJsonModule: true` to tsconfig.json for JSON locale imports
- Installed `@types/node@25.7.0` dev dependency (required by tsconfig.node.json's `"types": ["node"]`)
- Added generated build artifacts (`*.tsbuildinfo`, SDK emit files) to `.gitignore`

### Build Verification

- `vue-tsc -b` exits 0 — zero type errors
- `bun run build` exits 0 — 408KB JS bundle, 1.85KB CSS
- 4540 modules transformed in 763ms

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] lucide-vue-next@0.507.2 does not exist**
- **Found during:** Task 2, Step B
- **Issue:** `bun add lucide-vue-next@0.507.2` fails with "No version matching"
- **Fix:** Installed latest version 1.0.0; all icon exports verified unchanged (AlertCircle, CheckCircle, Download, XCircle, RefreshCw, FolderOpen)
- **Files modified:** package.json, bun.lock
- **Commit:** 9b05313

**2. [Rule 1 - Bug] Unused icon imports causing TS6133 errors**
- **Found during:** Task 2, vue-tsc verification
- **Issue:** `Loader` imported but unused in FFmpegStatus.vue (NSpin renders spinner); `Download` imported but unused in FFmpegDownload.vue (FolderOpen renders directory button)
- **Fix:** Removed unused imports from both component files
- **Files modified:** src/components/FFmpegStatus.vue, src/components/FFmpegDownload.vue
- **Commit:** 9b05313

**3. [Rule 3 - Blocking] TypeScript 6 baseUrl deprecation (TS5101)**
- **Found during:** Task 1, vue-tsc verification
- **Issue:** TypeScript 6.0 deprecates `baseUrl` without `ignoreDeprecations: "6.0"`; causes TS5101 error
- **Fix:** Added `ignoreDeprecations: "6.0"` to tsconfig.json compilerOptions
- **Files modified:** tsconfig.json
- **Commit:** 2afc0bb

**4. [Rule 3 - Blocking] tsconfig.node.json missing composite (TS6306)**
- **Found during:** Task 1, vue-tsc verification
- **Issue:** Project references require `composite: true` in referenced project; causes TS6306 error
- **Fix:** Added `composite: true` to tsconfig.node.json compilerOptions
- **Files modified:** tsconfig.node.json
- **Commit:** 2afc0bb

**5. [Rule 3 - Blocking] resolveJsonModule missing for JSON locale imports**
- **Found during:** Task 3, vue-tsc verification
- **Issue:** `import zhCN from '@/locales/zh-CN.json'` fails without `resolveJsonModule: true`
- **Fix:** Added `resolveJsonModule: true` to tsconfig.json compilerOptions
- **Files modified:** tsconfig.json
- **Commit:** d16a8a5

**6. [Rule 3 - Blocking] @types/node missing (TS2688)**
- **Found during:** Task 3, vue-tsc -b verification
- **Issue:** tsconfig.node.json specifies `"types": ["node"]` but @types/node not installed; causes TS2688
- **Fix:** Installed @types/node@25.7.0 as devDependency (pinned, not caret)
- **Files modified:** package.json, bun.lock
- **Commit:** d16a8a5

**7. [Rule 3 - Blocking] Build artifacts from vue-tsc -b not gitignored**
- **Found during:** Post-commit untracked file check
- **Issue:** `vue-tsc -b` with project references emits `.js`, `.d.ts`, `.tsbuildinfo` files in source tree
- **Fix:** Added patterns for `*.tsbuildinfo`, `src/**/*.js`, `src/**/*.d.ts`, and config emit files to `.gitignore`
- **Files modified:** .gitignore
- **Commit:** d16a8a5

### Plan Discrepancies Noted

- **Plan states lucide-vue-next version 0.507.2** — version does not exist on npm registry. Installed 1.0.0. All icon names unchanged between versions so zero code impact.
- **Plan does not mention @types/node dependency** — required by tsconfig.node.json project reference set up in Plan 01. Added as devDependency.
- **Plan's FFmpegStatus.vue imports Loader icon** — but template uses NSpin component for the loading state. Removed unused import.
- **Plan's FFmpegDownload.vue imports Download icon** — but template uses FolderOpen for directory selection button. Removed unused import.
- **Plan does not specify .gitignore updates for build artifacts** — vue-tsc -b generates emit files that should be gitignored.

## Threat Flags

None. All 5 mitigations from the plan's `<threat_model>` are in place:

| Threat ID | Mitigation | Status |
|-----------|-----------|--------|
| T-04-01 | Command names compile-time checked by Tauri | Accept — inherent to Tauri architecture |
| T-04-02 | TypeScript interfaces mirror Rust structs exactly | Mitigated — camelCase fields match serde rename_all |
| T-04-03 | Progress data is non-sensitive (bytes, speed, percent) | Accept |
| T-04-04 | Rust side throttles events to 10/sec max | Mitigated — frontend reactive state, no unbounded growth |
| T-04-05 | Directory picker uses OS-native dialog, fs scope restricted | Mitigated — tauri-plugin-dialog + fs plugin scope |

No new threat surfaces introduced beyond what the plan accounts for.

## Known Stubs

None. The `PlaceholderHome.vue` "Awaiting future features" text is by design (D-35) and not a stub — it accurately reflects Phase 1 scope boundary. All UI elements render real store data and i18n text. No hardcoded empty arrays, placeholder data, or unwired components.

## Self-Check: PASSED

- [x] All 3 commits exist: 2afc0bb, 9b05313, d16a8a5
- [x] All 9 created files exist on disk
- [x] `vue-tsc -b` exits 0 with no type errors
- [x] `bun run build` exits 0 with production bundle
- [x] All i18n keys present in both zh-CN and en locale files
- [x] All 3 components import from Pinia store and vue-i18n
- [x] App.vue uses NConfigProvider with darkTheme
- [x] .gitignore updated for build artifacts
- [x] SUMMARY.md written
