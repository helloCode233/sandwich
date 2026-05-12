# Feature Research

**Domain:** Desktop Video Fingerprint Modification Tool
**Researched:** 2026-05-12
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| FFmpeg auto-detection on launch | Desktop tools handling media must work without manual FFmpeg PATH setup; HandBrake, Shutter Encoder both bundle or auto-detect | LOW | Read PATH, check `ffmpeg -version` via `std::process::Command`. Store found/not-found state in Tauri managed state. |
| FFmpeg one-click download when missing | If a tool needs FFmpeg and it's absent, users expect a built-in downloader, not instructions to "install FFmpeg manually" | MEDIUM | Download static FFmpeg binary (~80MB) via `reqwest` with progress. Store in Tauri app data directory (`app_data_dir()`). Platform-specific binary selection. Requires network error handling and checksum verification. |
| Video file import (drag-and-drop) | Universal expectation for desktop media tools; users drag files from Finder/Explorer directly into the app | LOW | Enable Tauri window drag-drop. Parse drop event for file paths. Filter to supported video extensions. Add to queue state. Tauri's `on_file_drop_event` or frontend HTML5 drag-drop with disableDragDropHandler. |
| Video file import (file picker fallback) | Backup for users who prefer browsing; also needed when drag-drop is finicky on certain platforms | LOW | Use `tauri-plugin-dialog` `open()` with video file filters. Returns selected paths for queue insertion. |
| Video queue display and management | Users need to see what's queued, remove items, verify order before processing | LOW | Vue reactive list. Each queue item shows: filename, duration, resolution, file size. Remove button per item. Clear-all action. |
| Queue reordering | Processing order matters when testing; users want control over execution sequence | LOW | Drag-to-reorder within the queue list via HTML5 sortable or Vue draggable library (e.g., `vuedraggable`). Updates internal array index. |
| Seed list management (view, rename, delete, duplicate) | Seeds are the core abstraction; users need CRUD or it's not a real tool | LOW | Vue list component with inline actions. Rename via inline edit. Delete with confirmation dialog. Duplicate creates a copy with "(copy)" suffix. All persisted to local storage or SQLite. |
| Auto-generate seed (randomized operation chain) | This IS the product. Without seed generation, the tool is just a video queue with manual FFmpeg commands | HIGH | Core algorithm: given a random seed value, deterministically generate a chain of operations. Each operation has: type (from 7 supported), start_frame, frame_count, params. Must respect safety constraints (transparency <= 0.15, pixel shift <= 3px, drop interval >= 15 frames). Requires careful random distribution logic to ensure variety while staying within safe visual bounds. |
| Apply one seed to all queued videos (batch processing) | The "batch" in "batch fingerprint modifier"; applying a single seed across a queue is the fundamental workflow | MEDIUM | Iterate queue items. For each: build FFmpeg filtergraph from seed operations, spawn FFmpeg child process via `tokio::process::Command`, pipe stderr for progress, write output to target directory. All async via Tauri commands with event emission for progress. |
| Per-file and overall progress tracking | Video processing can take minutes to hours; users need to know what's happening and how much is left | MEDIUM | Parse FFmpeg stderr for `time=` or `out_time_ms=` patterns to get current frame/time. Emit Tauri events: `processing-progress` with {fileIndex, fileName, percent, time}. Overall progress = average across files. Display progress bars in queue. |
| Processing cancellation | Long-running operations need a stop button; users make mistakes or want to abort | MEDIUM | Kill the spawned FFmpeg process on cancel. Clean up partial output file. Must handle the Rust side (tokio `Child::kill()`) triggered by a Tauri command from the frontend cancel button. |
| Output directory configuration | Users need to control where processed files go; defaulting to source directory is destructive | LOW | Directory picker via `tauri-plugin-dialog` `open({directory: true})`. Persist last-used directory to app config/state. Default to OS-appropriate output folder (e.g., `~/Videos/sandwich-output/`). |
| Output file naming convention | Without clear naming, users lose track of which output came from which input with which seed | LOW | Default template: `{original_name}_{seed_alias}.{ext}`. Show preview of output names in queue. Handle name collisions (append number suffix). |
| Error handling and per-file failure reporting | Processing can fail per-file (corrupt source, disk full, unsupported codec); the tool must report what failed and why without crashing the whole batch | LOW | Catch FFmpeg non-zero exit codes. Display error icon + message on failed queue items. Continue processing remaining files. Show summary at end: "3 succeeded, 1 failed". |

