---
status: partial
phase: 01-foundation
source: [01-VERIFICATION.md]
started: 2026-05-13T12:30:00+08:00
updated: 2026-05-13T12:30:00+08:00
---

## Current Test

[awaiting human testing]

## Tests

### 1. Tauri window launch
expected: Running `bun tauri dev` opens a 1200x800 window titled "Sandwich" with the FFmpeg detection UI visible
result: [pending]

### 2. FFmpeg detection (PATH found)
expected: When FFmpeg >= 4.0 is in PATH, the UI shows green checkmark with version number, then auto-transitions to the welcome placeholder page after 1.5 seconds
result: [pending]

### 3. FFmpeg download flow
expected: When FFmpeg is not found, user sees warning icon + "not found" message + download button. Click opens native directory picker, download begins with real-time progress (percentage + size/speed), cancel works, post-download verification succeeds
result: [pending]

### 4. Store persistence
expected: After FFmpeg path is verified and stored via tauri-plugin-store, restarting the app shows "FFmpeg found" without re-detecting from PATH
result: [pending]

### 5. GitHub Actions CI
expected: Pushing to any branch triggers the CI workflow — frontend-checks (vue-tsc + ESLint + Prettier + Vitest) and backend-checks (cargo fmt + clippy + check + test) both pass
result: [pending]

### 6. Husky pre-commit hook
expected: `git commit` triggers lint-staged via husky, which runs ESLint + Prettier on staged TS/Vue files and exits clean
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps
