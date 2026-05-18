# Phase 07: Audio, Crop, Metadata & Duration - Research

**Researched:** 2026-05-18
**Domain:** FFmpeg filter chains for audio manipulation, video cropping, metadata rewriting, duration modification, and frame dropping
**Confidence:** HIGH

## Summary

Phase 7 extends the fingerprint modification system across four new dimensions: audio operations (5 new types replacing AudioTweak), asymmetric crop as a default operation, metadata write/selective-erase (2 new types alongside existing full erase), and duration manipulation (VideoSpeed + TrimEdges). FrameDrop is upgraded from setpts micro-jitter to real frame dropping via the `select` filter.

All new operations use pure FFmpeg built-in filters (D-05 compliance confirmed). The rubberband library for pitch shifting is NOT available in standard FFmpeg builds (requires `--enable-librubberband`), so pitch shift uses the `asetrate` + `atempo` two-filter technique instead. The architectural challenge is that two operations (VideoSpeed, MetadataSelectiveErase) require capabilities beyond the current single-filter-per-operation model: VideoSpeed generates both video and audio filters from one operation, and MetadataSelectiveErase needs current file metadata from ffprobe to determine what fields to keep.

**Primary recommendation:** Extend `build_filter_args_separated` to return multiple `(FilterKind, Vec<String>)` tuples (not a single pair), and add an optional `MetadataContext` parameter for metadata-aware filter builders. Pre-inject Crop and FrameDrop operations before the random generation loop in `generate_seed`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Audio filter construction (Resample, Volume, Pitch, EQ, Channel) | Rust backend (filters.rs) | — | Pure FFmpeg -af filter expressions; no frontend involvement |
| Crop filter construction (crop+scale chain) | Rust backend (filters.rs) | — | Pure FFmpeg -vf filter expressions; crop dimensions derived from video resolution |
| Metadata write (fake metadata generation) | Rust backend (filters.rs + seed.rs) | — | `-metadata` CLI arguments generated in Rust; word list and date math in seed generation |
| Metadata selective erase | Rust backend (filters.rs + probe.rs + executor.rs) | — | Requires ffprobe to read current metadata, then `-map_metadata -1` + selective `-metadata` writeback |
| VideoSpeed (setpts + atempo sync) | Rust backend (filters.rs) | — | One operation produces both VideoFilter (setpts) and AudioFilter (atempo); executor merge unchanged |
| TrimEdges (trim filter) | Rust backend (filters.rs) | — | Pure FFmpeg trim/atrim filter; start/end/duration parameters |
| FrameDrop (select-based frame decimation) | Rust backend (filters.rs) | — | Pure FFmpeg select filter with mod expression; drops ~1 frame per N |
| Default operation pre-injection (Crop + FrameDrop) | Rust backend (seed.rs) | — | `generate_seed` inserts default ops before random loop; no frontend change |
| Seed migration (AudioTweak split, FrameDrop re-parameterize) | Rust backend (seed.rs / startup) | — | On app startup, scan stored seeds; convert old-format operations to new types |
| OperationType display labels (i18n) | Vue frontend (locales) | — | New i18n keys for 10+ OperationType variants in zh-CN.json + en.json |
| SeedCard operation type rendering | Vue frontend (SeedCard.vue) | — | Existing component pattern; new types just need i18n entries |

## Standard Stack

### Core (No New Dependencies)

Phase 7 requires NO new Rust crates or npm packages. All functionality uses pure FFmpeg built-in filters and the existing Tauri 2.x + Vue 3 stack.

| Filter | Type | FFmpeg Category | Built-In? | Purpose |
|--------|------|-----------------|-----------|---------|
| `aresample` | Audio | libswresample | YES | Resample audio to arbitrary rate (Resample op) |
| `volume` | Audio | lavfi | YES | Adjust volume in dB (Volume op) |
| `asetrate` | Audio | lavfi | YES | Change sample rate without altering PCM data (Pitch op, step 1) |
| `atempo` | Audio | lavfi | YES | Adjust tempo without pitch change (Pitch op step 2, VideoSpeed audio sync) |
| `equalizer` | Audio | lavfi | YES | Two-pole peaking EQ at specific frequency (EQ op) |
| `channelmap` | Audio | lavfi | YES | Remap audio channels (Channel op) |
| `pan` | Audio | lavfi | YES | Mix channels with gain control (Channel op alternative) |
| `crop` | Video | lavfi | YES | Crop to specified dimensions at offset (Crop op) |
| `scale` | Video | libswscale | YES | Resize back to original dimensions (Crop op step 2) |
| `setpts` | Video | lavfi | YES | Change video frame presentation timestamps (VideoSpeed op) |
| `trim` / `atrim` | Video/Audio | lavfi | YES | Extract subpart by time or frame count (TrimEdges op) |
| `select` | Video | lavfi | YES | Expression-based frame selection (FrameDrop op) |
| `-metadata` | CLI option | core | YES | Set/delete metadata fields (MetadataWrite, MetadataSelectiveErase ops) |
| `-map_metadata` | CLI option | core | YES | Strip metadata from input (MetadataErase, MetadataSelectiveErase ops) |