### Differentiators (Competitive Advantage)

Features that set the product apart from manual FFmpeg scripting or general-purpose video tools.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Deterministic randomized seed generation | vs writing FFmpeg filter chains by hand: one click generates a complete multi-operation recipe with all parameters randomized. Users don't need to know FFmpeg filter syntax or safe parameter ranges. This alone makes the tool 10x faster than manual scripting. | HIGH | Core IP. Random seed value drives `StdRng::seed_from_u64()` which deterministically generates operation types, frame ranges, and parameters within safety bounds. Same seed always produces same recipe (reproducible results). The random distribution logic must avoid producing too-similar seeds (minimum "distance" between recipes). |
| Operation chain catalog (7 types) | vs single-filter manual runs: the tool chains multiple operation types automatically. Each seed produces 3-7 randomly selected operations from: math overlay, pixel jitter, frame drop, GOP change, metadata wipe, audio tweak, remux. No single operation alone can defeat fingerprinting — the combination creates exponentially many unique variants. | HIGH | Each operation type requires a FFmpeg filtergraph builder module:
- **Math overlay**: `geq`/`overlay` with `drawtext`-like shapes (ripple, stripe, concentric) at random positions with alpha ≤ 0.15
- **Pixel jitter**: `translate` filter with random x/y shift ≤ 3px per frame block
- **Frame drop**: `select` + `setpts` to drop frames at random intervals ≥ 15 frames
- **GOP change**: `-g` parameter randomization, force keyframe interval changes
- **Metadata wipe**: `-map_metadata -1` + `-map_chapters -1`, strip all metadata tracks
- **Audio tweak**: `atempo` (0.99-1.01), `volume` (±0.5dB), or `aphaser` with minimal wet mix
- **Remux**: `-c copy` with container format change (mp4↔mkv↔mov) — no re-encode |
| Safety constraint engine | vs manual scripting where you can ruin a video: the tool enforces visual safety bounds. Transparency ≤ 0.15, pixel shift ≤ 3px, frame drop interval ≥ 15. Users get "invisible" modifications that defeat fingerprinting without degrading perceived quality. | MEDIUM | Parameter space constrained in code. Each operation's random param generation has hardcoded safe ranges. No user-configurable overrides in v1 (reduces support burden, prevents foot-gun). |
| Instant preview of queued videos | vs command-line where you can't see what you're processing: thumbnail + metadata display for each video in the queue. Users verify they've loaded the right files before committing to a potentially hours-long batch. | LOW | Generate thumbnail via FFmpeg (`-ss 5 -vframes 1`). Read metadata via `ffprobe` (duration, resolution, codec, bitrate). Display in queue item card. Cache thumbnails to temp directory. |
| Processing log and history | vs FFmpeg CLI where output scrolls off screen: persistent, searchable log of every processing run. Shows: timestamp, seed used, operations applied, FFmpeg command generated, output file, duration, success/failure. Users can reference what they did or debug issues. | MEDIUM | Structured log entries stored alongside app data. Log viewer component with filtering (by date, seed, status). Export log as JSON for sharing/debugging. |
| Seed export/import | vs recreating complex filter chains from memory: share seed recipes as portable JSON files. One user discovers a particularly effective seed → export → share with team → import and use. | LOW | Export: serialize seed object to JSON file via Tauri save dialog. Import: parse JSON, validate structure, add to seed list. Seed JSON includes version field for forward compatibility. |
| One-click workflow (no FFmpeg knowledge) | vs learning FFmpeg filter syntax: the entire workflow is "generate seed → queue videos → click process." Zero FFmpeg knowledge required. The generated FFmpeg command is visible (transparency) but not required. | LOW | This is UX design, not implementation complexity. The frontend needs a clean flow: left panel (seeds) → right panel (queue) → bottom (output config) → big "Process All" button. |
| Batch processing summary with diff hints | vs manual verification: after processing, show a summary of what changed per file (operations applied, output size delta, duration delta). Users get confidence that modifications happened without watching every output. | MEDIUM | Collect FFmpeg stats from stderr parsing and `ffprobe` post-processing. Compare input/output metadata. Display in a dismissible summary panel. |
| Cross-platform (macOS, Windows, Linux) | vs scripts tied to one OS: Tauri builds produce native binaries for all three desktop platforms. Same workflow, same features, no shell script porting needed. | LOW | Inherited from Tauri. Requires testing on each platform (FFmpeg binary paths differ, file permissions differ). |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem attractive but would bloat v1 or cause significant issues.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Real-time video preview during processing | "I want to watch my video being modified live" | Requires piping half-processed video frames back from FFmpeg to the UI, massively complex. FFmpeg doesn't stream intermediate output cleanly. Would block the batch processing pipeline and add enormous complexity for near-zero real value. | Generate a short preview clip (first 5 seconds) after processing. User can spot-check the output. |
| Manual filter chain editor (drag-and-drop filter graph) | "I want full control over what operations are applied" | Contradicts the core value prop of automatic seed generation. Building a visual filter graph editor is a massive engineering effort (think: Blender nodes, DaVinci Fusion). Users who want this level of control should use existing tools or FFmpeg CLI directly. | Seed generation with constraints that produce effective results. Export generated FFmpeg command for users who want to inspect/tweak manually in CLI. |
| Video editor (timeline, cuts, transitions) | "While I'm here, can I also trim/edit my videos?" | Scope creep into full video editing. Timeline editing requires frame-accurate seeking, complex UI, undo/redo, preview rendering — an entire product category. Adds months to development. | LosslessCut integration workflow: use LosslessCut for trimming, then use this tool for fingerprint modification. |
| Different seeds per video in a batch | "I want video A to use seed X, video B to use seed Y" | Explodes the batch processing state machine. Now you need seed-video mapping, complex queue state management, and a UI for the mapping. Out of scope per PROJECT.md. Can be added in v2 when the core workflow is validated. | Run multiple batches sequentially: apply seed X to subset A, then seed Y to subset B. |
| Cloud encoding / remote processing | "My computer is slow, can you process in the cloud?" | Requires auth, billing, upload infrastructure, queue management, cloud FFmpeg execution, download management, and ongoing infrastructure costs. Transforms a desktop app into a SaaS platform. Not viable for v1. | The tool is designed for local processing. FFmpeg is efficient on modern hardware. For very slow machines, seed generation with fewer operations reduces processing time. |
| Audio-only processing mode | "I have audio files I want to process too" | Audio fingerprinting is a fundamentally different domain from video fingerprinting (different operations, different detection mechanisms, different tools). Supporting audio-only files would dilute the video focus and add complexity for a different user segment. | Scope to video files with audio tracks. Audio tweaks are applied to the audio track within video processing. |
| Plugin system for custom operations | "Let users write and share custom operation modules" | Designing a safe, sandboxed plugin API is months of work (security, versioning, API stability, distribution). Premature for v1 when the 7 built-in operations cover the fingerprint modification surface area well. | Add new operation types as built-in features based on validated user need. Re-evaluate plugin system for v3+. |
| AI-powered "uniqueness score" prediction | "Tell me how likely this seed is to bypass platform detection" | Platform fingerprinting algorithms are black boxes that change constantly. Any score would be speculative, misleading, and quickly outdated. Building this would require reverse-engineering proprietary systems — legally risky and technically infeasible. | Provide "what changed" summary (operations applied, metadata diff). Users develop intuition about effective seeds through usage. |
| Background processing (system tray, minimize to tray) | "Let me start a batch and close the window" | Adds system tray integration, background process management, notification architecture, and edge case handling (what if user quits?). Valuable but not v1-critical and adds significant complexity. | Keep the window open during processing. Tauri already keeps the app alive while processing runs. Add minimize-to-tray in v1.x. |
| Project files / workspace persistence | "Let me save my current queue and seeds as a project" | Adds file format design, save/load dialogs, dirty-state tracking, auto-save, and backward compatibility concerns. Adds weeks of work for a saving pattern that few users will need in v1. | Auto-persist state to local storage / SQLite. Users don't "save" — the app remembers their last state on relaunch. Add explicit project files in v2. |

