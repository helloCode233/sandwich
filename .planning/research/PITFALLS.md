# Pitfalls Research

**Domain:** Desktop video batch processing (Tauri 2.x + Vue 3 + FFmpeg sidecar)
**Researched:** 2026-05-12
**Confidence:** MEDIUM

---

## Critical Pitfalls

### Pitfall 1: FFmpeg Process Never Terminates (Orphaned Sidecar)

**What goes wrong:**
A spawned FFmpeg process outlives the app window. The user closes the app while processing is in progress, the Tauri window disappears, but FFmpeg continues encoding in the background. On next launch, the output file is locked or a second FFmpeg instance fights for disk I/O. Worse: `child.kill()` is called but FFmpeg ignores the signal on certain platforms because Tauri's shell plugin sends SIGKILL immediately, which can leave corrupt output files instead of letting FFmpeg finalize the MOOV atom.

**Why it happens:**

- Tauri's `Child::kill()` maps to a forceful process termination — on Unix it sends SIGKILL, which FFmpeg cannot trap to write trailer atoms.
- Developers assume closing the window = process cleanup, but the Rust backend is a separate OS process and sidecars are spawned as independent child processes.
- The `while let Some(event) = rx.recv().await` loop in the sidecar handler holds a reference to the AppHandle; if the app exits without dropping this future, the spawned task is abandoned, not cancelled.

**How to avoid:**

1. Send SIGTERM/SIGINT first (via platform-specific kill signals), wait up to 5 seconds for FFmpeg to flush and exit, then send SIGKILL only as last resort.
2. Register an `onCloseRequested` handler that checks if any processing is active, warns the user, and if closure proceeds, calls a Rust command that gracefully shuts down all active FFmpeg children before allowing window destruction.
3. Store all active `Child` handles in a `Mutex<Vec<Child>>` managed Tauri state. On app exit (`on_window_event` with `WindowEvent::Destroyed`), iterate and terminate.
4. On Windows, use `taskkill /PID` with `/T` flag to kill the process tree (FFmpeg may spawn child encoder processes).

**Warning signs:**

- CPU fan stays on after closing the app
- Output files show 0 bytes or are locked by another process
- App cannot overwrite previous output on re-launch
- `ps aux | grep ffmpeg` shows lingering processes after app exit

**Phase to address:**
Phase 3 (batch processing core) — cancellation logic must be designed _before_ processing is built, not retrofitted.

---

### Pitfall 2: Stderr Parsing for Progress Is Brittle and Platform-Dependent

**What goes wrong:**
FFmpeg writes progress statistics to stderr by default, not stdout. The progress output format varies slightly between FFmpeg versions, operating systems, and codec choices. Developers parse a specific pattern like `time=` from the stderr stream, but:

- On some builds, progress lines are prefixed with `\r` (carriage return) for in-place updating
- The `-progress` pipe output uses `key=value\n` format which is stable, but `-progress -` (to stdout) mixes with other diagnostic output
- On Windows, FFmpeg may use `\r\n` line endings, while Unix uses `\n`
- Progress output includes fields like `out_time_us=` (microseconds) and `progress=continue/end` — parsing only `time=` misses the `progress=end` sentinel
- Large batch jobs can produce buffered output, delaying progress updates by seconds

**Why it happens:**
FFmpeg's default progress output was designed for human-readable terminal display, not machine parsing. The `-progress` flag was added later but still mixes with other log output unless redirected carefully. Developers underestimate how fragile string matching on a live encoding stream can be.

**How to avoid:**

1. Always use `-progress pipe:1 -nostats` and redirect stderr to a log file or `/dev/null`. Parse the structured `key=value\n` format, not the human-readable stderr progress.
2. Parse the `progress=continue` / `progress=end` sentinel keys to detect completion of each output stream.
3. Use `out_time_us=` (microseconds as integer) for precise progress calculation, not the formatted `time=HH:MM:SS.ms` string.
4. Set `-stats_period 0.5` for sub-second progress updates during long encodes.
5. Buffer partial lines — FFmpeg may write a partial line mid-buffer. Accumulate bytes until `\n` before parsing.
6. On Windows, set FFmpeg's stdout to binary mode or handle both `\r\n` and `\n`.
7. Validate that `fps=` and `speed=` fields are present and sensible; missing fields indicate a broken pipe or crashed encoder.

**Warning signs:**

- Progress bar jumps from 0% to 100%
- Progress percentage exceeds 100%
- `NaN` or blank values in parsed fields
- Progress stalls at "99%" for minutes (actually at muxing stage, `progress=end` never parsed)
- Different behavior on macOS vs Windows with same code

**Phase to address:**
Phase 2 (FFmpeg integration foundation) — progress parsing is the single most integration-critical component and must be built as a dedicated, tested Rust module before any UI progress bar.

---

### Pitfall 3: Sidecar Binary Naming Mismatch Causes Silent Bundle Failure