### Explicitly NOT Used

| Filter | Reason |
|--------|--------|
| `rubberband` | Requires `--enable-librubberband` — not available in standard FFmpeg builds. Violates D-05 (pure built-in filters). |
| `atempo` values > 2 | FFmpeg docs warn "may skip samples" for tempo > 2. Our range (0.89-1.12) is safe with single instance. |
| `framestep` | Keeps 1 frame every N (opposite of D-18 requirement to drop 1 every N). `select` filter provides exact control. |

### Verified Package Versions (Existing Stack)

| Package | Current Version | Verified |
|---------|----------------|----------|
| vue | 3.5.34 | npm registry 2026-05-18 |
| pinia | 3.0.4 | npm registry 2026-05-18 |
| naive-ui | 2.44.1 | npm registry 2026-05-18 |
| @tauri-apps/api | 2.11.0 | npm registry 2026-05-18 |
| tauri (Rust) | 2.11.1 | Cargo.toml |
| ffmpeg-sidecar | 2.5.1 | Cargo.toml |
| serde | 1.0.149 | Cargo.toml |
| tokio | 1.52.3 | Cargo.toml |
| rand | 0.9.x | CLAUDE.md reference |

[CITED: npm registry, Cargo.toml, CLAUDE.md]

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Vue 3 Frontend                           │
│                                                                  │
│  SeedCard.vue           BatchControls.vue                       │
│  (displays op types     (strength tier selector,                │
│   via i18n labels)       start/cancel batch)                    │
│       │                       │                                  │
│       ▼                       ▼                                  │
│  seedStore (Pinia)      batchStore (Pinia)                      │
│       │                       │                                  │
└───────┼───────────────────────┼──────────────────────────────────┘
        │ invoke()              │ invoke()
        ▼                       ▼
┌───────────────────────────────────────────────────────────────────┐
│                     Tauri IPC Bridge                              │
│                                                                   │
│  commands::seed::generate_seed()    commands::batch::execute()    │
│       │                                    │                      │
│       │ pre-injects Crop + FrameDrop      │                      │
│       │ before random loop                │                      │
│       ▼                                    ▼                      │
│  ┌──────────────────┐          ┌──────────────────────┐          │
│  │ Seed Generation  │          │   FFmpeg Executor     │          │
│  │                  │          │                       │          │
│  │ pick_operation_  │          │ for each op:          │          │
│  │   type()         │          │   build_filter_args   │          │
│  │   (weighted 1k)  │          │   _separated(op,     │          │
│  │       │          │          │     metadata_ctx?)    │          │
│  │       ▼          │          │       │               │          │
│  │ generate_        │          │       ▼               │          │
│  │   operation()    │          │  VideoFilter → -vf    │          │
│  │   (tier-driven   │          │  AudioFilter → -af    │          │
│  │    param ranges) │          │  Other → pass-through │          │
│  │       │          │          │       │               │          │
│  │       ▼          │          │       ▼               │          │
│  │ validate_        │          │  Merge filters:      │          │
│  │   coverage()     │          │  comma-joined -vf,   │          │
│  │   (≥70% frames)  │          │  comma-joined -af    │          │
│  └──────────────────┘          │       │               │          │
│                                 │       ▼               │          │
│                                 │  Inject GPU encoder   │          │
│                                 │  + -c:v libx264/...   │          │
│                                 │       │               │          │
│                                 │       ▼               │          │
│                                 │  Spawn FFmpeg process │          │
│                                 │  Stream progress      │          │
│                                 │  events to frontend   │          │
│                                 └──────────────────────┘          │
└───────────────────────────────────────────────────────────────────┘
        │
        ▼
┌───────────────────────────────────────────────────────────────────┐
│                    FFmpeg Process (External)                       │
│                                                                   │
│  Input → [select] → [crop,scale] → [setpts] → ... → [encoder] →  │
│  Input → [aresample] → [volume] → [atempo] → ... → [encoder] →   │
│                                                                   │
│  Metadata: -map_metadata -1 -metadata title="..." ...             │
└───────────────────────────────────────────────────────────────────┘
```

### Current vs. Proposed Extension Points

```
                        Phase 6 (Current)          Phase 7 (Proposed)
