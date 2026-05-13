# Phase 2 Security Verification

**Phase:** 02 — Rust Backend
**Plans Verified:** 02-01, 02-02, 02-03, 02-04
**Threats Closed:** 24/24
**ASVS Level:** 1
**Verification Date:** 2026-05-13
**Block On:** high severity

## Summary

All 24 declared threats from the Phase 2 threat register have been verified. 13 mitigated threats have their mitigations confirmed present in the implementation. 11 accepted threats are documented below with rationale. Zero open threats.

## Threat Verification

### Mitigated Threats

| Threat ID | Category | Component | Disposition | Evidence |
|-----------|----------|-----------|-------------|----------|
| T-02-01 | Tampering | models (serde deserialization) | CLOSED | All structs in `models/seed.rs:5,20,36`, `models/video.rs:5,20,39`, `models/batch.rs:4,16,32,42` derive `Deserialize`. `Operation.params` is `serde_json::Value` (bounded depth 128 by serde default). Malformed JSON returns `serde_json::Error`. |
| T-02-04 | Tampering | filters.rs (FFmpeg argument injection) | CLOSED | All numeric params extracted via type-safe methods: `as_f64()` (filters.rs:12-13,102,107), `as_i64()` (filters.rs:46-47), `as_u64()` (filters.rs:67,78). Every numeric value is clamped to safety range before string formatting: opacity `.clamp(0.01, 0.15)`, dx/dy `.clamp(-3, 3)`, interval `.max(15)`, gop_size `.clamp(12, 250)`, db `.clamp(-2.0, 2.0)`, factor `.clamp(0.98, 1.02)`. |
| T-02-05 | Denial of Service | executor.rs (stderr pipe deadlock) | CLOSED | `child.iter()` called at executor.rs:110 drains stderr continuously. Comment at executor.rs:101-103 confirms Pitfall 1 mitigation. |
| T-02-09 | Denial of Service | executor.rs (cancel flag on ARM) | CLOSED | All `AtomicBool` operations use `Ordering::SeqCst`: `cancel_flag.load(Ordering::SeqCst)` at executor.rs:56,112; batch.rs:147,211. `flag.store(true, Ordering::SeqCst)` at batch.rs:263. Follows download.rs Phase 1 pattern (download.rs:83,107,361,449). |
| T-02-10 | Tampering | seed.rs (alias input) | CLOSED | `rename_seed` at seed.rs:155-157 validates `new_alias.trim().is_empty()` and returns error "Alias cannot be empty". No other mutation surfaces accept unconstrained alias strings. |
| T-02-11 | Tampering | seed.rs (seed_id injection) | CLOSED | Exact ID match (`s.id == seed_id`) used in `rename_seed` (seed.rs:164), `delete_seed` (seed.rs:185 via `retain`), and `copy_seed` (seed.rs:212-213). `delete_seed` additionally checks `len_before != len_after` (seed.rs:184-188) for not-found detection. IDs are UUIDs — never interpolated into paths or commands. |
| T-02-15 | Denial of Service | queue.rs (bounds check) | CLOSED | `remove_from_queue` at queue.rs:40-46 validates `index >= app_state.queue.len()` before `.remove(index)`. Error message includes current queue length for debuggability. |
| T-02-16 | Data Loss | seed.rs/queue.rs (mutex poisoning) | CLOSED | 23 occurrences of `.map_err(\|e\| format!("Lock error: {}", e))` across commands/seed.rs (7), commands/queue.rs (4), commands/batch.rs (10), commands/import.rs (2). All lock acquisitions propagate poison errors as `Result::Err`. No `unwrap()` on lock results. |
| T-02-17 | Spoofing | import.rs (extension bypass) | CLOSED | Extension allowlist (`SUPPORTED_EXTENSIONS`) defined at import.rs:17 with 7 extensions. Checked with `.to_lowercase()` before ffprobe spawn at import.rs:35-47. ffprobe D-14 validation at probe.rs:39-43 (exit code) and probe.rs:50-54 (video stream existence) provides defense-in-depth. |
| T-02-20 | Denial of Service | batch.rs (disk space) | CLOSED | `check_disk_space_for_output()` at import.rs:100-121 calls `fs2::available_space()`. Warning emitted via "low-disk-space" event when `< 100_000_000` bytes (import.rs:109-116). |
| T-02-21 | Data Loss | lib.rs (startup load failure) | CLOSED | Store loading at lib.rs:61-68 and lib.rs:74-82 uses `if let Ok(...)` pattern — deserialization failures silently fall through, leaving `AppState::default()` (empty vecs). No panic, no data corruption. |
| T-02-22 | Elevation of Privilege | batch.rs (output path injection) | CLOSED | `make_output_path()` at executor.rs:184-213 constructs filenames from `source_path.file_stem()` and `source_path.extension()` (Path API, not user input). Seed alias combined is alphanumeric + underscore (D-04 timestamp format). Collision suffix uses Path API `.exists()` and `.join()`. |
| T-02-24 | Denial of Service | batch.rs (global cancel static leakage) | CLOSED | `BATCH_CANCEL` storage cleared to `None` at: batch.rs:119 (empty queue early exit), batch.rs:225 (batch completion — both success and cancel paths). Fresh `Arc::new(AtomicBool::new(false))` at batch.rs:97 per batch. |

