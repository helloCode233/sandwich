---
phase: 01-foundation
plan: 01
subsystem: scaffold
tags: [scaffold, dependencies, configuration, tauri, vue, vite, typescript]
dependency_graph:
  requires: []
  provides: [buildable-project-skeleton, pinned-dependencies, all-config-files]
  affects: [01-02-ffmpeg-detection, 01-03-ffmpeg-download, 01-04-frontend-infra]
tech_stack:
  added:
    [
      tauri@2.11.1,
      vue@3.5.34,
      vite@8.0.12,
      typescript@6.0.3,
      naive-ui@2.44.1,
      pinia@3.0.4,
      vue-i18n@11.4.2,
      unocss@66.6.8,
      ffmpeg-sidecar@2.5.1,
      tokio@1.52.3,
      reqwest@0.12,
    ]
  patterns:
    [exact-version-pinning, tauri-generate-context, vite-vue-plugin-setup, unocss-vite-integration]
key_files:
  created:
    - package.json
    - src-tauri/Cargo.toml
    - src-tauri/tauri.conf.json
    - tsconfig.json
    - tsconfig.node.json
    - vite.config.ts
    - uno.config.ts
    - index.html
    - src-tauri/capabilities/default.json
    - src-tauri/src/main.rs
    - src-tauri/src/lib.rs
    - src-tauri/build.rs
    - src/main.ts
    - src/App.vue
    - src/vite-env.d.ts
    - .gitignore
    - src-tauri/icons/icon.png
  modified: []
decisions:
  - 'Manual scaffold instead of create-tauri-app (CLI requires TTY, not available in executor environment)'
  - 'All 39 user decisions (D-01 through D-39) from CONTEXT.md encoded in config files'
  - 'Placeholder RGBA icons generated programmatically for Tauri build requirement'
  - 'Frontend directories created per D-11 with .gitkeep placeholders'
  - 'Tauri gen/schemas committed for IDE $schema validation support'
metrics:
  duration: '~10 minutes'
  completed_date: '2026-05-13T03:51:23Z'
  tasks: 2
  files_created: 26
  files_modified: 0
---

# Phase 1 Plan 1: Scaffold Tauri 2.x + Vue 3 + Vite Project Summary

Tauri 2.11.1 desktop app scaffolded with Vue 3.5.34 frontend, Vite 8.0.12 build tooling, and 26 pinned dependencies across npm (9 prod + 17 dev) and Rust (11 crates + 1 dev). All config files configured to exact specifications from CONTEXT.md decisions D-01 through D-39. cargo check passes with zero errors.

## Tasks Completed

### Task 1: Scaffold project with create-tauri-app and install all pinned dependencies

**Commit:** `4b70a4d`

Scaffolded the complete Tauri + Vue + Vite + TypeScript project skeleton manually (create-tauri-app CLI requires a TTY, which is unavailable in the executor environment). All dependencies installed at exact pinned versions as specified in CLAUDE.md and RESEARCH.md.

**Key accomplishments:**

- Created all 15 source/config files matching the vue-ts template structure
- 9 production dependencies at exact versions (vue@3.5.34, @tauri-apps/api@2.11.0, naive-ui@2.44.1, pinia@3.0.4, vue-i18n@11.4.2, 4 Tauri plugins)
- 17 devDependencies at exact versions (vite@8.0.12, typescript@6.0.3, eslint@9.39.4, vitest@4.1.6, and 13 more)
- 11 Rust crate dependencies at exact versions (tauri@2.11.1, ffmpeg-sidecar@2.5.1, tokio@1.52.3, and 8 more)
- rstest@0.26.1 as dev-dependency
- Rust edition 2024 confirmed
- Placeholder RGBA icons (4 sizes) generated for Tauri build requirement
- `cargo check` passes with zero errors
- `bun install` completes with all packages resolved
- All 11 npm scripts configured (dev, build, preview, tauri, lint, format, format:check, typecheck, test, test:watch, prepare)
- Zero caret (^) or tilde (~) in any dependency version string

### Task 2: Configure all project config files (Tauri, TypeScript, Vite, UnoCSS, capabilities)

**Commit:** `c089596`

Verified all config files match the plan's exact specifications. Created frontend directory structure and committed generated Tauri schema files.

**Key accomplishments:**