**What goes wrong:**
The FFmpeg binary is placed in `src-tauri/binaries/ffmpeg` but Tauri expects `src-tauri/binaries/ffmpeg-aarch64-apple-darwin` (or the appropriate target triple). The app builds and bundles without error, but at runtime `Command.sidecar("ffmpeg")` fails with "sidecar not found" — and the error message is often swallowed or logged to a place the user never sees. In development mode (`tauri dev`), the sidecar works because it falls back to a different resolution path. The bug only appears in production builds.

**Why it happens:**
Tauri 2.x requires sidecar binaries to follow the naming convention `{basename}-{target-triple}`. The `externalBin` entry in `tauri.conf.json` specifies only the basename (`"binaries/ffmpeg"`). At bundle time, Tauri looks for files matching `ffmpeg-*` in the `src-tauri/binaries/` directory. If none exist, the build succeeds (no error), but the sidecar is not included in the bundle. Developers test with `tauri dev` where the shell plugin may resolve the binary differently, never discovering the production bug.

**How to avoid:**

1. Add a build script (`build.rs` or npm script) that runs `rustc --print host-tuple` and renames the FFmpeg binary accordingly before every build.
2. Write an integration test that runs the actual production build (`tauri build --debug`) and verifies the sidecar spawns correctly.
3. In the Rust sidecar spawn code, `.expect("Failed to spawn sidecar")` is not enough — add explicit error handling that emits a user-visible error event with the sidecar name, searched paths, and resolution instructions.
4. For cross-platform builds (CI), pre-name binaries for all target triples and commit them all: `ffmpeg-x86_64-unknown-linux-gnu`, `ffmpeg-aarch64-apple-darwin`, `ffmpeg-x86_64-apple-darwin`, `ffmpeg-x86_64-pc-windows-msvc.exe`.
5. Verify the sidecar exists at startup: on app launch, attempt `Command.sidecar("ffmpeg").spawn()` with `-version` flag. If it fails, show an immediate modal error, don't let the user proceed to video import.

**Warning signs:**

- Sidecar works in `tauri dev` but fails in `tauri build`
- Build output contains no mention of the sidecar binary being bundled
- The bundled `.app`/`.exe` bundle size is suspiciously small (missing ~80MB FFmpeg)
- `Error: sidecar not found` in production but never in development

**Phase to address:**
Phase 1 (project setup and FFmpeg integration) — before any processing logic. This must be the first thing verified.

---

### Pitfall 4: Vue 3 Reactivity on Video Queue Arrays Causes UI Freezes

**What goes wrong:**
The video queue is stored as a `reactive([...])` array or a `ref([...])` with deeply reactive items. Each video item has metadata (duration, resolution, codec, filename, path, thumbnail data URL). Vue 3's reactivity system wraps every property of every item in a Proxy. When the queue reaches 50-100+ videos, any mutation (adding a progress update to one item, changing status from "queued" to "processing") triggers recursive dependency tracking across the entire array. The UI freezes for hundreds of milliseconds during simple queue operations. Worse: progress callbacks from Rust update the queue at 2Hz per video — with 10 videos processing concurrently, that's 20 reactive updates per second cascading through the entire array.

**Why it happens:**
This is a Vue 3 design default — deep reactivity is convenient for forms and small state but disastrous for large homogenous collections that receive high-frequency updates. Developers new to Vue 3 don't know about `shallowRef()` and `markRaw()`, or assume Vue's reactivity "just works" at any scale.

**How to avoid:**

1. Store the video queue as `shallowRef<VideoItem[]>([])`. This makes the array itself reactive (push, pop, splice trigger updates) but does NOT deeply proxy individual items.
2. Mark individual video items with `markRaw()` when adding to the queue. This prevents Vue from ever wrapping them in Proxies.
3. Update progress on individual items by replacing the item in the array (immutable update pattern): `queue.value[index] = { ...queue.value[index], progress: newProgress }`. This triggers reactivity at the array level only for that index.
4. Use `v-memo` on video queue list items with a key function that only re-renders when the specific item's relevant properties change (status, progress — not filename or duration).
5. Use `shallowReactive()` for the app-level state object that holds the queue reference, not `reactive()`.
6. Never store `ArrayBuffer`, `Uint8Array`, or thumbnail blobs in reactive state. Keep those in a separate `Map` keyed by video ID and access imperatively.
7. Consider virtual scrolling (`vue-virtual-scroller`) if the queue can exceed 100 items — DOM nodes for invisible list items still consume memory and rendering budget.

**Warning signs:**

- Adding a video to the queue takes >100ms for anything beyond 20 items
- Scrolling the queue list janks during active processing
- Memory profiler shows thousands of Proxy objects
- DevTools Performance tab shows `reactive` / `track` / `trigger` taking >10ms per operation
- Typing in a text field (seed alias) lags while processing is active

**Phase to address:**
Phase 4 (UI implementation) — but the data model design (shallowRef vs reactive) must be decided in Phase 1 to avoid a reactivity rewrite.

---

### Pitfall 5: Tauri Event Listener Leaks Accumulate on Component Mount/Unmount Cycles