## Feature Dependencies

```
Batch Processing
    ├──requires──> FFmpeg Execution (spawn, monitor, kill)
    │                   ├──requires──> FFmpeg Detection
    │                   │                   └──falls_back_to──> FFmpeg Download
    │                   └──requires──> Progress Parsing (stderr parsing)
    ├──requires──> Seed Generation (produces filtergraph)
    │                   └──requires──> Operation Chain Builder (7 operation types)
    │                           └──requires──> Safety Constraint Engine
    ├──requires──> Video Queue (list of input files)
    │                   ├──requires──> Video Import (drag-drop + file picker)
    │                   └──enhances──> Video Preview (thumbnails)
    └──uses──> Output Management (directory + naming)

Processing Log ──consumes──> Batch Processing results
Seed Export/Import ──operates_on──> Seed Management
Batch Summary ──consumes──> Batch Processing results + ffprobe data
```

### Dependency Notes

- **Batch Processing requires FFmpeg Execution:** Cannot process videos without FFmpeg binary available and spawnable. FFmpeg detection must complete before "Process All" button is enabled.
- **Seed Generation requires Operation Chain Builder:** Seed randomness drives operation selection and parameterization. Each operation type needs its own FFmpeg filtergraph builder. The safety constraint engine bounds all random parameters.
- **Video Queue requires Video Import:** No files in queue until import mechanism works. Both drag-drop and file picker are co-required for robustness across platforms.
- **Progress Parsing enhances Batch Processing:** Not technically required (batch can run without progress), but users cannot see what's happening without it. Strongly coupled to FFmpeg stderr output format.
- **Processing Log consumes Batch Processing results:** Logging is a passive consumer — it records what happened. Can be added after batch processing works. Not a blocking dependency.
- **Video Preview enhances Video Queue:** Thumbnail generation is optional for queue functionality. Queue works fine with just filename/metadata.