### Accepted Threats

| Threat ID | Category | Component | Rationale |
|-----------|----------|-----------|-----------|
| T-02-02 | Data Loss | store loading in lib.rs setup | Store deserialization failures handled in Plan 04 with graceful fallback to `Default::default()` (empty state). No data is persisted in Plan 01 — types and scaffolding only. |
| T-02-03 | Tampering | probe.rs (ffprobe filepath) | Desktop app: file paths originate from user's file system via OS file picker. User already has filesystem access. Multi-tenant path traversal is not applicable. |
| T-02-06 | Denial of Service | executor.rs (disk space) | Executor assumes caller validates space. `import_video` (Plan 04) calls `check_disk_space_for_output()` which checks `fs2::available_space()` and emits "low-disk-space" warning at <100MB. No hard limit per D-13. |
| T-02-07 | Elevation of Privilege | executor.rs (FFmpeg codec exploits) | FFmpeg runs with same user privileges as the app. Attack surface limited to files user intentionally imports. Standard desktop app risk — no sandboxing beyond OS default. |
| T-02-08 | Information Disclosure | executor.rs (ffprobe output) | ffprobe metadata (duration, resolution, codec) is non-sensitive technical data. All processing is local — no data exfiltration surface. |
| T-02-12 | Denial of Service | seed.rs (store file growth) | No explicit seed count limit. Each seed ~500 bytes; 10,000 seeds ~5MB — well within desktop tolerance. Phase 3 UI will provide pagination/scroll for large lists. |
| T-02-13 | Denial of Service | queue.rs (store file growth) | Queue entries ~200 bytes each. 10,000 entries ~2MB. Bounded by user's patience in importing videos. Desktop app tolerance. |
| T-02-14 | Data Loss | seed.rs/queue.rs (crash during save) | tauri-plugin-store `.save()` assumed atomic per Assumption A3. Plan 04 startup load uses `if let Ok` graceful fallback on partial JSON. |
| T-02-18 | Tampering | import.rs (path traversal) | Desktop app: file paths come from user's filesystem. User already has full filesystem access. Path traversal is not a meaningful threat in single-user desktop context. |
| T-02-19 | Denial of Service | batch.rs (no rate limiting) | Processing is CPU-intensive by nature — no external requests. UI remains interactive via async commands. User controls batch size by queue management. |
| T-02-23 | Repudiation | batch.rs (batch history) | No audit log of completed batches. Output files themselves serve as the record of what was processed. Acceptable for v1. |

## Unregistered Flags

None. All new attack surface from the three summary files (02-02-SUMMARY.md: FFmpeg utilities; 02-03-SUMMARY.md: 8 IPC commands + 2 store files; 02-04-SUMMARY.md: 4 new commands + state wiring) is covered by the registered threat models T-02-03 through T-02-24.

## Verification Method

Each `mitigate` threat was verified by grepping the exact mitigation pattern in the implementation files cited in the plan's mitigation plan. Evidence locations use file:line references confirmed via direct read.

Each `accept` threat was assessed against the documented rationale and confirmed to match the plan's disposition.

No `transfer` threats exist in this phase.