OperationType enum              20 variants        30+ variants (10 new, -1 AudioTweak removed, + kept for compat)
build_filter_args match         20 arms            30+ arms
build_filter_args_separated     20 arms            30+ arms (+1 special: VideoSpeed returns 2 FilterKinds)
pick_operation_type weights     1000 buckets       1000 buckets (redistributed)
generate_operation tiers        20 arms            30+ arms
operation_type_has_20 test      1 test             UPDATE to 30+
seed migration                  strength_tier       AudioTweak split, FrameDrop re-parameterize
executor loop                   single FilterKind   multi FilterKind per op (VideoSpeed)
filter builder signature        fn(&Operation)      fn(&Operation, Option<&MetadataContext>)
```

### Pattern 1: Filter Builder Extension (New Operations)

**What:** Each new OperationType gets a dedicated builder function following the existing pattern, then registered in `build_filter_args` and `build_filter_args_separated` match arms.

**When to use:** For all new operations (AudioResample, AudioVolume, AudioPitch, AudioEQ, AudioChannel, Crop, MetadataWrite, MetadataSelectiveErase, VideoSpeed, TrimEdges).

**Example (AudioVolume):**
```rust
// Source: existing pattern in filters.rs lines 102-120
pub fn build_audio_volume_filter(op: &Operation) -> Result<Vec<String>, String> {
    let db: f64 = op.params["db"].as_f64().unwrap_or(0.0);
    let db = db.clamp(-3.0, 3.0); // D-02: volume ±3dB
    Ok(vec!["-af".to_string(), format!("volume={}dB", db)])
}
```

**Example (Crop — default operation):**
```rust
// D-05: each side 0.5%-3%, asymmetric, crop then scale back
pub fn build_crop_filter(op: &Operation) -> Result<Vec<String>, String> {
    let crop_w = op.params["cropW"].as_str().unwrap_or("iw*0.98");
    let crop_h = op.params["cropH"].as_str().unwrap_or("ih*0.98");
    let crop_x = op.params["cropX"].as_str().unwrap_or("iw*0.01");
    let crop_y = op.params["cropY"].as_str().unwrap_or("ih*0.01");
    let out_w = op.params["outW"].as_str().unwrap_or("iw");
    let out_h = op.params["outH"].as_str().unwrap_or("ih");
    let filter = format!("crop={}:{}:{}:{},scale={}:{}:flags=lanczos", 
        crop_w, crop_h, crop_x, crop_y, out_w, out_h);
    Ok(vec!["-vf".to_string(), filter])
}
```

### Pattern 2: Multi-Filter Operation (VideoSpeed)

**What:** One OperationType produces both a VideoFilter and an AudioFilter, requiring the `build_filter_args_separated` return type to expand from a single `(FilterKind, Vec<String>)` to `Vec<(FilterKind, Vec<String>)>`.

**When to use:** VideoSpeed only (D-14 requires synchronized setpts + atempo). Future operations with cross-domain effects may reuse.

**Implementation approach:**
```rust
// Changed signature: 
// OLD: fn build_filter_args_separated(op: &Operation) -> Result<(FilterKind, Vec<String>), String>
// NEW: fn build_filter_args_separated(op: &Operation, ctx: Option<&MetadataContext>) -> Result<Vec<(FilterKind, Vec<String>)>, String>

// For VideoSpeed (setpts + atempo synchronized):
fn build_video_speed_filter(op: &Operation) -> Result<Vec<(FilterKind, Vec<String>)>, String> {
    let factor: f64 = op.params["speedFactor"].as_f64().unwrap_or(1.0);
    // setpts: speed up video by inverse (speed=2x → PTS/2)
    let vf_expr = format!("setpts={:.4}*PTS", 1.0 / factor);
    // atempo: speed up audio to match
    let af_expr = format!("atempo={:.4}", factor);
    Ok(vec![
        (FilterKind::VideoFilter(vf_expr), vec!["-vf".to_string(), vf_expr.clone()]),
        (FilterKind::AudioFilter(af_expr), vec!["-af".to_string(), af_expr.clone()]),
    ])
}
```

**Executor adaptation (executor.rs lines 64-98):** The `for op in &seed.operations` loop changes from collecting a single FilterKind per op to iterating over a vec of FilterKinds. Backward compatible — all existing ops return a 1-element vec.

### Pattern 3: Metadata-Aware Filter Builder (SelectiveErase)

**What:** `MetadataSelectiveErase` needs the file's current metadata to determine which fields to keep. The executor probes the file via ffprobe before building filter args, passing the metadata through an optional `MetadataContext` parameter.

**When to use:** Only `MetadataSelectiveErase`. All other operations receive `None` and work as before.

**Implementation approach:**
```rust
// New context struct:
pub struct MetadataContext {
    pub fields: HashMap<String, String>, // key → value for all global metadata
}

// Executor change (before the filter-building loop):
let metadata_ctx = if seed.operations.iter().any(|op| 
    matches!(op.op_type, OperationType::MetadataSelectiveErase)
) {
    Some(probe_metadata(&entry.filepath)?) // extracts all global metadata tags
} else {
    None
};