- tauri.conf.json: identifier "com.sandwich.app", title "Sandwich", 1200x800 with minWidth 900, minHeight 600, beforeDevCommand "bun dev"
- tsconfig.json: strict=true, noUnusedLocals/Parameters, noFallthroughCasesInSwitch, noUncheckedSideEffectImports, @/\* path alias
- tsconfig.node.json: separate config for build tool files (vite.config.ts, uno.config.ts)
- vite.config.ts: Vue() and UnoCSS() plugins, port 1420, strict port, Tauri HMR config, @ alias resolve
- uno.config.ts: presetUno() atomic CSS
- capabilities/default.json: store:default, shell:default, dialog:default, fs:default, shell:allow-execute for xattr, fs scope restricted to $APPDATA/** and $HOME/**
- index.html: standard Tauri entry with `<div id="app"></div>` and `<script type="module" src="/src/main.ts"></script>`
- 6 frontend directories created: src/components/, src/stores/, src/composables/, src/types/, src/utils/, src/locales/
- Tauri auto-generated schema files (gen/schemas/) committed for IDE validation
- 23 automated verification checks all pass
- `cargo check` still passes with zero errors

## Verification Results

### Task 1 Verification

- [x] No caret (^) or tilde (~) in any package.json dependency version
- [x] All 9 production dependencies at exact pinned versions
- [x] All 17 devDependencies at exact pinned versions
- [x] Cargo.toml edition = "2024"
- [x] All 11 Rust crate dependencies + rstest dev-dependency at pinned versions
- [x] `cargo check` completes without errors
- [x] `bun install` completes without errors
- [x] All 11 npm scripts present

### Task 2 Verification

- [x] tauri.conf.json: identifier "com.sandwich.app", title "Sandwich", 1200x800, resizable, minWidth 900, minHeight 600
- [x] tsconfig.json: strict=true, @/\* alias
- [x] vite.config.ts: Vue + UnoCSS plugins
- [x] uno.config.ts: presetUno() exported
- [x] capabilities/default.json: store:default, shell:default, dialog:default, fs:default
- [x] capabilities: shell:allow-execute for /usr/bin/xattr
- [x] index.html: div#app + module script
- [x] All 6 frontend directories exist
- [x] `cargo check` completes without errors after all config changes

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] create-tauri-app requires TTY — manually scaffolded instead**

- **Found during:** Task 1, Step A
- **Issue:** Both `bun create tauri-app@4.6.2` and `npx create-tauri-app@4.6.2` fail with "IO error: not a terminal" in the non-interactive executor environment
- **Fix:** Manually created all 15 scaffold files matching the vue-ts template output; all config files set to the exact specifications from the plan
- **Files created:** package.json, src-tauri/Cargo.toml, src-tauri/build.rs, src-tauri/tauri.conf.json, src-tauri/capabilities/default.json, src-tauri/src/main.rs, src-tauri/src/lib.rs, index.html, src/main.ts, src/App.vue, tsconfig.json, tsconfig.node.json, vite.config.ts, uno.config.ts, src/vite-env.d.ts
- **Commit:** 4b70a4d

**2. [Rule 3 - Blocking] Missing icon.png required by Tauri generate_context! macro**

- **Found during:** Task 1, Step F (cargo check)
- **Issue:** `generate_context!()` macro panics: "failed to open icon .../icons/icon.png: No such file or directory"
- **Fix:** Created minimal valid RGBA PNG icons (512x512, 128x128, 32x32, 128x128@2x) programmatically
- **Files created:** src-tauri/icons/icon.png, 128x128.png, 32x32.png, 128x128@2x.png
- **Commit:** 4b70a4d

**3. [Rule 3 - Blocking] First icon attempt used RGB format — Tauri requires RGBA**

- **Found during:** Task 1, Step F (cargo check retry)
- **Issue:** `generate_context!()` macro panics: "icon .../icons/icon.png is not RGBA"
- **Fix:** Regenerated all icons with color type 6 (RGBA) instead of color type 2 (RGB)
- **Files modified:** src-tauri/icons/icon.png, 128x128.png, 32x32.png, 128x128@2x.png
- **Commit:** 4b70a4d

### Plan Discrepancies Noted

- **Plan states 10 production dependencies** but the dependencies block in Step B lists 9 entries (vue, @tauri-apps/api, naive-ui, pinia, vue-i18n, @tauri-apps/plugin-store, @tauri-apps/plugin-shell, @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs). All 9 from the explicit list are installed at pinned versions.
- **Plan states 10 Rust crate dependencies** but `cargo add` commands add 11 (tauri, ffmpeg-sidecar, tokio, serde, serde_json, anyhow, reqwest, and 4 tauri plugins). All 11 are installed. The `serde + serde_json` command adds 2 crates.

## Threat Surface Verification

All 4 mitigations from the plan's `<threat_model>` are in place:

| Threat ID | Mitigation                                                    | Status      |
| --------- | ------------------------------------------------------------- | ----------- |
| T-01-01   | capabilities/default.json uses per-plugin allow-lists         | Implemented |
| T-01-02   | No secrets in config files                                    | Verified    |
| T-01-03   | fs scope restricted to $APPDATA/** and $HOME/**               | Implemented |
| T-01-04   | All versions exact-pinned, bun.lock provides integrity hashes | Implemented |

No new threat surfaces introduced beyond what the plan accounts for.

## Known Stubs

None. The project skeleton is intentionally minimal — `src/App.vue` contains placeholder text "Hello from Sandwich" and `src/main.ts` creates a bare Vue app. These are expected foundations for Plans 02-04 to build upon, not stubs that prevent the plan's goal (buildable project skeleton) from being achieved.

## Self-Check: PASSED

- [x] Both commits exist: 4b70a4d, c089596
- [x] All 26 created files exist on disk
- [x] package.json has no caret/tilde in deps
- [x] Cargo.toml has edition = "2024" and all crates
- [x] cargo check passes
- [x] SUMMARY.md written