**What goes wrong:**
A Vue component calls `listen('ffmpeg-progress', handler)` in `<script setup>` top-level scope (not inside `onMounted`). Each time the component is created (route navigation, conditional rendering, hot module reload), a new listener is registered. The old listener is never cleaned up. After mounting 5 times, the handler fires 5 times per event. The app appears to work initially but progressively slows down as duplicate listeners pile up. In extreme cases, the same progress update triggers 50+ DOM updates, freezing the UI entirely.

**Why it happens:**
Tauri's `listen()` returns an `UnlistenFn` Promise — it must be awaited and the returned function called on cleanup. Vue's `<script setup>` runs the top-level code once on component creation but provides no automatic cleanup. Developers see the example in docs showing `const unlisten = await listen(...)` at top level and copy it without realizing the cleanup requirement. The docs say "removing the listener is required if your listener goes out of scope" but this warning is buried.

**How to avoid:**

1. ALWAYS register Tauri event listeners inside `onMounted()` and store the unlisten function for cleanup in `onUnmounted()`.
2. Create a composable `useTauriEvent(eventName, handler)` that encapsulates this pattern:
   ```typescript
   export function useTauriEvent<T>(event: string, handler: (payload: T) => void) {
     let unlisten: (() => void) | null = null;
     onMounted(async () => {
       unlisten = await listen<T>(event, (e) => handler(e.payload));
     });
     onUnmounted(() => {
       unlisten?.();
     });
   }
   ```
3. For global listeners (not tied to a component lifecycle), register once in `App.vue` setup, NOT in child components.
4. In development, add debug logging: `console.log(`[Event] ${eventName} listener count: ${++counter}`)` to detect duplicates.
5. Use `watchEffect` cleanup functions only with explicit understanding that the effect re-runs on dependency changes — each re-run must call the previous unlisten first.

**Warning signs:**

- UI becomes progressively slower after navigating between views
- HMR during development causes duplicate progress bars
- Console shows 2x, 3x, 5x log messages for the same event
- Memory usage grows over time without video processing activity

**Phase to address:**
Phase 4 (UI implementation) — affects every component that communicates with Rust. The composable should be created in Phase 1 as shared infrastructure.

---

### Pitfall 6: FFmpeg License Non-Compliance in Distributed Binaries

**What goes wrong:**
The default static FFmpeg build (from gyandev, BtbN, or `brew install ffmpeg`) includes GPL-licensed encoders: libx264 (H.264), libx265 (H.265/HEVC), libaom (AV1). Bundling and distributing a GPL build of FFmpeg with a proprietary closed-source Tauri application violates the GPL. The entire application becomes subject to GPL source distribution requirements. If the user also downloads the FFmpeg binary separately (the "mere aggregation" defense), this may or may not hold — legal opinions differ and the GPL's stance on dynamic linking is aggressive.

Additionally, FFmpeg's `--enable-gpl` build may include codecs with patent licensing requirements (H.264, H.265) in jurisdictions that enforce software patents. Distribution to US/EU users without patent licenses from MPEG-LA creates separate liability.

**Why it happens:**
Developers grab the most feature-complete FFmpeg build without checking `ffmpeg -buildconf | grep gpl` to see which license configuration was used. The LGPL build (without `--enable-gpl` and `--enable-nonfree`) is functionally sufficient for this project's operation types (overlays, pixel shifts, GOP manipulation, metadata stripping, remuxing) — none of which require GPL-licensed encoders. H.264/H.265 encoding is used only for output, and FFmpeg's LGPL build can encode H.264 via the `libopenh264` encoder (BSD-licensed) — though quality is lower than libx264.

**How to avoid:**

1. Use an **LGPL-only** FFmpeg build. The BtbN/gyandev auto-builds provide `ffmpeg-master-latest-linux64-lgpl-shared` and similar LGPL variants. Verify with `ffmpeg -buildconf 2>/dev/null | grep "gpl\|nonfree\|version3"` — all should be absent.
2. Include FFmpeg's license text in the app bundle (Resources directory) and display it in an About/Licenses dialog accessible from the app menu.
3. Do NOT statically link FFmpeg libraries into the Rust binary — that triggers the viral clause. Use FFmpeg as a standalone sidecar binary invoked via process spawn, which is legally "mere aggregation" under both GPL and LGPL interpretations.
4. Do not redistribute H.264/H.265 encoder binaries. Offer output in royalty-free codecs (VP9 in WebM, AV1 in MP4/WebM) by default, with H.264 as an opt-in that warns about patent considerations.
5. Document in README that users can substitute their own FFmpeg build with additional codecs.
6. If users in patent-encumbered jurisdictions are a concern, provide download links to pre-built LGPL FFmpeg binaries hosted by a third party (not distributed in your app bundle), making it the user's choice to download.

**Warning signs:**

- `ffmpeg -buildconf` shows `--enable-gpl`
- You're bundling libx264/libx265 from the default build
- Your app is proprietary/closed-source
- Your legal/TOS review hasn't specifically looked at FFmpeg's license