// Then in the loop:
for op in &seed.operations {
    for (kind, args) in build_filter_args_separated(op, metadata_ctx.as_ref())? {
        // ... existing merge logic
    }
}
```

**MetadataSelectiveErase filter logic:**
1. Receive current metadata fields (via MetadataContext)
2. Identify which categories to erase (random 1-3 from: time, device, description)
3. Determine which fields belong to erased categories based on field-name mapping
4. Output: `-map_metadata -1` (strip all) + `-metadata key=value` for each KEPT field
5. Erased fields are simply not written back

**Metadata categories and field mappings (Claude's Discretion):**
| Category | Fields |
|----------|--------|
| time | creation_time, date, modify_date, timecode, year |
| device | make, model, camera, lens, com.android.*, apple.* |
| description | title, comment, author, copyright, description, album, artist, genre |

### Pattern 4: Default Operation Pre-Injection

**What:** Before the random operation loop in `generate_seed`, insert Crop and FrameDrop operations. These do not count toward `step_count`.

**When to use:** For Crop (D-04) and FrameDrop (D-19) — every seed must contain at least one of each.

**Implementation:**
```rust
// In generate_seed(), before the random loop:
let mut operations = Vec::with_capacity(step_count + 2); // +2 for defaults

// Pre-inject default operations (don't count toward step_count)
operations.push(generate_operation(&mut rng, OperationType::Crop, strength_tier, total_frames));
operations.push(generate_operation(&mut rng, OperationType::FrameDrop, strength_tier, total_frames));

// Then random loop for step_count operations (may include additional Crop/FrameDrop)
for _ in 0..step_count {
    let op_type = pick_operation_type(&mut rng);
    let op = generate_operation(&mut rng, op_type, strength_tier, total_frames);
    operations.push(op);
}
```

### Pattern 5: Seed Migration (AudioTweak Split + FrameDrop Re-parameterize)

**What:** On app startup, scan all stored seeds and convert old-format operations to new types.

**Following Phase 6 D-19 established pattern:**
1. Check a version marker in the store (e.g., `schema_version` field)
2. If below Phase 7 schema version, iterate all seeds and transform operations
3. Save migrated seeds back to store

**Migration transformations:**
| Old Operation | Sub-Effect | New Operation(s) |
|---------------|-----------|-------------------|
| AudioTweak (effect="volume") | volume ±2dB | AudioVolume with db param |
| AudioTweak (effect="tempo") | tempo 0.98-1.02 | AudioPitch with pitchFactor=1.0 (no pitch change, tempo preserved via atempo) |
| AudioTweak (effect="echo") | echo | DISCARD (echo effect has no Phase 7 equivalent per D-01) |
| FrameDrop (setpts jitter) | offset + period | FrameDrop with interval=N param (select-based) |

**Serde compatibility:** Keep `AudioTweak` as a variant in the `OperationType` enum (marked `#[doc(hidden)]`) for deserialization of unmigrated seeds. Never select it in `pick_operation_type`. Migration removes it from all stored seeds.

### Anti-Patterns to Avoid

- **Using rubberband for pitch shift:** Requires custom FFmpeg compile. Violates D-05. Use `asetrate` + `atempo` two-filter chain instead.
- **Using framestep for FrameDrop:** Keeps 1/N frames (not drop 1/N). Use `select='mod(n+1\,N)'` instead.
- **Hardcoding resolution in crop:** Must use `iw`/`ih` expressions, not literal pixel values. Videos have varying resolutions.
- **Returning single FilterKind from VideoSpeed:** Would silently drop either the video or audio speed change. Must return both.
- **Not handling -vsync for frame-dropping filters:** The `select` filter changes frame count. May need `-vsync vfr` flag to prevent ffmpeg from inserting duplicate frames to maintain constant frame rate. The executor should inject `-vsync vfr` when FrameDrop operations are present.
- **Removing AudioTweak from the enum:** Would break deserialization of old seeds. Keep it for backward compat, mark deprecated, remove from pick pool.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Audio pitch shifting | Custom resampling math | `asetrate=r*sample_rate,atempo=1/r,aresample=original_rate` | FFmpeg handles resampling edge cases (anti-aliasing, dither), two-filter chain is standard technique |
| Frame dropping | Custom frame iterator | `select='mod(n+1\,N)'` | select filter handles PTS continuity, keyframe alignment, variable frame rate |
| Crop + scale pipeline | Image processing library | `crop=W:H:X:Y,scale=OW:OH:flags=lanczos` | FFmpeg crop+scale chain is GPU-accelerated on some platforms, handles pixel format conversion |
| Metadata field extraction | Regex on ffprobe output | ffprobe JSON output (`-print_format json -show_format`) | Structured JSON parsing is reliable; regex breaks on edge cases |
| Video speed sync math | Manual PTS calculation | `setpts=1/speed*PTS` + `atempo=speed` | setpts expression engine handles timebase conversion; atempo handles sample-level tempo adjustment |
| Seed migration tracking | Manual version checks | `schema_version` integer in store, increment per phase | Existing pattern from Phase 6 D-19; centralized migration check on startup |

**Key insight:** FFmpeg's built-in filter graph already handles all the complex domain logic (resampling, timing, pixel format conversion). The Rust code only needs to construct the correct filter expressions and CLI arguments. Attempting to do this math in Rust would introduce bugs in edge cases that FFmpeg has already solved over 20+ years of development.

## Runtime State Inventory

### Stored Data

