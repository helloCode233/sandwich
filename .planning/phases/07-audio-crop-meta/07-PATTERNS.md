# Phase 07: Audio, Crop, Metadata & Duration - Pattern Map

**Mapped:** 2026-05-18
**Files analyzed:** 12 (7 modified, 1 created, 4 extended-in-place)
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/models/seed.rs` | model | data-definition | itself (existing enum + struct patterns) | exact -- extends existing |
| `src-tauri/src/ffmpeg/filters.rs` | service/utility | transform (Operation -> FFmpeg args) | itself (existing 20 filter builders) | exact -- extends existing |
| `src-tauri/src/commands/seed.rs` | controller (Tauri command) | request-response | itself (existing pick/generate/generate_seed) | exact -- extends existing |
| `src-tauri/src/ffmpeg/executor.rs` | service | streaming (FFmpeg process + events) | itself (existing executor loop) | exact -- extends existing |
| `src-tauri/src/ffmpeg/probe.rs` | utility | file-I/O (ffprobe -> data) | itself (existing extract_metadata) | exact -- extends existing |
| `src/types/seed.ts` | type-definition | data-definition | itself (existing OperationType union) | exact -- extends existing |
| `src/locales/zh-CN.json` | config | config (key-value) | itself (existing i18n pattern) | exact -- extends existing |
| `src/locales/en.json` | config | config (key-value) | itself (existing i18n pattern) | exact -- extends existing |
| `src/components/seed/SeedCard.vue` | component (Vue) | reactive-display | itself (existing display pattern) | exact -- minimal change |
| `src-tauri/src/lib.rs` | config | startup-lifecycle | itself (existing migration registration) | exact -- extends existing |
| `src-tauri/src/migrations/mod.rs` | config | module-registration | itself (line 1: `pub mod seed_v2;`) | exact -- adds line |
| `src-tauri/src/migrations/seed_v3.rs` | migration | data-transform | `src-tauri/src/migrations/seed_v2.rs` | role-match (same role, different transform) |

## Pattern Assignments

### 1. `src-tauri/src/models/seed.rs` (model, data-definition)

**Analog:** `src-tauri/src/models/seed.rs` -- lines 125-174 (OperationType enum), lines 75-92 (Seed struct with `#[serde(default)]`)

**OperationType enum extension pattern** (lines 125-174):
```rust
/// The 30 operation types covering all fingerprint modification categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    // ... existing 20 variants ...
    // Phase 7: Audio operations (5) -- replace AudioTweak's 3 sub-effects
    /// Resample audio to random rate 22050-48000 Hz.
    AudioResample,
    /// Adjust volume by +/-3 dB.
    AudioVolume,
    /// Pitch shift via asetrate+atempo chain, +/-2 semitones.
    AudioPitch,
    /// Parametric EQ at random frequency.
    AudioEQ,
    /// Channel remapping (swap, mono mixdown, etc.).
    AudioChannel,
    // Phase 7: Crop (1) -- default operation
    /// Asymmetric crop (0.5%-3% per side) then scale back to original.
    Crop,
    // Phase 7: Metadata (2) -- supplement existing MetadataErase
    /// Write fake metadata fields (creation_time, title, author, etc.).
    MetadataWrite,
    /// Selectively erase metadata by category (time/device/description).
    MetadataSelectiveErase,
    // Phase 7: Duration (2)
    /// Video speed change (setpts + atempo synchronized), 0.95-1.05x.
    VideoSpeed,
    /// Trim head/tail frames (1-30 frames from start, end, or both).
    TrimEdges,
}
```
**Key pattern:** Each variant gets a doc comment. `#[serde(rename_all = "camelCase")]` at enum level produces camelCase names (e.g. `AudioResample` -> `"audioResample"`). AudioTweak is **kept** for backward deserialization but removed from the random pick pool.