## MVP Definition

### Launch With (v1)

Minimum viable product — what's needed to validate the "one-click batch deduplication" concept.

- [ ] **FFmpeg detection + download** — Zero-config prerequisite for all video processing. Without this, the app is a non-functional shell.
- [ ] **Seed generation (auto-randomized chains)** — The core value proposition. Must produce varied, safe operation chains from a single click.
- [ ] **Seed list management (view, rename, delete, duplicate)** — Users need to build a library of effective seeds.
- [ ] **Video import (drag-drop + file picker)** — Getting videos into the tool. Drag-drop is the UX differentiator; file picker is the reliability fallback.
- [ ] **Video queue with basic management (remove, clear)** — Users need to see and manage what they're processing. Reordering is nice-to-have but can be deferred if the queue processes in insertion order.
- [ ] **Batch processing (one seed, many videos)** — The core workflow. Spawn FFmpeg per file, emit progress events, handle cancellation.
- [ ] **Progress tracking (per-file progress bars)** — Users need visibility into long-running operations.
- [ ] **Output directory + naming** — Processed files must go somewhere predictable with identifiable names.
- [ ] **Error handling (per-file failures, batch continues)** — Robustness: one corrupt file shouldn't kill the entire batch.

### Add After Validation (v1.x)

Features to add once the core loop works and users validate the concept.