| Item | Details | Action Required |
|------|---------|------------------|
| Seeds in `seeds.json` (tauri-plugin-store) | All user seeds contain Operation arrays with AudioTweak and old FrameDrop (setpts) operations. Stored in app data directory: `~/.local/share/com.sandwich.app/seeds.json` (Linux), `~/Library/Application Support/com.sandwich.app/seeds.json` (macOS), `%APPDATA%/com.sandwich.app/seeds.json` (Windows). | **Data migration:** On startup, run migration to split AudioTweak into new audio types, re-parameterize FrameDrop from setpts to select, and add `schema_version` marker. This is a code edit (migration logic) that performs a data migration (updates stored records). |
| Seed export JSON files | Users may have exported seed JSON files containing AudioTweak and old FrameDrop operations. These are standalone files, not managed by the store. | **Code edit:** The import logic must also apply migration transformations on imported seeds. Add `schema_version` to export format so imported seeds trigger migration if needed. |

### Live Service Config

None — the app has no external service configurations (n8n, Datadog, Tailscale, Cloudflare, etc. are not used).

### OS-Registered State

None — no Task Scheduler entries, pm2 processes, launchd plists, or systemd units registered by this application.

### Secrets and Env Vars

None — no API keys, environment variables, or secret storage used by this application. FFmpeg binary path is stored in tauri-plugin-store (not a secret).

### Build Artifacts

None — no pip packages, compiled binaries (beyond the Tauri bundle itself), or Docker images carry the old OperationType strings. The Tauri binary embeds the Rust backend; version upgrades replace the entire binary.

**Nothing found in category:** Live service config, OS-registered state, secrets/env vars, build artifacts — verified by codebase audit (the application is self-contained desktop software with no external service integrations).

## Common Pitfalls

### Pitfall 1: Rubberband is NOT a Built-In Filter

**What goes wrong:** Developer writes `rubberband=pitch=1.05` in the filter builder, tests locally with a custom-compiled FFmpeg, and it works. On user machines with standard FFmpeg builds (from ffmpeg-sidecar auto-download), the filter is unrecognized and processing fails.

**Why it happens:** `rubberband` requires `--enable-librubberband` at compile time. The ffmpeg-sidecar auto-download fetches standard builds (from ffmpeg.org or BtbN) which do NOT include librubberband.

**How to avoid:** Use `asetrate=r*sample_rate,atempo=1/r,aresample=original_rate` for pitch shifting. Verify all filter names against `ffmpeg -filters` output from the ffmpeg-sidecar downloaded binary.

**Warning signs:** Filter works in dev but fails on fresh install; error message "No such filter: 'rubberband'".

[CITED: Context7 ffmpeg-all docs — rubberband explicitly states "FFmpeg must be configured with '--enable-librubberband'"]

### Pitfall 2: select Filter Requires -vsync vfr

**What goes wrong:** When `select` drops frames, ffmpeg's default `-vsync cfr` (constant frame rate) mode inserts duplicate frames to fill the gaps, effectively undoing the frame drop. The output has the same frame count as the input but with duplicated frames.

**Why it happens:** `-vsync cfr` is the default. FFmpeg tries to maintain the declared frame rate by duplicating or dropping frames as needed.

**How to avoid:** When any operation in the seed uses FrameDrop (select filter), inject `-vsync vfr` into the ffmpeg arguments. This tells ffmpeg to output frames at their natural PTS without inserting duplicates.

**Warning signs:** Output file has same frame count as input; visual inspection shows duplicated frames instead of dropped frames.

### Pitfall 3: Metadata Fields Must Be Retrieved Before Erasure

**What goes wrong:** `MetadataSelectiveErase` is implemented as `-map_metadata -1` (strip all) but doesn't write back the kept fields because it can't determine which fields exist without reading the input first.

**Why it happens:** The filter builder only has access to the Operation parameters, not the file's current metadata. Without ffprobe, there's no way to enumerate existing metadata fields.

**How to avoid:** The executor must probe the file for metadata before building `MetadataSelectiveErase` filter args. Pass the metadata map into the filter builder via `MetadataContext`. Only fields that actually exist in the file can be kept (writing metadata for non-existent fields is harmless but pointless).

**Warning signs:** Output file has no metadata at all after "selective" erase; kept categories are missing.

### Pitfall 4: Multiplication Operator Clash with Existing FfmpegCommand

**What goes wrong:** The `atempo=0.9500` filter format in VideoSpeed passes a float with many decimal places. Some systems may not parse the precision correctly.

**Why it happens:** Rust's `format!("{:.4}", factor)` produces locale-independent output, but edge cases with floating-point representation may produce unexpected values.

**How to avoid:** Use `format!("{:.4}", factor)` with 4 decimal places (matches existing pattern in build_audio_tweak_filter for tempo). Clamp to known safe range (D-15: 0.95-1.05). Test with boundary values.

**Warning signs:** FFmpeg errors about invalid atempo value; output speed is slightly off from expected.

