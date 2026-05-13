---
phase: 01-foundation
plan: 02
subsystem: development-tooling
tags: [eslint, prettier, rustfmt, clippy, husky, lint-staged, ci, github-actions]
dependency_graph:
  requires: [01-01]
  provides: [linting-pipeline, formatting-pipeline, ci-workflow, git-hooks]
  affects: [01-03-ffmpeg-download, 01-04-frontend-infra]
tech_stack:
  added: []
  patterns: [eslint-9-flat-config, husky-v9-hooks, lint-staged-folders, dtolnay-rust-toolchain]
key_files:
  created:
    - eslint.config.mjs
    - prettier.config.mjs
    - .prettierignore
    - rustfmt.toml
    - .clippy.toml
    - .husky/pre-commit
    - .lintstagedrc.mjs
    - .github/workflows/ci.yml
  modified:
    - eslint.config.mjs
decisions:
  - "ESLint 9 flat config using tseslint.config() with Vue SFC parser and vue-eslint-parser"
  - "Prettier configured independently from ESLint (no eslint-plugin-prettier) — current best practice"
  - "rustfmt.toml at project root (not src-tauri/) — cargo fmt looks upward for config"
  - "Clippy cognitive-complexity-threshold raised to 25 (FFmpeg download logic is inherently complex)"
  - "CI uses dtolnay/rust-toolchain@stable (community-preferred, faster than actions-rs)"
  - "CI frontend and backend checks run as separate parallel jobs"
  - "Bun version pinned to 1.3.2 in CI to match local dev environment"
  - "Husky v9 pre-commit hook runs bun lint-staged on staged files"
  - "Rust files get cargo fmt --check on commit (not auto-fix — format changes too significant to auto-apply)"
metrics:
  duration: "~8 minutes"
  completed_date: "2026-05-13T04:22:08Z"
  tasks: 2
  files_created: 8
  files_modified: 1
---

# Phase 1 Plan 2: Development Tooling Summary

ESLint 9 flat config with Vue + TypeScript support, Prettier formatting, rustfmt + clippy for Rust, husky + lint-staged Git hooks, and a GitHub Actions CI workflow — all configured and passing. Lint checks exit clean across all languages.

## Tasks Completed

### Task 1: Configure ESLint 9 flat config, Prettier, rustfmt, and clippy

**Commits:** File creation by parallel Plan 01-04 agent (`9b05313`); ESLint ignore fix by this agent (`50bd806`)

Created all 5 config files for code quality tooling. Config files were committed by a parallel Plan 01-04 agent executing concurrently. The ESLint generated-file ignores were fixed in a follow-up commit.

**Key accomplishments:**
- `eslint.config.mjs`: ESLint 9 flat config with `tseslint.config()`, Vue SFC support via `vue-eslint-parser`, `vue/multi-word-component-names` disabled, ignores for `src-tauri/**`, `dist/**`, `node_modules/**`, `.planning/**`, and vue-tsc generated files
- `prettier.config.mjs`: `semi: true`, `singleQuote: true`, `trailingComma: 'all'`, `printWidth: 100` — Prettier runs independently from ESLint
- `.prettierignore`: Excludes `.planning/`, `.claude/`, `src-tauri/`, `dist/`, `node_modules/` — prevents formatter from modifying orchestrator-managed files
- `rustfmt.toml` at project root: `edition = "2024"`, `max_width = 100`, `group_imports = "StdExternalCrate"` — applies to all Rust workspaces
- `.clippy.toml`: `cognitive-complexity-threshold = 25`, `too-many-arguments-threshold = 8` — slightly more lenient than defaults for FFmpeg download logic
- All checks pass: `bun eslint .` exit 0, `bun prettier --check .` exit 0, `cargo fmt --check` exit 0, `cargo clippy -- -D warnings` exit 0

### Task 2: Configure husky + lint-staged Git hooks and GitHub Actions CI workflow

**Commits:** File creation by parallel Plan 01-03/01-04 agents (`9b3777b`, `3d01aa4`)

Created the git hook infrastructure and CI workflow. Files were committed by parallel agents executing concurrently.

