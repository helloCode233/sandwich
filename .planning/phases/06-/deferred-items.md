# Deferred Items - Phase 6

## Pre-existing Compilation Errors (Discovered 06-03)

These errors prevent `cargo test` from running in the worktree. They appear to originate from Plan 06-01 (data model extensions) and are present at the base commit `0af78ae`.

| File | Line | Error | Type |
|------|------|-------|------|
| `src/models/batch.rs` | 191 | `unresolved import md5` — `md5` crate not in Cargo.toml as dependency | Missing dependency |
| `src/commands/batch.rs` | 262 | `mismatched types: expected Vec<FileSuccess>, found Vec<String>` | Type mismatch |
| `src/ffmpeg/executor.rs` | 170 | `missing field seed_alias in initializer of PerFileProgress` | Missing field |

**Impact:** Prevents verification of Plan 06-03 tests via `cargo test`. Filter builder code was verified through manual review against established patterns and acceptance criteria.