**Phase to address:**
Phase 1 (project setup) — license choice is a Day 0 decision. Changing the FFmpeg build after features depend on GPL codecs is painful.

---

### Pitfall 7: UI Thread Blocking During FFmpeg File Discovery (Probe Phase)

**What goes wrong:**
When a user drags 50 video files into the app, the frontend iterates through them and calls `invoke('probe_video', { path })` for each file sequentially via `await`. Each probe takes 200-500ms (FFmpeg spawn overhead + file read). The UI freezes for 10-25 seconds while all probes complete sequentially. The queue panel shows nothing until ALL probes finish. The user thinks the app crashed.

**Why it happens:**
FFmpeg has no persistent daemon mode — every `ffprobe` call spawns a new process. The startup overhead is significant (loading shared libraries, parsing codec registries). Sequential `await` in JavaScript serializes the work. Even though each invoke is async, awaiting each one blocks the loop iteration. Meanwhile, the Vue UI is technically responsive but shows nothing useful because no results have been returned yet.

**How to avoid:**

1. Spawn all probe commands concurrently with `Promise.allSettled()` — FFmpeg I/O is parallelized at the OS level. Cap concurrency at ~8 to avoid file descriptor exhaustion.
2. Stream results to the UI: emit a Tauri event for each completed probe, and have the Vue queue component append items as they arrive. This gives instant visual feedback.
3. Use a Rust-side thread pool: accept the entire file list, spawn probes in parallel on the Rust side, emit `video-probed` events as each completes. The frontend only calls a single `invoke('probe_videos', { paths })` command.
4. Display probes in the queue immediately with a "scanning..." status, updating to show metadata as it arrives.
5. Skip probe for files that were previously probed (cache metadata by file path + modification timestamp in local storage or a SQLite DB).

**Warning signs:**

- "Not responding" / beachball cursor on bulk import
- Queue panel blank for many seconds after import
- CPU spikes to 100% but no UI progress
- Individual probe times under 500ms but total import takes 20+ seconds

**Phase to address:**
Phase 3 (batch processing core) — import pipeline design is part of the processing flow.

---

### Pitfall 8: Inadequate Progress Granularity — "Processing..." With No Cancellation

**What goes wrong:**
The UI shows "Processing video 3 of 50..." with a spinner. There is no per-video percentage, no estimated time remaining, no cancel button, and no way to skip a stuck video. The user has no idea if the app is working, frozen, or about to finish. If a video is corrupted and FFmpeg hangs on it, the entire batch stalls forever with no recovery path. The only option is to force-quit the app, which leaves corrupt partial outputs.

**Why it happens:**
Developers implement the happy path: queue videos, loop with `await process(video)`, update a counter. They treat processing as opaque — "FFmpeg is running, I'll wait until it exits." Adding fine-grained progress and cancellation requires per-video state management, progress event streaming, and abort controller integration, which feels like "extra" work rather than core functionality.

**How to avoid:**