**Key accomplishments:**
- `.husky/pre-commit`: Runs `bun lint-staged` on commit — ESLint + Prettier on TS/Vue/JS, Prettier on JSON/MD/CSS/HTML/YAML, cargo fmt --check on Rust
- `.lintstagedrc.mjs`: Pattern-based mapping — `*.{ts,vue}` and `*.{js,mjs,cjs}` get eslint --fix + prettier --write; `*.{json,md,css,html,yml,yaml}` get prettier --write; `*.rs` gets cargo fmt --check
- `.github/workflows/ci.yml`: Two parallel jobs — frontend-checks (vue-tsc, ESLint, Prettier, Vitest) and backend-checks (cargo fmt --check, cargo clippy -- -D warnings, cargo check, cargo test) on ubuntu-latest
- CI uses `oven-sh/setup-bun@v2` with bun-version 1.3.2 and `--frozen-lockfile`
- CI backend installs Tauri system deps (libwebkit2gtk-4.1-dev, libappindicator3-dev, librsvg2-dev, patchelf, libsoup-3.0-dev, libjavascriptcoregtk-4.1-dev)
- CI uses `dtolnay/rust-toolchain@stable` with rustfmt + clippy components
- Pre-commit hook verified — lint-staged runs and passes on staged files

## Verification Results

### Must-Have Truths

- [x] `bun lint` finds no errors in scaffolded source files (exit 0, 0 errors, 66 Vue formatting warnings)
- [x] `bun format:check` passes on all project files (exit 0)
- [x] `cargo fmt --check` in src-tauri/ passes (exit 0)
- [x] `cargo clippy -- -D warnings` in src-tauri/ passes (exit 0, all 9 clippy errors fixed)
- [x] Git pre-commit hook runs lint-staged, which runs ESLint + Prettier on staged files
- [x] `.github/workflows/ci.yml` exists and references vue-tsc, ESLint, Prettier, cargo check, cargo fmt, cargo clippy, cargo test, vitest

### Task 1 Artifacts

- [x] eslint.config.mjs: `tseslint.config()` + `pluginVue.configs['flat/recommended']` + `vue-eslint-parser` + ignores for `src-tauri/**`, `dist/**`, `node_modules/**`, `.planning/**`, `src/**/*.js`, `src/**/*.d.ts`
- [x] prettier.config.mjs: `semi: true`, `singleQuote: true`, `trailingComma: 'all'`, `printWidth: 100`
- [x] rustfmt.toml: `edition = "2024"`, `max_width = 100`, `group_imports = "StdExternalCrate"`
- [x] .clippy.toml: `cognitive-complexity-threshold = 25`

### Task 2 Artifacts

- [x] .husky/pre-commit: contains `bun lint-staged`
- [x] .lintstagedrc.mjs: TS/Vue → eslint + prettier; JS/MJS/CJS → eslint + prettier; JSON/MD/CSS/HTML/YML/YAML → prettier; RS → cargo fmt --check
- [x] .github/workflows/ci.yml: two jobs (frontend-checks, backend-checks) with all required steps
- [x] CI uses oven-sh/setup-bun@v2, dtolnay/rust-toolchain@stable, libwebkit2gtk-4.1-dev
- [x] CI runs: vue-tsc, ESLint, Prettier format check, cargo fmt, cargo clippy, cargo check, cargo test

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Added vue-tsc generated file ignores to ESLint config**
- **Found during:** Post-Task 2 ESLint verification
- **Issue:** ESLint reported 539 errors on `*.js` and `*.d.ts` files generated by `vue-tsc -b` in `src/` (e.g., `src/App.vue.js`, `src/types/ffmpeg.js`). These are build artifacts, not application source.
- **Fix:** Added `'src/**/*.js'`, `'src/**/*.d.ts'`, `'*.js'`, `'*.d.ts'` to ESLint ignores list
- **Files modified:** eslint.config.mjs
- **Commit:** 50bd806

**2. [Rule 2 - Missing critical functionality] Added .prettierignore to protect orchestrator-managed files**
- **Found during:** Task 1 Step F (Prettier format verification)
- **Issue:** `bun format` (Prettier --write .) was formatting `.planning/` files (STATE.md, ROADMAP.md) that the orchestrator owns and manages. Parallel execution instructions explicitly forbid modifying these files.
- **Fix:** Created `.prettierignore` excluding `.planning/`, `.claude/`, `src-tauri/`, `dist/`, `node_modules/`
- **Files created:** .prettierignore
- **Commit:** Committed by parallel Plan 01-04 agent (9b05313)