- [ ] **Video preview thumbnails** — Add when users report uncertainty about which file is which. Trigger: user feedback requesting visual file identification.
- [ ] **Queue reordering** — Add when users report needing to control processing order. Trigger: user feedback or observed usage patterns.
- [ ] **Processing log** — Add after first user reports "I don't remember what I did to this video." Trigger: support/debugging need.
- [ ] **Seed export/import** — Add when users ask to share seeds or back them up. Trigger: community signals or export requests.
- [ ] **Batch processing summary** — Add when users express uncertainty about whether processing had any effect. Trigger: trust-building need.
- [ ] **Processing cancellation polish** — Add when users report that killing processes leaves state inconsistent. Trigger: bug reports around cancellation.

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Different seeds per video in batch** — Add when power users hit the one-seed limitation. Trigger: top feature request after v1 stabilizes.
- [ ] **Project files / explicit save** — Add when users manage multiple workflows or share setups. Trigger: usage at scale.
- [ ] **Minimize to tray / background processing** — Add when users report frustration with keeping window open during long batches. Trigger: batch processing times > 30 minutes become common.
- [ ] **Platform-optimized seed presets** — Add when users report platform-specific needs (e.g., "seeds that work best for TikTok"). Trigger: community knowledge accumulation.
- [ ] **Queue import from CSV/text file** — Add when users process 50+ videos regularly. Trigger: queue management friction at scale.

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| FFmpeg detection + download | HIGH | MEDIUM | P1 |
| Seed generation (auto-randomized) | HIGH | HIGH | P1 |
| Seed list management | HIGH | LOW | P1 |
| Video import (drag-drop + picker) | HIGH | LOW | P1 |
| Video queue (basic management) | HIGH | LOW | P1 |
| Batch processing (one seed, many videos) | HIGH | MEDIUM | P1 |
| Progress tracking (per-file) | HIGH | MEDIUM | P1 |
| Output directory + naming | HIGH | LOW | P1 |
| Error handling (per-file) | HIGH | LOW | P1 |
| Video preview thumbnails | MEDIUM | LOW | P2 |
| Queue reordering | MEDIUM | LOW | P2 |
| Processing log | MEDIUM | MEDIUM | P2 |
| Seed export/import | MEDIUM | LOW | P2 |
| Batch processing summary | MEDIUM | MEDIUM | P2 |
| Processing cancellation polish | MEDIUM | MEDIUM | P2 |
| Different seeds per video | LOW | HIGH | P3 |
| Project files | LOW | MEDIUM | P3 |
| Minimize to tray | LOW | MEDIUM | P3 |
| Platform-optimized presets | MEDIUM | LOW | P3 |
| Queue import from CSV | LOW | LOW | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

This tool occupies a niche with no direct competitors (no dedicated video fingerprint modification GUI exists). The competitive alternatives are manual workflows and general-purpose tools.

| Feature | Manual FFmpeg Scripting | Shutter Encoder / HandBrake | Our Approach |
|---------|------------------------|-----------------------------|--------------|
| Seed/recipe concept | Users write shell scripts manually | HandBrake: compression presets; Shutter Encoder: function picker | Auto-generate randomized multi-operation seeds with one click |
| Multi-operation chains | Possible but users must know filter syntax and chain manually | Single-function at a time (transcode OR trim, not both) | Automatic chaining of 3-7 operation types per seed |
| Safety constraints | None — users can destroy video quality accidentally | None — users can misconfigure | Hardcoded safe parameter ranges (transparency ≤ 0.15, shift ≤ 3px, etc.) |
| Batch processing | Requires shell loop scripting | Queue-based batch, good UX | Queue-based batch with per-file progress and error isolation |
| Zero FFmpeg knowledge | No — requires deep FFmpeg expertise | Partially — still requires understanding of codecs, containers, settings | Yes — no FFmpeg knowledge needed |
| Fingerprint modification focus | Possible but not guided | Not designed for this purpose | Purpose-built: every feature supports the fingerprint modification workflow |
| Output verification | Manual `ffprobe` or visual inspection | Not provided | Batch summary with input/output diffs |
| Cross-platform | Scripts break between OSes | Both support macOS/Windows/Linux | Tauri native: macOS/Windows/Linux from same codebase |

## Sources

- [PROJECT.md](/Users/ghost/Code/sandwich/.planning/PROJECT.md) — Requirements and scope definitions
- [Tauri 2.9.5 docs (Context7)](/websites/rs_tauri_2_9_5) — Verified: IPC invoke/command/event system, window drag-drop, async runtime spawning
- [FFmpeg docs (Context7)](/websites/ffmpeg_ffmpeg-all) — Verified: select/aselect filter for frame dropping, colorchannelmixer for pixel manipulation, filtergraph chaining
- Training knowledge: HandBrake, Shutter Encoder, LosslessCut feature sets — MEDIUM confidence (not verified via official sources due to WebFetch restrictions). Used for competitive positioning only.
- Training knowledge: Video fingerprinting techniques (six-class method referenced in PROJECT.md) — MEDIUM confidence. Safety constraint values (transparency 0.15, pixel shift 3px, drop interval 15 frames) from PROJECT.md requirements.

---
*Feature research for: Desktop Video Fingerprint Modification Tool*
*Researched: 2026-05-12*