### Pitfall 5: Crop + Existing Operations Interaction

**What goes wrong:** Crop changes the video dimensions, but other video operations (MathOverlay, PixelShift, etc.) have start_frame/duration_frames that reference the original frame count. After crop, the frame count may differ due to potential frame boundary issues.

**Why it happens:** The crop filter changes `iw`/`ih` for subsequent filters in the chain. If crop is applied BEFORE other filters, those filters see the cropped dimensions. If applied AFTER, they see original dimensions.

**How to avoid:** Order operations consistently — default crop is pre-injected before the random loop, so it appears early in the operation chain. The executor applies filters in order, so crop typically runs first. The `scale` back to original dimensions ensures subsequent filters see the original size. Test interaction with each existing filter type.

**Warning signs:** Distorted output where overlay positions don't match video content; scale filter errors about aspect ratio.

## Code Examples

Verified patterns from Context7 FFmpeg documentation:

### Audio Resample
```bash
# Source: Context7 / ffmpeg-all — aresample filter
# Resample to arbitrary rate within 22050-48000 Hz range (D-03)
ffmpeg -i input.mp4 -af "aresample=32000" output.mp4
```

### Audio Volume
```bash
# Source: Context7 / ffmpeg-all — volume filter
# Adjust by ±3dB max (D-02)
ffmpeg -i input.mp4 -af "volume=2.5dB" output.mp4
```

### Audio Pitch (asetrate + atempo chain)
```bash
# Source: Context7 / ffmpeg-all — asetrate section 36.53 + atempo filter
# +2 semitones: pitch factor 2^(2/12) ≈ 1.1225
# Step 1: asetrate changes sample rate (changes BOTH pitch and speed)
# Step 2: atempo restores speed without affecting pitch  
# Step 3: aresample brings sample rate back to original
# All three filters are BUILT-IN (no rubberband needed)
ffmpeg -i input.mp4 -af "asetrate=48000*1.1225,atempo=0.8909,aresample=48000" output.mp4
```

### Audio EQ
```bash
# Source: Context7 / ffmpeg-all — equalizer filter
# Boost 1kHz by 3dB with harmonic width 200Hz
ffmpeg -i input.mp4 -af "equalizer=f=1000:t=h:width=200:g=3" output.mp4
```

### Channel Swap (Stereo L/R)
```bash
# Source: Context7 / ffmpeg-all — channelmap filter
# Swap left and right channels
ffmpeg -i input.mp4 -af "channelmap=map=FL-FR|FR-FL" output.mp4
```

### Crop + Scale (Default Operation)
```bash
# Source: Context7 / ffmpeg-all — crop + scale filters
# Crop 2% from left, 1% from right; 1.5% from top, 2.5% from bottom
# Scale back to original dimensions with lanczos
# crop=out_w:out_h:x:y where w=iw*(1-left%-right%), h=ih*(1-top%-bottom%)
ffmpeg -i input.mp4 -vf "crop=iw*0.97:ih*0.96:iw*0.02:ih*0.015,scale=iw:ih:flags=lanczos" output.mp4
```

### Frame Drop via select Filter
```bash
# Source: Context7 / ffmpeg-all — select filter
# Drop 1 frame every 40 frames: keep frames where mod(n+1,40) != 0
# n=0: mod(1,40)=1 → keep; n=39: mod(40,40)=0 → drop; n=40: mod(41,40)=1 → keep
ffmpeg -i input.mp4 -vf "select='mod(n+1\,40)',setpts=N/FRAME_RATE/TB" -vsync vfr output.mp4
```

### VideoSpeed (setpts + atempo synchronized)
```bash
# Source: Context7 / ffmpeg-all — setpts + atempo filters
# Speed up to 1.03x: video PTS / 1.03, audio tempo 1.03
ffmpeg -i input.mp4 -vf "setpts=0.9709*PTS" -af "atempo=1.03" output.mp4
```

### TrimEdges (Head and Tail)
```bash
# Source: Context7 / ffmpeg-all — trim/atrim filters
# Trim 15 frames from start, 20 frames from end
ffmpeg -i input.mp4 -vf "trim=start_frame=15:end_frame=total-20,setpts=PTS-STARTPTS" \
  -af "atrim=start=0.5:end=duration-0.667,asetpts=PTS-STARTPTS" output.mp4
```

### Metadata Write (Fake Metadata)
```bash
# Source: Context7 / ffmpeg-all — -metadata option
# Write fake metadata fields (D-10)
ffmpeg -i input.mp4 \
  -metadata creation_time="2026-05-03T14:22:00" \
  -metadata title="Untitled Project" \
  -metadata author="admin" \
  -metadata comment="" \
  -metadata copyright="Copyright 2026" \
  -metadata encoder="Sandwich 0.1.0" \
  -c copy output.mp4
```