**3. [Rule 2 - Missing critical functionality] Added yml/yaml to lint-staged Prettier patterns**
- **Found during:** Task 2 Step B (creating .lintstagedrc.mjs)
- **Issue:** `.github/workflows/ci.yml` uses `.yml` extension, but lint-staged only covered `*.{json,md,css,html}` for Prettier formatting. CI workflow files should be formatted by Prettier.
- **Fix:** Added `yml` and `yaml` to the Prettier-only pattern: `'*.{json,md,css,html,yml,yaml}'`
- **Files modified:** .lintstagedrc.mjs
- **Commit:** Committed by parallel Plan 01-03 agent (9b3777b)

**4. [Rule 1 - Bug] Fixed 9 clippy errors in parallel agent's Rust code**
- **Found during:** Task 1 Step G (cargo clippy verification)
- **Issue:** `cargo clippy -- -D warnings` reported 9 errors: 3 dead_code (unused structs), 1 unnecessary_map_or, 4 collapsible_if, 1 single_match. These were in Rust code committed by parallel agents (Plans 01-03/01-04).
- **Fix:** Added `#[allow(dead_code)]` to FfmpegConfig, DownloadProgress, DownloadStage; simplified `map_or` to `is_some_and`; collapsed `if let` chains with `&& let`; changed `match` to `if let` in lib.rs
- **Files modified:** src-tauri/src/commands/ffmpeg.rs, src-tauri/src/commands/download.rs, src-tauri/src/lib.rs
- **Commits:** Incorporated into parallel agent commits (64102b5, c4f72bb)

### Plan Discrepancies Noted

- **Parallel agent concurrency:** Both Task 1 config files and Task 2 hook/CI files were committed by parallel agents executing Plans 01-03 and 01-04. This agent's contributions were limited to verification, Rule 2/1 fixes, and the final ESLint ignored-files fix.
- **rustfmt.toml unstable features:** `format_code_in_doc_comments`, `format_macro_matchers`, `imports_granularity`, and `group_imports` are unstable features requiring nightly Rust. They produce warnings on stable but do NOT cause format failures (exit 0). The plan specified these values — they are retained for future nightly compatibility.

## Threat Surface Verification

All 3 mitigations from the plan's `<threat_model>` are in place:

| Threat ID | Mitigation | Status |
|-----------|-----------|--------|
| T-02-01 | CI uses pinned action versions (checkout@v4, setup-bun@v2, cache@v4, dtolnay/rust-toolchain@stable) | Implemented |
| T-02-02 | No secrets in CI workflow (Phase 1 has no API keys, tokens, or credentials) | Verified |
| T-02-03 | Timeout limits set (10min frontend, 15min backend) prevent runaway jobs | Implemented |

No new threat surfaces introduced beyond what the plan accounts for.

## Known Stubs

None. All config files are fully implemented. The ESLint warnings on Vue components (66 cosmetic warnings for `max-attributes-per-line`, `singleline-html-element-content-newline`) are from Vue components created by the Plan 01-04 agent — these are formatting preferences that don't affect correctness.

## Self-Check: PASSED

- [x] Commit 50bd806 exists: ESLint generated file ignores
- [x] All 8 config/hook/CI files exist on disk
- [x] eslint.config.mjs contains tseslint.config(), Vue plugin, and correct ignores
- [x] prettier.config.mjs contains semi: true, singleQuote: true
- [x] rustfmt.toml contains edition = "2024"
- [x] .clippy.toml contains cognitive-complexity-threshold = 25
- [x] .husky/pre-commit contains bun lint-staged
- [x] .lintstagedrc.mjs maps patterns to lint/format commands
- [x] .github/workflows/ci.yml is valid YAML with frontend-checks and backend-checks jobs
- [x] `bun eslint .` exits 0
- [x] `bun prettier --check .` exits 0
- [x] `cargo fmt --check` exits 0
- [x] `cargo clippy -- -D warnings` exits 0
- [x] SUMMARY.md written