**New field addition pattern** (lines 75-92, Seed struct):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seed {
    // ... existing fields ...
    /// Schema version for migration tracking (Phase 7).
    /// #[serde(default)] ensures old seeds without this field default to 0.
    #[serde(default)]
    pub schema_version: u32,
}
```

**Test enum count pattern** (lines 7-33):
```rust
#[test]
fn operation_type_has_30_variants() {
    let variants = &[
        // existing 20 + Phase 7 10 new (5 audio + 2 metadata + 2 duration + 1 crop)
        // AudioTweak is counted in existing 20, kept for backward compat
        OperationType::AudioResample,
        OperationType::AudioVolume,
        // ... all 30 variants ...
    ];
    assert_eq!(variants.len(), 30, "OperationType must have exactly 30 variants");
}
```

---

### 2. `src-tauri/src/ffmpeg/filters.rs` (service/utility, transform)

**Analog:** `src-tauri/src/ffmpeg/filters.rs` lines 1-492 (entire file)

**Filter builder function pattern** (lines 10-41, build_math_overlay_filter as exemplar):

Each new operation gets a dedicated `pub fn build_X_filter(op: &Operation) -> Result<Vec<String>, String>` function.

**Audio filter builder pattern** (copy from lines 135-144, build_hue_rotate_filter):
```rust
/// Build FFmpeg filter arguments for audio resampling.
/// D-03: Random rate 22050-48000 Hz.
pub fn build_audio_resample_filter(op: &Operation) -> Result<Vec<String>, String> {
    let sample_rate: u32 = op.params["sampleRate"].as_u64().unwrap_or(44100) as u32;
    let sample_rate = sample_rate.clamp(22050, 48000);
    let filter = format!("aresample={}", sample_rate);
    Ok(vec!["-af".to_string(), filter])
}
```

**Crop + scale combo pattern** (copy from lines 43-62, build_pixel_shift_filter chain):
```rust
/// Build crop filter (D-05: asymmetric per-side, D-07: tier-driven).
/// crop=W:H:X:Y extracts a sub-rectangle, then scale=OW:OH scales back.
pub fn build_crop_filter(op: &Operation) -> Result<Vec<String>, String> {
    let left_pct: f64 = op.params["leftPct"].as_f64().unwrap_or(1.0);
    let right_pct: f64 = op.params["rightPct"].as_f64().unwrap_or(1.0);
    let top_pct: f64 = op.params["topPct"].as_f64().unwrap_or(1.0);
    let bottom_pct: f64 = op.params["bottomPct"].as_f64().unwrap_or(1.0);

    // Clamp to safety range (0.5%-3.5%)
    let left_pct = left_pct.clamp(0.5, 3.5);
    let right_pct = right_pct.clamp(0.5, 3.5);
    let top_pct = top_pct.clamp(0.5, 3.5);
    let bottom_pct = bottom_pct.clamp(0.5, 3.5);

    // crop=out_w:out_h:x:y where each is an expression
    let filter = format!(
        "crop=iw*(1-{}/100-{}/100):ih*(1-{}/100-{}/100):iw*{}/100:ih*{}/100,scale=iw:ih:flags=lanczos",
        left_pct, right_pct, top_pct, bottom_pct, left_pct, top_pct
    );
    Ok(vec!["-vf".to_string(), filter])
}
```

**build_filter_args dispatch pattern** (lines 352-378):
```rust
pub fn build_filter_args(op: &Operation) -> Result<Vec<String>, String> {
    match op.op_type {
        // ... existing 20 arms ...
        // Phase 7: Audio (5)
        OperationType::AudioResample => build_audio_resample_filter(op),
        OperationType::AudioVolume => build_audio_volume_filter(op),
        OperationType::AudioPitch => build_audio_pitch_filter(op),
        OperationType::AudioEQ => build_audio_eq_filter(op),
        OperationType::AudioChannel => build_audio_channel_filter(op),
        // Phase 7: Crop (1) -- VideoFilter
        OperationType::Crop => build_crop_filter(op),
        // Phase 7: Metadata (2) -- Other (CLI args, not filters)
        OperationType::MetadataWrite => build_metadata_write_filter(op),
        OperationType::MetadataSelectiveErase => build_metadata_selective_erase_filter(op),
        // Phase 7: Duration (2)
        OperationType::VideoSpeed => build_video_speed_filter(op),
        OperationType::TrimEdges => build_trim_edges_filter(op),
        // AudioTweak kept for backward compat -- delegate to existing
        OperationType::AudioTweak => build_audio_tweak_filter(op),
    }
}
```

**build_filter_args_separated SIGNATURE CHANGE** (line 394):

Old signature:
```rust
pub fn build_filter_args_separated(op: &Operation) -> Result<(FilterKind, Vec<String>), String>
```

New signature (to support VideoSpeed multi-filter + MetadataContext):
```rust
pub fn build_filter_args_separated(
    op: &Operation,
    metadata_ctx: Option<&MetadataContext>,
) -> Result<Vec<(FilterKind, Vec<String>)>, String>
```

**MetadataContext struct** — defined in filters.rs (not probe.rs) so Plan 03 compiles independently in Wave 2:
```rust
pub struct MetadataContext {
    pub fields: HashMap<String, String>,
}
```

**VideoSpeed multi-filter pattern** (new, returns 2 FilterKinds):
```rust
OperationType::VideoSpeed => {
    let args = build_video_speed_filter(op)?;
    let vf_expr = args.get(1).cloned().unwrap_or_default();
    let af_expr = args.get(3).cloned().unwrap_or_default();
    Ok(vec![
        (FilterKind::VideoFilter(vf_expr.clone()), vec!["-vf".to_string(), vf_expr]),
        (FilterKind::AudioFilter(af_expr.clone()), vec!["-af".to_string(), af_expr]),
    ])
}
```

All existing match arms remain as-is but return `Ok(vec![(kind, args)])` instead of `Ok((kind, args))`.

---

### 3. `src-tauri/src/commands/seed.rs` (controller, request-response)

**Analog:** `src-tauri/src/commands/seed.rs` lines 1-817 (entire file)

**pick_operation_type weight redistribution pattern** (lines 13-46):

Copy existing 1000-bucket pattern. The 10 new types need weight allocation. Per CONTEXT.md, AudioTweak is removed from the pool (kept in enum for backward compat). Total pool: 30 types - 1 (AudioTweak deprecated) = 29 types in pool. **Crop and FrameDrop ARE in the pool at low weight** (~14 buckets each) because they are pre-injected defaults but can also appear again for a second instance (dual-guarantee per D-04, D-19).

```rust
fn pick_operation_type(rng: &mut impl Rng) -> OperationType {
    let roll: u32 = rng.random_range(1..=1000);
    match roll {
        // Math overlay: ~120 buckets
        1..=40 => OperationType::MathOverlay,
        41..=80 => OperationType::MathOverlay,
        81..=120 => OperationType::MathOverlay,
        // ... etc ...
        // Phase 7: Audio (5): ~120 buckets
        // Phase 7: Metadata (2): ~40 buckets (plus existing MetadataErase at ~38)
        // Phase 7: Duration (2): ~50 buckets
        // Crop and FrameDrop: ~28 buckets total (low weight, dual-guarantee per D-04/D-19)
        // NOTE: AudioTweak NOT in pick pool -- deprecated, kept only for backward deserialization
        _ => unreachable!("roll is 1..=1000"),
    }
}
```

**generate_seed pre-injection pattern** (lines 84-172):

Modify the existing `generate_seed` function to pre-inject default operations before the random loop:

```rust
#[tauri::command]
pub async fn generate_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    strength: String,
    total_frames: Option<u32>,
) -> Result<Seed, String> {
    // ... strength_tier parsing (unchanged) ...
    // ... step count (unchanged) ...

    let step_count = rng.random_range(min_steps..=max_steps);
    // +2 capacity for default operations (Crop + FrameDrop)
    let mut operations = Vec::with_capacity(step_count + 2);

    // --- Phase 7: Pre-inject default operations (D-04, D-19) ---
    // These do not count toward step_count and are guaranteed in every seed.
    operations.push(generate_operation(&mut rng, OperationType::Crop, strength_tier, total_frames));
    operations.push(generate_operation(&mut rng, OperationType::FrameDrop, strength_tier, total_frames));

    // Random loop: step_count operations (may randomly pick additional Crop/FrameDrop)
    for _ in 0..step_count {
        let op_type = pick_operation_type(&mut rng);
        let op = generate_operation(&mut rng, op_type, strength_tier, total_frames);
        operations.push(op);
    }
    // ... rest unchanged (coverage validation, alias, persist, emit) ...
}
```

**generate_operation new arm pattern** (lines 218-479):

Copy the tier-driven range pattern. Example for AudioResample:

```rust
OperationType::AudioResample => {
    let (rate_min, rate_max) = match strength_tier {
        StrengthTier::Conservative => (32000u32, 48000u32),
        StrengthTier::Standard => (24000u32, 48000u32),
        StrengthTier::Aggressive => (22050u32, 48000u32),
    };
    serde_json::json!({ "sampleRate": rng.random_range(rate_min..=rate_max) })
}
```

The FrameDrop arm is re-parameterized from setpts jitter (offset/period) to select-based interval:
```rust
OperationType::FrameDrop => {
    let (int_min, int_max) = match strength_tier {
        StrengthTier::Conservative => (40u32, 50u32),  // D-19
        StrengthTier::Standard => (30u32, 45u32),      // D-19
        StrengthTier::Aggressive => (25u32, 35u32),    // D-19
    };
    serde_json::json!({ "interval": rng.random_range(int_min..=int_max) })
}
```

**Coverage validation** (unchanged, lines 48-74). Default ops contribute their frame ranges like any other operation -- `validate_coverage` is coverage-blind.

**Test patterns** -- copy from lines 615-817:
- `variant_index` helper function (lines 621-644): extend with new variant indices (0-29)
- `pick_operation_type_covers_all_X_types` test (lines 657-670): update seen_flags array size to 29
- Tier-driven range tests (lines 773-803): add tests for new operation parameter ranges

---

### 4. `src-tauri/src/ffmpeg/executor.rs` (service, streaming)

**Analog:** `src-tauri/src/ffmpeg/executor.rs` lines 1-266 (entire file)

**Executor loop adaptation** (lines 64-71): The `for op in &seed.operations` loop changes to accommodate multi-FilterKind returns:

```rust
// OLD (lines 64-71):
for op in &seed.operations {
    let (kind, args) = build_filter_args_separated(op)?;
    match kind { ... }
}