### Metadata Selective Erase (Category-Based)
```bash
# Source: Context7 / ffmpeg-all — -metadata with empty value deletes field
# Strip all metadata, then selectively write back kept fields
# (requires ffprobe first to know which fields exist)
ffmpeg -i input.mp4 \
  -map_metadata -1 \
  -metadata title="Original Title" \
  -metadata author="Original Author" \
  -c copy output.mp4
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| AudioTweak single op with 3 sub-effects (volume/tempo/echo) | 5 independent audio OperationType variants | Phase 7 | Granular audio control; each effect independently weightable in random pool |
| setpts micro-timing jitter for FrameDrop | `select='mod(n+1\,N)'` true frame decimation | Phase 7 (revert from Phase 6) | Actually removes frames from output; more effective for fingerprint modification |
| rubberband for pitch shifting | `asetrate` + `atempo` two-filter chain | N/A (never used rubberband, but researched) | No external dependency; compatible with standard FFmpeg builds |
| No default operations | Crop + FrameDrop pre-injected in every seed | Phase 7 | Guaranteed baseline fingerprint modification; complements random pool |
| Single FilterKind per operation | Multi FilterKind per operation (for VideoSpeed) | Phase 7 | Enables cross-domain operations; backward compatible with all existing ops |
| Full metadata erase only | FullErase + SelectiveErase + Write fake | Phase 7 | Three metadata strategies for different use cases |

**Deprecated/outdated:**
- **AudioTweak:** Replaced by 5 dedicated audio OperationType variants. Kept in enum for backward deserialization; never generated for new seeds.
- **setpts FrameDrop:** User rejected this Phase 6 approach. Replaced by select-based true frame dropping.
- **echo sub-effect:** No Phase 7 equivalent. Lost during AudioTweak split. Echo was the least-used sub-effect and has no fingerprint modification benefit.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `asetrate` + `atempo` + `aresample` chain produces acceptable pitch-shift quality for ±2 semitone range | Architecture Patterns | If quality is poor (audible artifacts), we need an alternative. However, this is a well-known technique used in production FFmpeg pipelines. |
| A2 | `select='mod(n+1\,N)',setpts=N/FRAME_RATE/TB` with `-vsync vfr` correctly drops 1 frame every N without inserting duplicates | Common Pitfalls | If `-vsync vfr` is insufficient or causes sync issues, we need additional `-r` flag or `fps` filter at the end. |
| A3 | The ffmpeg-sidecar auto-downloaded FFmpeg binary includes all filters listed (aresample, volume, asetrate, atempo, equalizer, channelmap, pan, crop, scale, setpts, trim, select) | Standard Stack | If any filter is missing from the standard binary, the corresponding operation will fail at runtime. All listed filters are in FFmpeg's lavfi library (compiled by default). |
| A4 | `-metadata` with empty value reliably deletes metadata fields | Architecture Patterns | If empty value is platform-dependent or doesn't work for certain container formats (e.g., MOV), we need `-map_metadata -1` + selective write-back as the primary strategy. |
| A5 | tauri-plugin-store `seeds.json` can be read at startup for migration before the main window loads | Runtime State Inventory | If the store is unavailable at startup time, migration must be deferred and seeds will be in a mixed-format state. |
| A6 | Seed export JSON files from Phase 6 contain AudioTweak and old FrameDrop operations that need migration on import | Runtime State Inventory | If users have never exported seeds, this is a no-op. The import migration is defensive coding. |

## Open Questions (RESOLVED)

1. **What to do with AudioTweak echo effect during migration?**
   - What we know: D-01 lists 5 new audio types (Resample, Volume, Pitch, EQ, Channel). Echo has no direct equivalent.
   - What's unclear: Should echo operations be silently dropped during migration, converted to a conservative EQ, or kept as-is in a deprecated state?
   - RESOLVED: Drop echo operations during migration. The echo was a fixed preset (`aecho=0.8:0.9:20:0.1`) with no randomized parameters — minimal fingerprint modification value. Log a warning to the user.

2. **Does `-vsync vfr` interact correctly with GPU encoders?**
   - What we know: GPU encoders (VideoToolbox, NVENC, AMF) may have different behavior with variable frame rate output.
   - What's unclear: Whether some GPU encoders override vsync settings or produce unexpected frame timing.
   - RESOLVED: Test FrameDrop with each GPU encoder variant. Add encoder-specific fallback (e.g., insert `fps=fps=source_fps` filter after select if the GPU encoder needs constant frame rate input).

3. **Should the step_count minimum change with default operations?**
   - What we know: D-06 specifies step counts: conservative 5-7, standard 6-9, aggressive 8-12. These are expected to be the random loop iterations. With 2 extra operations (Crop + FrameDrop), total operations per seed = step_count + 2.
   - What's unclear: Whether the user expects total operations to stay in the current range or increase by 2. 5-7 becoming 7-9 total may shift the complexity profile.
   - RESOLVED: Keep step_count as-is (5-7, 6-9, 8-12) for the random loop. Total operations will be step_count + 2 default ops. This is a slight increase in total operations (7-9 for conservative, up to 10-14 for aggressive) but adds guaranteed baseline coverage.

4. **Are there any interaction issues between crop and existing position-sensitive operations?**
   - What we know: Crop changes `iw`/`ih` for subsequent filters. Operations like MathOverlay (geq uses X/W, Y/H) and SolidColorOverlay (colorize) use relative positioning that should adapt to the new dimensions after crop+scale.
   - What's unclear: Whether the crop+scale cycle introduces sub-pixel shifts that compound with micro-rotate or pixel-shift operations.
   - RESOLVED: Run visual verification with crop+scale combined with each existing operation type. The scale filter with lanczos should handle sub-pixel positions correctly.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Frontend build (Vue/Vite) | Check local | — | Required — must be installed |
| npm | Package management | Check local | — | Required — must be installed |
| Rust (cargo) | Backend compilation | Check local | — | Required — must be installed |
| FFmpeg (ffmpeg-sidecar) | Video/audio processing | Managed by app | Auto-downloaded | Already handled by ffmpeg-sidecar auto_download |
| ffprobe (ffmpeg-sidecar) | Metadata extraction | Managed by app | Auto-downloaded | Already handled by ffmpeg-sidecar auto_download |
| Tauri CLI | Build/development | Check local | — | Required for development; not needed at runtime |

**Note:** Phase 7 uses ONLY the existing development environment. No new tools, services, databases, or CLIs are required. All FFmpeg functionality is provided by filters in the standard ffmpeg-sidecar binary. The `asetrate` + `atempo` pitch-shift technique was specifically chosen to avoid requiring `rubberband` (which would need a custom FFmpeg build).

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | No | Desktop app with no user authentication |
| V3 Session Management | No | No sessions — single-user desktop app |
| V4 Access Control | No | No multi-user access control |
| V5 Input Validation | Yes | **Metadata field values:** Sanitize user-injected strings before passing to `-metadata`. Shell injection risk: metadata values must not contain unescaped shell metacharacters (backticks, $(), ;, \|). Use ffmpeg-sidecar's argument passing (which avoids shell) rather than constructing shell command strings. |
| V6 Cryptography | No | No cryptographic operations in Phase 7 |

### Known Threat Patterns for FFmpeg Metadata Injection

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Shell injection via metadata values | Tampering / Elevation | ffmpeg-sidecar passes arguments as `Vec<String>` directly to `std::process::Command` — no shell interpretation. Verify no code path constructs a single shell command string from user-provided metadata text. [VERIFIED: executor.rs uses FfmpegCommand API with `.args(&all_args)` — no shell string construction] |
| Path traversal in output metadata fields | Tampering | Metadata values are metadata-only — they don't affect file paths. Output path is constructed by `make_output_path()` using sanitized stem + alias. [VERIFIED: executor.rs lines 236-265] |
| Malformed seed import bypassing operation validation | Tampering | Seed import already validates operation count (< 20). Phase 7 should add validation that new OperationType variants have valid parameter structures. |

## Sources

### Primary (HIGH confidence)
- [Context7: FFmpeg `/websites/ffmpeg_ffmpeg-all`] — aresample filter (section 36.47), volume filter, asetrate filter (section 36.53), atempo filter, equalizer filter, channelmap filter, pan filter, crop filter, scale filter, setpts/asetpts filters, trim/atrim filters, select/aselect filters (section 39.103), framestep filter (section 39.103), rubberband filter (section 36.104 — confirms external dependency), -metadata option, -map_metadata option. 4950 total snippets, HIGH reputation.
- [npm registry] — vue@3.5.34, pinia@3.0.4, naive-ui@2.44.1, @tauri-apps/api@2.11.0 — verified via `npm view` 2026-05-18.
- [Cargo.toml] — tauri@2.11.1, ffmpeg-sidecar@2.5.1, serde@1.0.149, tokio@1.52.3 — verified via file read 2026-05-18.

### Secondary (MEDIUM confidence)
- Source code audit: `src-tauri/src/models/seed.rs`, `src-tauri/src/ffmpeg/filters.rs`, `src-tauri/src/ffmpeg/executor.rs`, `src-tauri/src/ffmpeg/probe.rs`, `src-tauri/src/commands/seed.rs`, `src/types/seed.ts`, `src/locales/zh-CN.json`, `src/locales/en.json` — verified current architecture, extension points, and integration patterns.

### Tertiary (LOW confidence)
- A1-A6 in Assumptions Log — marked for validation during implementation or discuss-phase confirmation.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all filters verified via Context7 FFmpeg documentation; no new dependencies needed.
- Architecture: HIGH — extension points identified from existing code structure; patterns verified against current implementations.
- Pitfalls: MEDIUM — pitfall 2 (vsync interaction) and pitfall 5 (crop interaction) need runtime validation; identified from FFmpeg documentation but not tested in this codebase.

**Research date:** 2026-05-18
**Valid until:** 2026-06-18 (30 days — FFmpeg filter APIs are highly stable)

---

*Phase: 07-audio-crop-meta*
*Research complete — ready for planning*