1. **Three levels of progress**: (a) Overall batch progress (X of Y videos), (b) Per-video progress (percentage through current video's duration), (c) Per-operation progress (which operation in the seed chain is executing).
2. **Cancel everything**: A global "Stop" button that sends a cancel signal to the Rust backend. The Rust backend iterates active `Child` handles, sends SIGTERM to each FFmpeg process, and cleans up partial output files.
3. **Cancel one**: Each queue item has an individual cancel/skip button.
4. **Timeout**: Set an overall timeout per video (e.g., 10x the video duration at 1x speed). If FFmpeg takes longer than expected, auto-abort that video, mark it as "failed (timeout)", move to the next.
5. **Retry**: Failed videos get a "Retry" button. Failures are recorded with the FFmpeg stderr log attached for debugging.
6. **Pause/Resume queue**: Stop spawning new FFmpeg processes; let current ones finish; don't start next videos.

**Warning signs:**

- Progress indicator is a spinner, not a bar
- No "Cancel" button visible during processing
- Batch stops entirely when one video fails
- User reviews mention "I had to force quit"

**Phase to address:**
Phase 3 (batch processing core) — cancellation and progress are not "polish," they are core processing architecture.

---

### Pitfall 9: Corrupt Output Detection — FFmpeg Exit Code 0 Does Not Guarantee Valid Output

**What goes wrong:**
FFmpeg exits with code 0 (success) but the output file is:

- 0 bytes (disk full, but FFmpeg didn't detect the write failure)
- Truncated (MOOV atom missing from MP4 — plays in VLC but unseekable in most players)
- Encoded frames but output shows green/black corruption (decoder issue for specific pixel formats)
- Audio-only with no video track (filter chain silently dropped all video frames)
- Correct duration but all frames are duplicates (filter graph misconfiguration)
  The app marks the file as "completed successfully" and the user doesn't discover the problem until they try to use the video elsewhere.

**Why it happens:**
FFmpeg's exit code only indicates whether the process encountered a fatal error. Certain failure modes produce "valid" output streams that are technically correct but practically useless. The MOOV atom delay in MP4 (written at file close) means a crash during muxing leaves a file that some players can partially recover but is structurally corrupt. FFmpeg's `-xerror` flag helps but doesn't catch silent data corruption or empty streams.

**How to avoid:**

1. **Post-encode validation step**: After FFmpeg exits, probe the output file with ffprobe. Verify: `duration > 0`, `nb_streams > 0`, at least one video stream exists, file size > 10KB (for any reasonable video).
2. **MOOV atom relocation**: For MP4 output, use `-movflags +faststart` to move the MOOV atom to the beginning of the file during encoding. This catches MOOV issues immediately rather than at file close.
3. **Frame count verification**: `ffprobe -v error -count_frames -select_streams v:0 -show_entries stream=nb_read_frames -of default=nokey=1:noprint_wrappers=1 output.mp4` — verify it's > 0.
4. **Hash check**: For critical validation, decode N random frames from output and verify they differ from input frames (fingerprinting changed something). This is expensive but definitive.
5. **Use `-abort_on empty_output`** flag to make FFmpeg fail explicitly if it produces no output frames.
6. **Always use `-xerror`** to treat any error as fatal.
7. **Atomic output**: Write to a temp file first (`output.mp4.tmp`), validate it, then rename to the final filename. This prevents partial files from being treated as complete.
8. **Disk space check**: Before encoding, check available disk space is at least 2x the estimated output size (rule of thumb: output ≈ input \* compression ratio, but with filters it can be larger).

**Warning signs:**

- Output file size is suspiciously small (< 1% of input size)
- `ffprobe output.mp4` shows `Duration: N/A` or 0 streams
- VLC plays it but QuickTime/Windows Media Player doesn't
- Seek bar doesn't work in the output file
- Output is marked "success" but the user reports "file is broken"

**Phase to address:**
Phase 3 (batch processing core) — post-encode validation is a processing step, not a separate phase.

---

### Pitfall 10: Cross-Platform Path Handling Breaks FFmpeg Arguments

**What goes wrong:**
A file path like `/Users/name/Video Files/project (final).mp4` or `C:\Users\name\Videos\project.mp4` is passed to FFmpeg as an argument. On macOS/Linux, spaces break argument parsing if the path is not properly escaped. On Windows, backslashes in paths are interpreted as escape characters. The Tauri sidecar API passes arguments as an array, which handles basic quoting, but the path resolution between the frontend (JavaScript `file.path`), the Rust command layer, and FFmpeg's internal path handling creates multiple conversion points where things go wrong.

Additionally, Tauri's `resolveResource()` and `app.path().resolve()` use different base directories than what FFmpeg expects. Passing a Tauri resource path (e.g., `$RESOURCE/overlay.png`) directly to FFmpeg fails because FFmpeg has no knowledge of Tauri's virtual filesystem.

**Why it happens:**
Each layer (JS, Tauri IPC, Rust, OS, FFmpeg) has its own path format expectations. JavaScript uses forward-slash POSIX paths. Tauri resolves them to OS-native paths. FFmpeg uses its own internal path normalization. Non-ASCII characters (Unicode filenames) add encoding complexity — on Windows, paths may be UTF-16 but FFmpeg expects them in the system code page.

**How to avoid:**

1. Always pass paths to FFmpeg as absolute, OS-native paths resolved on the Rust side. Use `std::path::Path::canonicalize()` and `std::fs::canonicalize()` for normalization.
2. On Windows, ensure the path uses `\\?\` prefix for long paths (paths exceeding 260 characters with subdirectories in batch processing are common).
3. Wrap ALL file paths in double quotes when passing to FFmpeg: `format!("\"{}\"", path.display())`. Even though Tauri's argument array handles basic quoting, FFmpeg's filter graph syntax (which uses `:` and `,` as separators) can interpret unquoted paths containing these characters as graph separators.
4. For FFmpeg filter arguments that reference external files (e.g., `movie=filename.png`), use the `movie='filename.png'` quoting with single quotes inside the filter string.
5. Test with files containing: spaces, parentheses, Unicode characters (Chinese, Japanese, emoji in filenames), and the characters `: , ; ' " [ ]` in filenames.
6. Use `std::ffi::OsString` for command arguments on the Rust side to avoid UTF-8 conversion issues with non-UTF-8 paths on Unix.

**Warning signs:**

- FFmpeg errors: "No such file or directory" for files that clearly exist
- FFmpeg errors: "Unable to find a suitable output format" (interpreting path as format)
- Processing works on macOS but fails on Windows, or vice versa
- Files with spaces in path fail; files without spaces work
- Unicode filenames produce garbled output paths or crash

**Phase to address:**
Phase 2 (FFmpeg integration foundation) — path handling is fundamental to every FFmpeg invocation.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut                                                                    | Immediate Benefit                | Long-term Cost                                                                                                       | When Acceptable                                                                        |
| --------------------------------------------------------------------------- | -------------------------------- | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Calling FFmpeg via `Command::new("ffmpeg")` instead of `Command::sidecar()` | Avoids sidecar naming complexity | FFmpeg path resolution depends on system PATH; breaks on machines without ffmpeg installed; bundling impossible      | Never — defeats the purpose of a self-contained desktop app                            |
| Parsing FFmpeg stderr with regex for progress instead of `-progress pipe:1` | Faster initial implementation    | Breaks with every FFmpeg version update; locale-dependent (non-English error messages); fragile to concurrent output | Never — the pipe protocol is stable and designed for this                              |
| `reactive()` for video queue (unbounded deep proxy)                         | "Just works" for 5 videos        | Exponential overhead at 50+ items; forces reactivity rewrite late in project                                         | Only in prototype with <10 items and awareness that it will be rewritten               |
| Single-threaded sequential video processing                                 | Simple logic, easy debugging     | 50 videos at 2 min each = 100 min processing; user abandons app                                                      | Never in production; acceptable in Phase 2 while validating FFmpeg command correctness |
| Hardcoding `target-triple` in CI instead of `rustc --print host-tuple`      | One less build script            | Fails on new Apple Silicon CI runners; fails if contributor has different arch                                       | Never                                                                                  |
| Skipping `unlisten()` because "the component never unmounts"                | Less boilerplate                 | Hot reload in dev causes duplicate listeners; future route-based navigation breaks; memory leak in long sessions     | Never                                                                                  |

## Integration Gotchas

Common mistakes when connecting Tauri, Vue, and FFmpeg.

| Integration                     | Common Mistake                                                                                                       | Correct Approach                                                                                                                                                           |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tauri Shell → FFmpeg            | Using `command.execute()` (waits for completion, blocks) instead of `command.spawn()`                                | Use `command.spawn()` to get a `Child` handle; stream stdout/stderr events; track the child for cancellation                                                               |
| Rust → Vue events               | Emitting large objects (entire video metadata) as event payloads at high frequency                                   | Emit lightweight events with IDs; fetch full data via `invoke()` only when needed for display                                                                              |
| Vue → Rust invoke               | Wrapping every FFmpeg call in a Tauri command instead of using Rust-side orchestration                               | Tauri commands should be high-level ("process batch"), not low-level ("run ffmpeg with these args"). Rust manages FFmpeg processes internally                              |
| FFmpeg → file system            | Writing output directly to user-selected directory without temp file                                                 | Write to a temp location first, validate, then atomically move to destination. Prevents partial files at the output path                                                   |
| Tauri window → FFmpeg lifecycle | Assuming the Tauri `onCloseRequested` handler will always get a chance to clean up FFmpeg                            | The Rust backend needs its own shutdown signal handling (`tokio::spawn` with a CancellationToken). The frontend close guard is defense-in-depth, not the primary mechanism |
| FFmpeg build → Tauri bundle     | Using `brew install ffmpeg` or system package manager FFmpeg in development, then expecting it to work in production | Always use the same static FFmpeg build that will be bundled. Development should use the sidecar, never system PATH                                                        |
| Video preview → FFmpeg probe    | Calling ffprobe on every file every time the queue loads                                                             | Cache probe results (SQLite or JSON file keyed by path + mtime). A 50-video queue should probe instantly on second open                                                    |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap                                                  | Symptoms                                               | Prevention                                                                        | When It Breaks                               |
| ----------------------------------------------------- | ------------------------------------------------------ | --------------------------------------------------------------------------------- | -------------------------------------------- |
| Sequential ffprobe on import                          | UI freeze during bulk import                           | Concurrent probing with `Promise.allSettled()`, stream results                    | 10+ files                                    |
| Deeply reactive video queue                           | UI jank during progress updates                        | `shallowRef()` + `markRaw()`, immutable item replacement                          | 30+ items                                    |
| Accumulated Tauri event listeners                     | Progressively slower UI, duplicate handler invocations | `onUnmounted()` cleanup via composable                                            | 5+ component remounts                        |
| Single FFmpeg instance per batch                      | 50 videos = 2+ hours processing                        | Parallel processing (2-4 concurrent FFmpeg instances, balanced against CPU cores) | 10+ videos                                   |
| Unbounded event emission from Rust                    | Browser console flooded, IPC bottleneck                | Throttle progress events to 4Hz max per video; batch updates                      | 5+ concurrent videos with progress streaming |
| Full re-render of video queue on each progress update | Dropped frames in UI, input lag                        | `v-memo` with narrow dependency keys; virtual scrolling beyond 100 items          | 50+ items with 2Hz updates                   |
| Loading all video thumbnails into memory              | App memory > 1GB, swap thrashing                       | Generate thumbnails on-demand; cache to disk; unload invisible thumbnails         | 30+ videos with thumbnails                   |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake                                                                                    | Risk                                                                                                                  | Prevention                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Passing unsanitized user input (seed parameters) directly into FFmpeg filter graph strings | Filter injection: `params=1:2;rm -rf /` could execute arbitrary commands if FFmpeg's filter graph parser is exploited | Validate all seed parameters against strict type/range constraints on the Rust side BEFORE constructing filter strings. Only pass numbers and enumerated values, never free-form strings |
| FFmpeg reading from/writing to arbitrary paths based on user-supplied filenames            | Path traversal: a malicious filename like `../../../etc/passwd` could read or overwrite system files                  | Canonicalize all paths and verify they resolve within the expected directories. Reject paths containing `..` after canonicalization                                                      |
| Bundling a GPL-licensed FFmpeg build with proprietary software                             | Legal liability, forced source disclosure, DMCA takedown                                                              | LGPL-only build; separate download option; license audit before distribution                                                                                                             |
| Including libx264/libx265 encoders without patent license                                  | Patent infringement liability in US/EU/JP markets                                                                     | Default to VP9/AV1 (royalty-free); document H.264/H.265 as "user-provided build" option                                                                                                  |
| Using the same temp directory for all FFmpeg operations without cleanup                    | Disk exhaustion, information leakage between processing sessions                                                      | Use unique temp directories per batch session; clean up on app exit and at session start; enforce disk space minimum                                                                     |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall                                   | User Impact                                                                              | Better Approach                                                                                                                       |
| ----------------------------------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| "Processing..." with no progress details  | User abandons the app after 30 seconds of uncertainty                                    | Show: file-level progress bar, overall queue progress, estimated time remaining, current operation description                        |
| No cancel button during processing        | User must force-quit the app, loses all progress                                         | Always-visible stop button; individual file skip; confirmation dialog before abandoning in-progress work                              |
| Silent failures (FFmpeg error swallowed)  | User thinks processing succeeded but output is broken                                    | Surface FFmpeg stderr in an expandable error log per file; show red error badge alongside failed items                                |
| Queue lost on app restart                 | User painstakingly builds a queue of 30 files, closes app, everything gone               | Auto-save queue to disk (`localStorage` for MVP, SQLite for production); restore on launch                                            |
| No "processing" state preserved on close  | User closes app mid-batch, reopens to find no record of what was processing              | Persist batch state: which files were completed, which were in-progress, which queued. Offer to resume or restart                     |
| FFmpeg download modal is confusing/broken | First-time user stuck because FFmpeg download fails silently or takes too long           | Show download progress bar with speed/size ETA; provide manual download link as fallback; detect failure and offer retry              |
| Seed creation is opaque                   | User clicks "Generate Seed" and has no idea what operations were created or what they do | Show a human-readable summary of generated operations (e.g., "Adds a subtle ripple pattern for 30 frames, then shifts pixels by 2px") |
| Output directory confusion                | User can't find processed files                                                          | Show clickable "Open output folder" button; display last-used output path prominently; remember last directory                        |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **FFmpeg integration:** Can spawn and kill FFmpeg — but does it work with paths containing spaces, Unicode, and 200+ character paths?
- [ ] **Progress reporting:** Shows a percentage — but does it work when encoding speed is faster than realtime (>1x)? Does it handle `progress=end` correctly vs assuming 100% when the process exits?
- [ ] **Cancellation:** Has a cancel button — but does it clean up partial output files? Does it handle the case where FFmpeg has already finished muxing and is just flushing to disk?
- [ ] **Error handling:** Catches FFmpeg non-zero exit — but does it detect corrupt output with exit code 0? Does it surface the actual stderr to the user?
- [ ] **Sidecar bundling:** App runs in `tauri dev` — but does `tauri build` produce a bundle that works on a clean machine with no FFmpeg installed?
- [ ] **Window close:** Closes normally when idle — but does it prevent data loss when processing is active? Does it kill orphaned FFmpeg processes?
- [ ] **Large queue:** Works with 3 test videos — but does it work with 100 videos? Is the UI responsive? Does memory stay under 500MB?
- [ ] **Restart resilience:** Processes a batch — but if the app crashes mid-batch, can it resume? Is the queue persisted?
- [ ] **FFmpeg download:** Has an auto-download feature — but does it handle network errors? What if the download URL changes? What if the user is offline?
- [ ] **License compliance:** Includes FFmpeg — but is the license text included? Is the FFmpeg build definitely LGPL? Are there patent-encumbered codecs?

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall                                    | Recovery Cost | Recovery Steps                                                                                                                                                            |
| ------------------------------------------ | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Orphaned FFmpeg processes                  | LOW           | Kill via task manager / `pkill ffmpeg`. In code: on app startup, scan for ffmpeg processes belonging to this app (store PIDs in a session file) and terminate them        |
| Corrupt output detection misses a bad file | LOW           | Add a "Validate Output" button that re-probes processed files. User can manually re-process failed items                                                                  |
| Tauri event listener leak                  | LOW           | The composable pattern fixes this at the source. Migration: wrap all `listen()` calls in `useTauriEvent()`, one component at a time                                       |
| Vue reactivity rewrite                     | MEDIUM        | Convert `reactive([])` → `shallowRef([])`, add `markRaw()` to item factory. Touches many files but each change is mechanical. Schedule as dedicated refactor PR           |
| GPL FFmpeg bundled by mistake              | HIGH          | Rebuild with LGPL flags. Features using GPL-only codecs (libx264) must switch to LGPL-compatible alternatives (libopenh264) or be removed. May require feature regression |
| Sidecar naming mismatch in production      | MEDIUM        | Add build script with target-triple detection. Re-bundle. Write an automated smoke test that runs the production build and verifies sidecar spawning                      |
| Cross-platform path bugs                   | MEDIUM        | Add a comprehensive path test suite with all pathological filename patterns. Run on macOS and Windows CI. The fix is usually localized to the FFmpeg argument builder     |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall                                   | Prevention Phase                           | Verification                                                                                          |
| ----------------------------------------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------- |
| FFmpeg process orphaned on close (P1)     | Phase 3                                    | Kill app during processing → verify no ffmpeg in `ps aux`                                             |
| Stderr progress parsing fragility (P2)    | Phase 2                                    | Run test suite against multiple FFmpeg versions; parse known-good progress output fixtures            |
| Sidecar naming mismatch (P3)              | Phase 1                                    | `tauri build --debug` on clean VM → launch → verify ffmpeg -version succeeds                          |
| Vue reactivity freeze on large queue (P4) | Phase 1 (data model), Phase 4 (UI)         | Import 100 videos → verify queue operations <50ms, UI scrolls at 60fps during processing              |
| Event listener leak (P5)                  | Phase 1 (composable), Phase 4 (components) | Mount/unmount processing view 10x → verify only 1 handler fires per event                             |
| FFmpeg license non-compliance (P6)        | Phase 1                                    | `ffmpeg -buildconf` shows no --enable-gpl; license file in bundle; legal review                       |
| UI blocking during probe import (P7)      | Phase 3                                    | Import 50 files → verify first item appears in <1s, all done in <5s                                   |
| Inadequate progress/cancellation UX (P8)  | Phase 3                                    | Start batch of 10 → verify per-file and overall progress, cancel within 2s, partial output cleaned up |
| Corrupt output not detected (P9)          | Phase 3                                    | Generate known-bad output (kill ffmpeg mid-encode) → verify app marks as failed, not success          |
| Cross-platform path handling (P10)        | Phase 2                                    | Test paths: spaces, Unicode, 300+ chars, special chars `:,';[]` → all succeed on macOS and Windows    |

## Sources

- **Tauri 2.x Sidecar Documentation** — Context7 (/websites/v2_tauri_app, /tauri-apps/tauri-docs). Sidecar naming convention requires `{name}-{target-triple}` suffix; `externalBin` only references base name; `Command.sidecar()` spawns from bundled binaries directory. Confidence: HIGH.
- **Tauri 2.x Event System** — Context7 (/websites/v2*tauri_app). `listen()` returns `Promise<UnlistenFn>`; unlisten required on listener scope exit; event names restricted to alphanumeric, `-`, `/`, `:`, `*`. Confidence: HIGH.
- **Tauri 2.x Window Close Guard** — Context7 (/websites/v2_tauri_app). `onCloseRequested()` with `event.preventDefault()` for close interception; requires `unlisten()` on component unmount. Confidence: HIGH.
- **Tauri 2.x Async Commands** — Context7 (/websites/v2_tauri_app). Commands should return `Result` for frontend error handling; `tauri::async_runtime::spawn()` for long-running tasks; async-compatible locks required across await points. Confidence: HIGH.
- **Tauri 2.x Resource Paths** — Context7 (/websites/v2_tauri_app). `resolveResource()` in JS; `app.path().resolve()` in Rust; paths follow `tauri.conf.json` bundle resource mapping. Confidence: HIGH.
- **FFmpeg Progress Protocol** — Context7 (/websites/ffmpeg_ffmpeg-all). `-progress pipe:1` outputs key=value format; `progress=continue/end` sentinel; `-stats_period` controls update frequency; `out_time_us=` for machine-parseable time. Confidence: HIGH.
- **FFmpeg Error Detection** — Context7 (/websites/ffmpeg_ffmpeg-all). `-xerror` for immediate exit on error; `-abort_on empty_output` for empty stream detection; `-max_error_rate` for decode failure threshold. Confidence: HIGH.
- **FFmpeg License** — FFmpeg official website (ffmpeg.org/legal.html). LGPL vs GPL builds; `--enable-gpl` includes GPL codecs (x264, x265); LGPL build is safe for proprietary distribution with proper notice. Confidence: MEDIUM (legal interpretation requires lawyer review; technical build configuration is HIGH).
- **Vue 3 Reactivity** — Context7 (/vuejs/vue). `shallowRef()` for top-level reference tracking without deep proxy; `markRaw()` to opt out of reactivity; `shallowReactive()` for root-level state. Confidence: MEDIUM (specific large-list performance numbers are from ecosystem knowledge, not official Vue benchmarks).

---

_Pitfalls research for: Tauri 2.x + Vue 3 + FFmpeg desktop video batch processor_
_Researched: 2026-05-12_