// NEW: handle multi-FilterKind per op (VideoSpeed returns 2 FilterKinds)
for op in &seed.operations {
    let results = build_filter_args_separated(op, metadata_ctx.as_ref())?;
    for (kind, args) in results {
        match kind {
            FilterKind::VideoFilter(expr) => vf_exprs.push(expr),
            FilterKind::AudioFilter(expr) => af_exprs.push(expr),
            FilterKind::Other(other_args_batch) => other_args.extend(other_args_batch),
        }
    }
}
```

**-vsync vfr injection** (new, after filter merge, before GPU encoder):

```rust
// Phase 7: FrameDrop uses 'select' filter which drops frames.
// -vsync vfr prevents ffmpeg from inserting duplicate frames to maintain CFR.
let has_frame_drop = seed.operations.iter().any(|op|
    matches!(op.op_type, OperationType::FrameDrop)
);
if has_frame_drop {
    let mut vsync_args = vec!["-vsync".to_string(), "vfr".to_string()];
    vsync_args.extend(encoder_args);
    encoder_args = vsync_args;
}
```

**MetadataContext** -- Since `MetadataSelectiveErase` needs current file metadata, the executor probes the file before building filter args:

```rust
let metadata_ctx: Option<MetadataContext> = if seed.operations.iter().any(|op|
    matches!(op.op_type, OperationType::MetadataSelectiveErase)
) {
    match probe_global_metadata(&entry.filepath) {
        Ok(fields) => Some(MetadataContext { fields }),
        Err(_) => None, // fallback to full erase
    }
} else {
    None
};
```

---

### 5. `src-tauri/src/ffmpeg/probe.rs` (utility, file-I/O)

**Analog:** `src-tauri/src/ffmpeg/probe.rs` lines 1-148 (entire file, especially `extract_metadata`)

**New function pattern** -- `probe_global_metadata` for MetadataSelectiveErase:

```rust
/// Run ffprobe to extract all global metadata tags (not streams).
pub fn probe_global_metadata(filepath: &str) -> Result<HashMap<String, String>, String> {
    let ffprobe_bin = ffprobe_path();
    let output = Command::new(&ffprobe_bin)
        .args(["-v", "quiet", "-print_format", "json", "-show_format", filepath])
        .output()
        .map_err(|e| format!("ffprobe failed: {}", e))?;
    if !output.status.success() {
        return Err(format!("Cannot probe file metadata"));
    }
    let probe: RawProbeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe JSON: {}", e))?;
    Ok(probe.format.tags.unwrap_or_default())
}
```

The `RawFormat` struct needs extension:
```rust
struct RawFormat {
    #[serde(default)] duration: String,
    #[serde(default)] size: String,
    #[serde(default)] tags: Option<HashMap<String, String>>,
}
```

---

### 6-12. (See individual plan context sections for the remaining pattern assignments)

The remaining files (types/seed.ts, locales, SeedCard.vue, lib.rs, migrations/mod.rs, migrations/seed_v3.rs) follow identical extension patterns as described in their respective plan files.

---

## Shared Patterns

### Authentication / Authorization
**Not applicable** -- desktop app with no authentication.

### Error Handling
**Source:** `src-tauri/src/commands/seed.rs` lines 91-100 (strength tier validation), `src-tauri/src/ffmpeg/executor.rs` lines 49-50 (cancel check pattern)

All Tauri commands return `Result<T, String>`. Use `.map_err(|e| format!("...", e))` for error conversion. Filter builders return `Result<Vec<String>, String>`.

### Tauri Command Pattern
**Source:** `src-tauri/src/commands/seed.rs` lines 84-90

```rust
#[tauri::command]
pub async fn command_name(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) -> Result<ReturnType, String> {
    let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    // ... work ...
    let _ = app.emit("event-name", ());
    Ok(result)
}
```

### Persistence Pattern
**Source:** `src-tauri/src/commands/seed.rs` lines 600-612

Standard tauri-plugin-store pattern.

### Serialization (serde)
**Source:** `src-tauri/src/models/seed.rs` lines 77-123

`#[serde(rename_all = "camelCase")]` on structs, `#[serde(default)]` for backward compat on new fields.

### Pinia Store Pattern
**Source:** `src/stores/seed.ts` lines 1-64

Composition API setup store with `defineStore('name', () => { ref + computed + actions })`.

### Vue Component Pattern
**Source:** `src/components/seed/SeedCard.vue` lines 1-241

`<script setup lang="ts">` with Naive UI components, `useI18n()` for translations, `useMessage()` for notifications.

### Module Registration
**Source:** `src-tauri/src/migrations/mod.rs` line 1, `src-tauri/src/commands/mod.rs` line 1

Simply `pub mod module_name;`.

---

## No Analog Found

All 12 files have exact or role-match analogs in the existing codebase. No files require novel patterns.

## Metadata

**Analog search scope:** `src-tauri/src/models/`, `src-tauri/src/ffmpeg/`, `src-tauri/src/commands/`, `src-tauri/src/migrations/`, `src-tauri/src/state.rs`, `src-tauri/src/lib.rs`, `src/types/`, `src/locales/`, `src/components/seed/`, `src/stores/`, `src/composables/`
**Files scanned:** 24
**Pattern extraction date:** 2026-05-18
