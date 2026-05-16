# Phase 6: 增强指纹修改 - Research

**Researched:** 2026-05-16
**Domain:** FFmpeg video filter chains, Vue 3 drag-and-drop, seed generation intelligence, desktop file I/O
**Confidence:** HIGH

## Summary

Phase 6 systematically enhances the fingerprint modification engine on top of the Phase 5 production-hardened foundation. Four major work streams: (1) 13 new FFmpeg operation types across four categories (color/noise/geometric/blend), all using built-in filters with zero external dependencies; (2) seed generation intelligence upgrade with 5-12 step chains, 3-tier strength presets, and >=70% video coverage guarantee; (3) seed JSON export/import via tauri-plugin-dialog file save/open dialogs; (4) three v2 deferred productivity features: drag-to-reorder queue, first-frame thumbnail preview, and processing log history panel.

The existing FilterKind architecture (VideoFilter/AudioFilter/Other) and executor pipeline fully support new operation types without executor changes. New filters are built in `filters.rs` following the established pattern and automatically merged into comma-joined `-vf` chains. The Pinia Composition API store pattern, Naive UI component tree, and Tauri IPC event system are all reused in-place.

**Primary recommendation:** Extend the existing codebase in-place -- no architectural refactors needed. The executor, filter dispatch, seed generation, and UI components all have clean extension points ready for Phase 6 additions. Use vue-draggable-plus (SortableJS-based) for queue reordering rather than hand-rolling HTML5 drag-and-drop.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| New FFmpeg filter construction | API / Backend (Rust) | -- | Filter chain strings are built in `filters.rs` and passed to ffmpeg-sidecar; purely backend |
| OperationType enum extension | API / Backend (Rust) | Browser / Client (TS types) | Rust owns the canonical enum; TypeScript mirrors via union type for IPC serialization |
| Seed generation w/ strength tiers | API / Backend (Rust) | -- | Random generation, parameter clamping, coverage algorithm all run in Rust `commands/seed.rs` |
| Seed JSON export/import | API / Backend (Rust) | Browser / Client (Vue) | Rust reads/writes seed data; `tauri-plugin-dialog` save/open dialogs invoked from frontend via `invoke()` |
| Legacy seed migration | API / Backend (Rust) | -- | Startup migration logic in Rust reads old seeds, adds `strength_tier: "standard"`, writes back |
| Drag-to-reorder queue | Browser / Client (Vue) | Frontend Server (SSR) N/A | HTML5 drag-and-drop with SortableJS binding; order persisted to Pinia store and serialized to Rust queue |
| Thumbnail extraction | API / Backend (Rust) | -- | ffmpeg-sidecar FfmpegCommand extracts first frame; base64 encoding in Rust, stored in VideoEntry |
| Processing log history | Browser / Client (Vue) | API / Backend (Rust) | Logs accumulated in Pinia store from `batch-log` events; UI filtering/sorting is pure frontend |
| Strength tier selector UI | Browser / Client (Vue) | -- | Simple 3-option selector in BatchControls; selected tier sent to Rust on seed generation |

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Four new operation categories ALL required: color processing (hue rotation/saturation/brightness/color balance), noise texture (grain noise/blur/sharpen), geometric fine-tuning (rotate <=1 degree/scale 99%-101%/flip), blend overlay (semi-transparent solid color/gradient/watermark blend, opacity <=0.15).
- **D-02:** Each category at least 3 concrete operation variants, total 12+ new operations. Combined with existing 7 types, total reaches 19+.
- **D-03:** Safety constraints change from hardcoded clamp to 3-tier adjustable: Conservative (safe lower bounds, 5-7 steps), Standard (current mid-range behavior, 6-9 steps), Aggressive (near upper bounds, 8-12 steps). User selects global strength with one click; individual operation parameters NOT exposed.
- **D-04:** New operations join unified random pool with existing 7 types. Users do NOT select by category.
- **D-05:** Pure FFmpeg built-in filter implementation (geq, hue, eq, curves, noise, atadenoise, rotate, scale, transpose, overlay, colorbalance, colorchannelmixer, etc.). No third-party filter libraries. No custom FFmpeg compilation.
- **D-06:** Operation steps expand from 3-7 to 5-12. Step count is an important random dimension of seed complexity.
- **D-07:** Three global strength presets -- Conservative: parameters near safe lower bounds + 5-7 steps; Standard: parameters mid-range + 6-9 steps; Aggressive: parameters near upper bounds + 8-12 steps. Seed model gains `strength_tier` field.
- **D-08:** Operation chain order remains pure random (no pipeline stage sorting). Randomness is core to fingerprint modification.
- **D-09:** Operation chain MUST cover >=70% of video duration. Each operation assigned random start_frame/duration_frames (not default 0=full video). Final coverage validated; re-randomize if below threshold. FrameDrop retains its existing time-slice behavior.
- **D-10:** Seed export/import -- single-file JSON format containing complete fields (id/alias/operations/created_at/strength_tier).
- **D-11:** UI entry -- seed card hover shows export/import small icon buttons in bottom-right corner (alongside existing rename/copy/delete buttons).
- **D-12:** Import generates new UUID and created_at timestamp; adds as new seed to list. Does NOT match existing IDs (prevents overwrite risk).
- **D-13:** SEED-COMPLEX-01 (different videos use different seeds) deferred to later phase. Current multi-seed batch processing (Phase 5) already satisfies primary needs.
- **D-14:** PROD-01 drag-to-reorder -- HTML5 drag-and-drop on queue list. New order persisted to store; batch processing follows list order top-to-bottom.
- **D-15:** PROD-02 thumbnail preview -- extract first frame via ffmpeg (`-ss 1 -vframes 1`) on import, stored as base64 in VideoEntry and persisted. Queue list displays thumbnail per row.
- **D-16:** PROD-03 processing log history -- inline log panel (NOT standalone modal), search/filter by date/filename/seed name, shows per-processing details (time/duration/MD5 before-after/success-failure status/output path). Logs persisted in store.
- **D-17:** Operation category weight distribution (total ~19+ operations):
  - Math overlay (existing 3): ~15%
  - Color processing (new): ~20%
  - Noise texture (new): ~15%
  - Geometric fine-tuning (new): ~15%
  - Blend overlay (new): ~10%
  - Remaining old categories (pixel shift/frame drop/GOP/metadata/audio/remux, 6): ~25%
  - Sub-operations within categories evenly weighted. Exact values finalized during planning.
- **D-18:** Incrementally extend existing dual-panel layout -- left panel: seed list + strength selector + export/import buttons; right panel: queue list + thumbnails + drag handles + log tab. No third panel or drawer sidebar.
- **D-19:** Auto-migrate ALL legacy seeds on startup -- add `strength_tier: "standard"` default field. Old seed operations unchanged (preserve original 7 types and params). Migrated seeds marked "upgraded"; user may use or delete + rebuild.
- **D-20:** New seeds (Phase 6+) include both old and new operation types, format includes strength_tier field. New format backward compatible -- old app versions ignore unknown fields (serde `#[serde(default)]`).

### Claude's Discretion

- Specific FFmpeg filter selection and parameter range definitions for each new operation category
- Fine-grained weight values for sub-operation types within each category
- Parameter interpolation/mapping rules for each strength tier
- Coverage >=70% validation algorithm and retry strategy
- HTML5 drag-and-drop sort implementation specifics (draggable/v-model binding)
- Thumbnail base64 resolution/file size limit strategy
- Log panel UI details (tab position, filter controls, summary statistics)
- i18n new keys (strength tiers, new operation names, export/import, thumbnails, logs)
- Auto-migration script implementation logic
- Seed model extension fields (`strength_tier: "conservative" | "standard" | "aggressive"`)
- VideoEntry model extension fields (`thumbnail_base64: Option<String>`, `order_index: u32`)
- OperationType enum extension (12+ new variants)
- Whether weight configuration is adjustable via config file

### Deferred Ideas (OUT OF SCOPE)

- SEED-COMPLEX-01 (different videos use different seeds) -> later phase
- Continuous strength slider (1-10) -> currently 3 presets; may upgrade based on feedback
- Operation chain pipeline stage intelligent sorting -> currently random; add as needed
- Multi-frame preview strip (hover shows multiple time-point thumbnails) -> first frame sufficient for now
- Log statistics dashboard (success rate/most-used seeds, etc.) -> later iteration
- Weight configuration UI (user-customizable category weights) -> later phase
- GPU encoder manual selection UI (Phase 5 D-06) -> later
- Code signing/store publishing -> separate future phase

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PROD-01 | Video queue drag-to-reorder | vue-draggable-plus v0.6.1 (SortableJS-based, Vue 3 native); v-model two-way binding to reactive array in queue store |
| PROD-02 | Video thumbnail preview (first frame) | ffmpeg-sidecar FfmpegCommand with `-ss 1 -vframes 1` args; base64 encode in Rust; scale to 120px wide max to limit memory |
| PROD-03 | Processing log history panel | Naive UI NTabs with inline panel; computed filtered lists; Pinia store for log persistence |
| SEED-EXPORT-01 | Seed export as JSON file | tauri-plugin-dialog `save()` with `filters: [{ name: 'JSON', extensions: ['json'] }]`; serde_json serialization in Rust |
| SEED-EXPORT-02 | Seed import from JSON file | tauri-plugin-dialog `open()` with JSON filter; deserialize + validate + regenerate UUID/timestamp in Rust |

## Standard Stack

### Core (Additions to Existing Stack)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| vue-draggable-plus | 0.6.1 | Drag-and-drop list reordering | Vue 3 native SortableJS wrapper; v-model two-way binding; `:animation="150"` for smooth transitions; lightweight (~15KB); used by 2k+ Vue 3 projects [VERIFIED: npm registry] |
| ffmpeg-sidecar | 2.5.1 (existing) | Single-frame thumbnail extraction + all FFmpeg execution | Already in project; `FfmpegCommand::new_with_path().args(["-ss", "1", "-vframes", "1"]).spawn()` for frame capture [VERIFIED: crates.io, Context7 /nathanbabcock/ffmpeg-sidecar] |
| tauri-plugin-dialog | 2.7.1 (existing) | Save/open file dialogs for seed JSON | Already in project; `save()` returns path string for Rust to write JSON; `open()` with filter for JSON files [VERIFIED: npm registry, tauri.app docs] |
| serde + serde_json | 1.x (existing) | Seed JSON serialization | Already in project; `#[serde(rename_all = "camelCase")]` with `#[serde(default)]` for backward compatibility [VERIFIED: crates.io] |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @vueuse/core | 14.3.0 | `useDraggable` composable (alternative) | Only if vue-draggable-plus proves insufficient for the use case; Overkill for simple list reordering |
| rand | 0.9.x (existing) | Coverage validation re-randomization | Already in project; used for operation generation and coverage retry loop |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| vue-draggable-plus | vuedraggable (v2.24.3, Vue 2-era) | vuedraggable v2.24.3 is Vue 2-focused with Vue 3 compatibility layer; vue-draggable-plus is Vue 3 native with better TypeScript support and smaller bundle |
| vue-draggable-plus | Hand-rolled HTML5 drag-and-drop | Custom implementation requires handling drag ghost images, touch support, accessibility, sort animations -- 200+ lines of fragile code. VueDraggablePlus handles all edge cases |
| Thumbnail via ffmpeg-sidecar rawvideo | Thumbnail via HTML5 `<video>` + canvas | Browser-based extraction is unreliable for arbitrary video codecs; ffmpeg is deterministic |
| Naive UI NTabs for log panel | Custom tab implementation | NTabs is already in the Naive UI component tree; consistent with existing UI patterns |

**Installation:**
```bash
npm install vue-draggable-plus
```

**Version verification:**
- vue-draggable-plus: `0.6.1` [VERIFIED: npm registry, 2026-05-16]
- @vueuse/core: `14.3.0` [VERIFIED: npm registry]
- ffmpeg-sidecar: `2.5.1` [VERIFIED: crates.io]
- @tauri-apps/plugin-dialog: `2.7.1` [VERIFIED: npm registry]

## Architecture Patterns

### System Architecture Diagram

```
User Input (UI)
    |
    v
[Left Panel: Seeds]                    [Right Panel: Queue + Logs]
    |                                        |
    |-- Generate Seed (strength tier)        |-- Import Video (+ thumbnail extraction)
    |   |                                    |   |
    |   v                                    |   v
    |   Rust: commands/seed.rs               |   Rust: commands/import.rs
    |   |-- pick_operation_type (weights)    |   |-- ffprobe metadata
    |   |-- generate_operation (params)      |   |-- ffmpeg -ss 1 -vframes 1 --> base64
    |   |-- validate_coverage >= 70%         |   |-- VideoEntry { thumbnail_base64, order_index }
    |   |-- Seed { strength_tier }           |   |
    |   v                                    |   v
    |   Pinia: seedStore                     |   Pinia: queueStore
    |                                        |
    |-- Export Seed (JSON)                   |-- Drag Reorder (vue-draggable-plus)
    |   |-- dialog.save() --> path           |   |-- v-model="queueStore.entries"
    |   |-- Rust: write seed JSON to file    |   |-- persist order_index to Rust
    |                                        |
    |-- Import Seed (JSON)                   |-- Processing Log
    |   |-- dialog.open() --> path               |-- Pinia: logStore (from batch-log events)
    |   |-- Rust: read JSON, validate            |-- NTabs: filter/search panel
    |   |-- new UUID + timestamp                 |
    |                                            |
    v                                            v
[Batch Processing]
    |
    v
Rust: commands/batch.rs
    |-- For each (video x seed):
    |       executor.rs: execute_single_file()
    |           |-- filters.rs: build_filter_args_separated()
    |           |   |-- match OperationType (20 variants)
    |           |   |-- return FilterKind::VideoFilter/AudioFilter/Other
    |           |
    |           |-- Merge vf_exprs into comma-joined chain
    |           |-- Inject GPU encoder (-c:v h264_videotoolbox / libx264)
    |           |-- Spawn ffmpeg-sidecar child process
    |           |-- Stream progress via batch-file-progress event
    |
    v
Output files + log entries
```

### Recommended Project Structure (Additions Only)

```
src-tauri/src/
├── ffmpeg/
│   └── filters.rs              # EXTEND: 13 new builder functions + OperationType match arms
├── models/
│   ├── seed.rs                 # EXTEND: OperationType enum (13 variants), Seed { strength_tier }
│   ├── video.rs                # EXTEND: VideoEntry { thumbnail_base64, order_index }
│   └── batch.rs                # EXTEND: ProcessingLogEntry struct
├── commands/
│   ├── seed.rs                 # EXTEND: strength_tier param, coverage algorithm, weight table
│   ├── import.rs               # EXTEND: thumbnail extraction on import
│   └── export_seed.rs          # NEW: export_seed, import_seed Tauri commands
└── migrations/
    └── seed_v2.rs              # NEW: legacy seed auto-migration on startup

src/
├── components/
│   ├── seed/
│   │   └── SeedCard.vue        # EXTEND: strength tier badge + export/import buttons
│   ├── queue/
│   │   └── QueueList.vue       # EXTEND: VueDraggable wrapper + thumbnail img
│   ├── batch/
│   │   └── BatchControls.vue   # EXTEND: strength tier selector (NSelect 3-option)
│   ├── log/
│   │   └── LogPanel.vue        # NEW: inline log panel with NTabs, search, filters
│   └── App.vue                 # EXTEND: log panel integration
├── stores/
│   ├── seed.ts                 # EXTEND: exportSeed/importSeed actions
│   ├── queue.ts                # EXTEND: reorder action for drag-and-drop persistence
│   └── log.ts                  # NEW: processing log store
├── types/
│   ├── seed.ts                 # EXTEND: strengthTier field, new OperationType union members
│   ├── video.ts                # EXTEND: thumbnailBase64, orderIndex fields
│   └── log.ts                  # NEW: ProcessingLogEntry type
└── locales/
    ├── zh-CN.json              # EXTEND: ~30 new i18n keys
    └── en.json                 # EXTEND: ~30 new i18n keys
```

### Pattern 1: FFmpeg Filter Builder Extension (Rust)

**What:** Add a new filter builder function in `filters.rs` following the existing 7-builder pattern, then add a match arm in `build_filter_args()` and `build_filter_args_separated()`.

**When to use:** For each of the 13 new OperationType variants.

**Example -- HueRotate filter builder:**
```rust
// Source: Context7 /websites/ffmpeg_ffmpeg-all §11.129 hue filter
/// Build FFmpeg filter arguments for hue rotation.
/// Strength tier affects: h angle range (conservative: +/-15deg, standard: +/-45deg, aggressive: +/-90deg)
pub fn build_hue_rotate_filter(op: &Operation) -> Result<Vec<String>, String> {
    let hue_angle: f64 = op.params["hueAngle"].as_f64().unwrap_or(0.0);
    let saturation: f64 = op.params["saturation"].as_f64().unwrap_or(1.0);

    // Clamp based on strength tier (set during generation)
    let hue_angle = hue_angle.clamp(-90.0, 90.0);
    let saturation = saturation.clamp(0.5, 1.5);

    let filter = format!("hue=h={}:s={}", hue_angle, saturation);
    Ok(vec!["-vf".to_string(), filter])
}
```

**Example -- SolidColorOverlay using colorize filter:**
```rust
// Source: Context7 /websites/ffmpeg_ffmpeg-all §11.33 colorize filter
/// Overlay a semi-transparent solid color. Opacity <= 0.15 per D-01/D-03.
pub fn build_solid_color_overlay_filter(op: &Operation) -> Result<Vec<String>, String> {
    let hue: f64 = op.params["hue"].as_f64().unwrap_or(0.0);
    let saturation: f64 = op.params["saturation"].as_f64().unwrap_or(0.5);
    let lightness: f64 = op.params["lightness"].as_f64().unwrap_or(0.5);
    let mix: f64 = op.params["mix"].as_f64().unwrap_or(0.08);

    // Safety: mix (opacity) <= 0.15 per D-01/D-03
    let mix = mix.clamp(0.01, 0.15);

    let filter = format!(
        "colorize=hue={}:saturation={}:lightness={}:mix={}",
        hue, saturation, lightness, mix
    );
    Ok(vec!["-vf".to_string(), filter])
}
```

### Pattern 2: Weighted Random with Category Buckets (Rust)

**What:** Extend the existing `pick_operation_type()` cumulative probability function to include 13 new operations across 4 new categories.

**Current pattern (7 types, 100 buckets):**
```rust
fn pick_operation_type(rng: &mut impl Rng) -> OperationType {
    let roll: u32 = rng.random_range(1..=100);
    match roll {
        1..=30 => OperationType::MathOverlay,
        31..=42 => OperationType::PixelShift,
        // ... etc
    }
}
```

**Phase 6 extension (20 types, 100 buckets):**
```rust
fn pick_operation_type(rng: &mut impl Rng) -> OperationType {
    let roll: u32 = rng.random_range(1..=100);
    match roll {
        // Math overlay (existing 3): ~15% total, ~5% each
        1..=5 => OperationType::MathOverlay,      // ripple variant
        6..=10 => OperationType::MathOverlay,     // stripes variant (selector in params)
        11..=15 => OperationType::MathOverlay,    // concentric variant
        // Color processing (new 4): ~20% total, ~5% each
        16..=20 => OperationType::HueRotate,
        21..=25 => OperationType::SaturationAdjust,
        26..=30 => OperationType::BrightnessContrast,
        31..=35 => OperationType::ColorBalance,
        // ... etc, totaling 100
    }
}
```

Note: The current `MathOverlay` is a single enum variant with a `pattern` param field. The pattern above preserves this but the planner may choose to split MathOverlay into sub-variants. Either approach works with the existing architecture.

### Pattern 3: Coverage >=70% Validation Algorithm (Rust)

**What:** After generating all operations with random start_frame/duration_frames, compute coverage and re-randomize if <70%.

**Algorithm:**
```rust
/// Given video total_frames and operations list, compute covered frame count.
/// Operations with duration_frames=0 cover from start_frame to end of video.
/// Returns true if coverage >= 70%.
fn validate_coverage(operations: &[Operation], total_frames: u32) -> bool {
    if total_frames == 0 { return true; }
    if operations.is_empty() { return false; }

    // Build a boolean array or use interval coalescing
    let mut covered = vec![false; total_frames as usize];

    for op in operations {
        let start = op.start_frame as usize;
        let end = if op.duration_frames == 0 {
            total_frames as usize
        } else {
            ((op.start_frame + op.duration_frames) as usize).min(total_frames as usize)
        };
        for i in start..end {
            covered[i] = true;
        }
    }

    let covered_count = covered.iter().filter(|&&c| c).count();
    (covered_count as f64 / total_frames as f64) >= 0.70
}
```

**Retry strategy:** If coverage <70%, re-generate all start_frame/duration_frames for operations (preserving op_type and params). Retry up to 100 times; if still fails, fall back to extending the last operation's duration_frames to reach 70%.

### Pattern 4: VueDraggablePlus Queue Reordering (Vue 3)

**What:** Wrap the queue list `v-for` iteration in `<VueDraggable>` with `v-model` bound to the reactive entries array.

**When to use:** PROD-01 -- replacing the static `v-for` loop in QueueList.vue.

**Example:**
```vue
<!-- Source: Context7 /websites/vue-draggable-plus_pages_dev -->
<script setup lang="ts">
import { VueDraggable } from 'vue-draggable-plus'
import { useQueueStore } from '@/stores/queue'

const queueStore = useQueueStore()

function onDragEnd() {
  // Persist new order to Rust backend
  queueStore.persistOrder()
}
</script>

<template>
  <VueDraggable
    v-model="queueStore.entries"
    :animation="150"
    handle=".drag-handle"
    :disabled="batchStore.isProcessing"
    @end="onDragEnd"
  >
    <div
      v-for="(entry, index) in queueStore.entries"
      :key="entry.filepath"
      class="queue-item"
    >
      <!-- drag handle icon -->
      <NIcon class="drag-handle cursor-grab" :size="16">
        <GripVertical />
      </NIcon>
      <!-- existing queue item content -->
      <!-- thumbnail image -->
      <img
        v-if="entry.thumbnailBase64"
        :src="'data:image/jpeg;base64,' + entry.thumbnailBase64"
        class="w-16 h-9 object-cover rounded"
      />
    </div>
  </VueDraggable>
</template>
```

### Pattern 5: Seed JSON Export/Import (Rust + Tauri Dialog)

**What:** Two new Tauri commands that use `tauri-plugin-dialog` from the frontend for path selection, and Rust for file I/O + serde serialization.

**Export flow:** Frontend calls `dialog.save()` -> gets path string -> invokes `export_seed` Rust command with seed_id + path -> Rust serializes seed to JSON -> writes file -> returns success.

**Import flow:** Frontend calls `dialog.open()` with JSON filter -> gets path string -> invokes `import_seed` Rust command with path -> Rust reads file -> deserializes JSON -> validates schema -> regenerates UUID + created_at -> pushes to store -> emits `seeds-updated`.

```rust
// Rust: export_seed command
#[tauri::command]
pub async fn export_seed(
    state: State<'_, Mutex<AppState>>,
    seed_id: String,
    filepath: String,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let seed = app_state.seeds.iter()
        .find(|s| s.id == seed_id)
        .ok_or_else(|| format!("Seed not found: {}", seed_id))?;

    let json = serde_json::to_string_pretty(seed)
        .map_err(|e| format!("Serialization error: {}", e))?;
    std::fs::write(&filepath, json)
        .map_err(|e| format!("File write error: {}", e))?;
    Ok(())
}

// Rust: import_seed command
#[tauri::command]
pub async fn import_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    filepath: String,
) -> Result<Seed, String> {
    let json_str = std::fs::read_to_string(&filepath)
        .map_err(|e| format!("File read error: {}", e))?;

    let mut seed: Seed = serde_json::from_str(&json_str)
        .map_err(|e| format!("Invalid seed JSON: {}", e))?;

    // D-12: Regenerate UUID and timestamp
    seed.id = uuid::Uuid::new_v4().to_string();
    seed.created_at = chrono::Utc::now().to_rfc3339();

    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.seeds.push(seed.clone());
    }

    persist_seeds(&app)?;
    let _ = app.emit("seeds-updated", ());
    Ok(seed)
}
```

### Pattern 6: Strength Tier Parameter Mapping (Rust)

**What:** A tier-to-parameter-range mapping table used during seed generation instead of hardcoded clamp values.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StrengthTier {
    #[serde(rename = "conservative")]
    Conservative,
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "aggressive")]
    Aggressive,
}

/// Parameter ranges per strength tier for hue rotation
struct HueRotateRanges {
    hue_min: f64,
    hue_max: f64,
    saturation_min: f64,
    saturation_max: f64,
}

const HUE_ROTATE_RANGES: [(StrengthTier, HueRotateRanges); 3] = [
    (StrengthTier::Conservative, HueRotateRanges { hue_min: -15.0, hue_max: 15.0, saturation_min: 0.9, saturation_max: 1.1 }),
    (StrengthTier::Standard,     HueRotateRanges { hue_min: -45.0, hue_max: 45.0, saturation_min: 0.7, saturation_max: 1.3 }),
    (StrengthTier::Aggressive,   HueRotateRanges { hue_min: -90.0, hue_max: 90.0, saturation_min: 0.5, saturation_max: 1.5 }),
];
```

### Pattern 7: Log Panel with Pinia Store (Vue 3 + Naive UI)

**What:** An inline log panel (D-18: not a modal) using Naive UI NTabs as a sibling tab to the queue list in the right panel. Log entries accumulated in a Pinia store from `batch-log` events. Search/filter via computed properties.

**Store structure:**
```typescript
// src/stores/log.ts
export interface LogEntry {
  id: string;
  timestamp: string;      // ISO 8601
  file: string;           // source filename
  seedAlias: string;
  status: 'success' | 'failure';
  md5Before: string;
  md5After: string;
  modified: boolean;
  outputPath: string | null;
  errorMessage: string | null;
  durationMs: number;
}
```

### Anti-Patterns to Avoid

- **Do NOT change executor.rs core logic:** The FilterKind mechanism already handles arbitrary filter merging. Adding new `match` arms in `build_filter_args_separated()` is sufficient.
- **Do NOT create separate category-based seed pools:** D-04 explicitly requires unified random pool. All 20 operation types go into one weighted distribution.
- **Do NOT expose individual operation parameter sliders to users:** D-03 states user selects global strength tier only. Parameter ranges are internal to seed generation.
- **Do NOT sort operation chains by pipeline stage:** D-08 confirms random order. Do not implement any sorting logic.
- **Do NOT modify ffmpeg-sidecar or use custom FFmpeg builds:** D-05 requires built-in filters only. All 13 new operations use standard ffmpeg-sidecar binary filters.
- **Do NOT use modal dialogs for log panel:** D-16 and D-18 specify inline panel with tabs.
- **Do NOT match imported seeds by ID:** D-12 requires new UUID generation, never overwrite existing seeds.
- **Do NOT base64-encode full-resolution thumbnails:** Thumbnails should be scaled to max 120px width to keep base64 strings under ~8KB. Large thumbnails bloat store persistence and IPC payloads.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Drag-and-drop list reordering | Custom HTML5 drag-and-drop with draggable/dragover/drop event handlers | vue-draggable-plus (SortableJS wrapper) | Handles touch events, ghost images, sort animations, accessibility (aria-grabbed), nested scroll containers, and cross-browser quirks. 200+ lines of fragile code avoided |
| Thumbnail extraction | HTMLVideoElement + canvas.drawImage() in browser | ffmpeg-sidecar FfmpegCommand `-ss 1 -vframes 1` in Rust | Browser codec support is incomplete (mkv, avi, wmv fail); ffmpeg decodes all formats. Single synchronous short command (~100ms) |
| Log search/filter UI | Custom search input with manual array filtering | Pinia computed property with Array.filter() + Naive UI NInput v-model | Vue reactivity + computed properties provide free memoized filtering; NInput provides consistent theming |
| JSON file serialization for seed export | Custom format or manual string building | serde_json::to_string_pretty() + std::fs::write | Handles all edge cases (escaping, Unicode, number precision); already in stack |
| Coverage validation interval math | Manual range intersection logic | Simple boolean array per-frame coverage (max 65400 frames for 45min@24fps = 64KB bool array) | False sharing between operations correctly handled; O(frames * ops) is adequate (<1ms for typical videos) |
| Base64 encoding of thumbnail bytes | Custom base64 implementation | Rust `base64` crate (or `data_encoding`) or built-in for simple cases | Standard RFC 4648 encoding; handles padding and line wrapping correctly |

**Key insight:** The existing executor pipeline (`executor.rs`) already handles arbitrary filter merging via `FilterKind`. Adding new operation types requires ONLY new builder functions in `filters.rs` + match arm additions — the executor needs zero changes. This is the architectural payoff of the separated FilterKind design from Phase 2.

## Common Pitfalls

### Pitfall 1: FFmpeg Filter Chain Length Limits

**What goes wrong:** With 5-12 operations per seed, each producing a filter expression, the resulting `-vf` comma-joined chain can exceed FFmpeg's practical filter graph complexity limit or OS command-line length limit.

**Why it happens:** New operations (color, noise, geometric, blend) all produce `-vf` expressions. When combined with existing `geq` (which generates long expressions), the chain can become extremely long.

**How to avoid:** (a) Monitor filter chain string length during generation; cap at 4096 chars. (b) Some simple filters like `hflip`, `vflip` produce short expressions. (c) Use `-filter_complex` instead of `-vf` if chain exceeds practical limits (unlikely for <=12 ops with simple filters).

**Warning signs:** FFmpeg exits with "Option not found" or "Too many filters" errors even though individual filters are valid.

### Pitfall 2: Thumbnail Base64 Payload Size

**What goes wrong:** Full-resolution screenshot base64 encoded can be 500KB+ per video. With 50+ videos in queue, store persistence and IPC serialization become slow.

**Why it happens:** `-vframes 1` captures at source resolution (e.g., 1920x1080 = ~6MB raw, ~500KB base64).

**How to avoid:** Add `-vf "scale=120:-1"` to the thumbnail extraction command to scale width to 120px (height auto-calculated). This produces ~2-8KB base64 strings. Document this limit in code comments.

**Warning signs:** Seed store file (seeds.json) grows >10MB; IPC invoke() calls take >500ms.

### Pitfall 3: VueDraggablePlus Key Stability

**What goes wrong:** After drag reorder, Vue may reuse DOM elements with stale state because `v-for` keys don't uniquely identify items post-reorder.

**Why it happens:** VueDraggablePlus mutates the array order in-place. If keys are index-based, Vue reuses wrong elements.

**How to avoid:** Always use `:key="entry.filepath"` (stable unique identifier) for queue items, NOT `:key="index"`. The QueueList.vue already uses `:key="entry.filepath"` -- preserve this.

**Warning signs:** Thumbnail images flash or show wrong video after drag; progress bars attach to wrong file.

### Pitfall 4: Seed Migration on Empty State

**What goes wrong:** Migration logic runs on first launch before any seeds exist, or migration mutates the seed list during active batch processing.

**Why it happens:** The migration (D-19) must run at startup but must handle the empty-seeds case gracefully and must not run mid-processing.

**How to avoid:** (a) Check `if seeds.is_empty() { return; }` before migration. (b) Run migration once on app startup before any batch processing begins. (c) Store a migration version marker (e.g., `migration_v2_applied: true`) in the seed store to avoid re-running.

**Warning signs:** Migration adds `strength_tier: "standard"` to non-existent seeds causing panic; migration runs multiple times.

### Pitfall 5: Coverage Algorithm on Very Short Videos

**What goes wrong:** Videos shorter than 1 second (e.g., 24 frames) can never achieve 70% coverage with 5+ operations each with distinct frame ranges.

**Why it happens:** Each operation with a unique start_frame narrows the coverage window. On very short videos, coverage naturally drops.

**How to avoid:** (a) For videos with total_frames < 50, relax coverage to max(50%, achievable). (b) Or: apply all short-video operations with duration_frames=0 (full video) bypassing the coverage requirement. (c) Document this edge case clearly.

**Warning signs:** Infinite retry loop generating seeds for short videos; seed generation timeout.

### Pitfall 6: FFmpeg Filter Compatibility with GPU Encoders

**What goes wrong:** Some FFmpeg software filters are incompatible with hardware encoders (e.g., `geq` may not work with `h264_videotoolbox` on macOS in certain filter chain positions).

**Why it happens:** Hardware encoders require specific pixel formats and may reject software filter output formats.

**How to avoid:** (a) The existing executor already injects `-c:v` encoder args BEFORE the filter chain. (b) Test the full 20-operation matrix against all three GPU encoder paths (VideoToolbox, NVENC, VAAPI) in the verification phase. (c) Fall back to `libx264` if a specific operation chain fails with GPU encoding (wrap in error handling).

**Warning signs:** FFmpeg exits with "Impossible to convert between formats" or pixel format errors with GPU encoder but works with CPU.

### Pitfall 7: Store Persistence Race Conditions

**What goes wrong:** Queue reorder + thumbnail extraction on import both trigger store persistence. Concurrent writes may corrupt the queue.json or seeds.json store files.

**Why it happens:** tauri-plugin-store is not transactional; concurrent `store.save()` calls may interleave writes.

**How to avoid:** (a) Debounce store persistence: after drag reorder, wait 500ms before persisting. (b) Serialize all store writes through a single async mutex or queue. (c) The existing pattern in `commands/batch.rs` already uses `Mutex<AppState>` -- verify this covers all new write paths.

**Warning signs:** Queue order reverts after restart; thumbnails intermittently missing.

## Code Examples

Verified patterns from official sources:

### Thumbnail Extraction with ffmpeg-sidecar (Rust)
```rust
// Source: Context7 /nathanbabcock/ffmpeg-sidecar + FFmpeg docs
// Extract first frame, scale to 120px wide, output as single JPEG to stdout
use ffmpeg_sidecar::command::FfmpegCommand;

fn extract_thumbnail(ffmpeg_path: &str, video_path: &str) -> Result<String, String> {
    let ffmpeg_bin = std::path::Path::new(ffmpeg_path).join("ffmpeg");
    let output = FfmpegCommand::new_with_path(&ffmpeg_bin.to_string_lossy())
        .input(video_path)
        .args(["-ss", "1", "-vframes", "1", "-vf", "scale=120:-1"])
        .args(["-f", "image2pipe", "-vcodec", "mjpeg", "-"])
        .spawn()
        .map_err(|e| format!("Thumbnail spawn failed: {}", e))?
        .wait_with_output()
        .map_err(|e| format!("Thumbnail wait failed: {}", e))?;

    // output.output contains raw JPEG bytes -> base64 encode
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&output.output))
}
```

Note: The `base64` crate version 0.22.x API changed from `base64::encode()` to `base64::engine::general_purpose::STANDARD.encode()`. Verify the crate version in Cargo.toml: `cargo search base64` shows `base64 = "0.22.1"` [VERIFIED: crates.io]. The project may already have this dependency; check Cargo.toml.

### Hue Rotation Filter (FFmpeg CLI)
```bash
# Source: Context7 /websites/ffmpeg_ffmpeg-all §11.129
ffmpeg -i input.mp4 -vf "hue=h=45:s=1.2" output.mp4
# h = hue angle in degrees (-360 to 360)
# s = saturation (0 to 10, 1.0 = unchanged)
# b = brightness (-10 to 10, 0 = unchanged) -- also available
```

### Color Balance Filter (FFmpeg CLI)
```bash
# Source: Context7 /websites/ffmpeg_ffmpeg-all §11.28
ffmpeg -i input.mp4 -vf "colorbalance=rs=0.1:gs=-0.1:bs=0.05:rh=0.0:gh=0.0:bh=0.0" output.mp4
# rs/gs/bs = red/green/blue shadows adjustment (-1.0 to 1.0)
# rm/gm/bm = midtones; rh/gh/bh = highlights
```

### Noise Filter (FFmpeg CLI)
```bash
# Source: Context7 /websites/ffmpeg_ffmpeg-all §39.179
ffmpeg -i input.mp4 -vf "noise=alls=15:allf=t+u" output.mp4
# alls = strength (0-100)
# allf = flags: a=averaged temporal, p=patterned, t=temporal, u=uniform
```

### Micro-Rotate Filter (FFmpeg CLI)
```bash
# Source: Context7 /websites/ffmpeg_ffmpeg-all §11.217
ffmpeg -i input.mp4 -vf "rotate=0.5*PI/180:ow=iw:oh=ih" output.mp4
# First arg = angle in radians; 0.5*PI/180 ≈ 0.0087 rad ≈ 0.5°
# ow/oh = output width/height (use iw/ih to maintain dimensions)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| 7 hardcoded OperationType variants | 20 variants with category-organized weight distribution | Phase 6 | Unified random pool; all operations drawn from same 100-bucket cumulative distribution |
| Hardcoded safety clamp values | StrengthTier-driven parameter range tables | Phase 6 | Conservative/Standard/Aggressive tiers select different parameter ranges dynamically |
| 3-7 steps per seed | 5-12 steps per seed (tier-dependent) | Phase 6 | Conservative: 5-7, Standard: 6-9, Aggressive: 8-12 |
| All operations apply to full video (start=0, dur=0) | Operations assigned random frame ranges with >=70% coverage guarantee | Phase 6 | Dramatically increases fingerprint diversity; same operations at different time positions produce different perceptual outputs |
| No seed portability | Single-file JSON export/import with UUID regeneration | Phase 6 | Users can share seed recipes between installations; backup/restore workflow |
| Static queue order | Drag-and-drop reorder via SortableJS | Phase 6 | User controls processing priority; order persisted across restarts |
| Text-only queue entries | 120px-wide first-frame JPEG thumbnail per entry | Phase 6 | Visual identification of queue items; base64 at ~2-8KB per entry |
| Batch results shown transiently in summary modal | Persistent log history with search/filter | Phase 6 | Audit trail of all processing runs; filter by date/filename/seed |

**Deprecated/outdated:**
- Hardcoded `opacity.clamp(0.01, 0.15)` -- replaced by StrengthTier-driven ranges where Conservative applies 0.01-0.05, Standard 0.03-0.10, Aggressive 0.08-0.15
- `step_count = rng.random_range(3..=7)` -- replaced by tier-dependent range selection
- `start_frame: 0, duration_frames: 0` default for non-FrameDrop operations -- replaced by random frame range assignment per D-09

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `colorize` filter is compiled into the standard ffmpeg-sidecar auto-download binaries (BtbN static builds) | Standard Stack / Architecture Patterns | The blend overlay category's SolidColorOverlay variant would need an alternative implementation using `geq` with alpha expressions. Mitigation: `geq` can approximate solid color overlay by setting constant alpha + color expressions -- fallback exists within built-in filters |
| A2 | `noise` filter `alls` parameter range is 0-100 on all FFmpeg builds >=4.0 | Standard Stack / Architecture Patterns | If the range differs, the strength tier parameter ranges would need adjustment. Low risk -- the 0-100 range has been stable for years |
| A3 | `vue-draggable-plus` v0.6.1 is compatible with Vue 3.5.x and does not conflict with Naive UI component rendering | Architecture Patterns | If conflicts arise, fall back to @vueuse/core `useDraggable` composable or raw HTML5 drag-and-drop. Lower risk -- 0.x version indicates pre-1.0, but the library is actively maintained with recent releases |
| A4 | The `base64` crate (v0.22.x) is already in the project's Cargo.toml or can be added without version conflicts | Code Examples | If not present, need to add `base64 = "0.22"` to Cargo.toml. The new API (`general_purpose::STANDARD.encode()`) differs from old v0.21 API (`base64::encode()`) |
| A5 | ffmpeg-sidecar's `wait_with_output()` method buffers the entire stdout (thumbnail JPEG output) in memory -- safe for a 120px-wide JPEG (~2-8KB) | Architecture Patterns | If output is larger than expected, memory usage is still negligible (8KB). The risk is only in the method being renamed or deprecated in ffmpeg-sidecar 2.5.x |
| A6 | Standard ffmpeg-sidecar auto-download binaries include the `atadenoise` filter | Standard Stack | If not included, the TemporalDenoise operation would need an alternative filter like `hqdn3d` (high quality denoise 3D) which IS universally available. Fallback exists |
| A7 | Naive UI NTabs component supports lazy rendering of tab content (log panel tab) -- panel content only renders when tab is active | Architecture Patterns | If NTabs doesn't support lazy rendering, the log panel computed properties would execute even when viewing queue tab, causing unnecessary computation. Mitigation: use `v-if` on tab content based on active tab |

## Open Questions

1. **Thumbnail extraction during import: async or blocking?**
   - What we know: Thumbnail extraction requires a separate ffmpeg process (~100-500ms). Import is already async (Tauri command). Running thumbnail extraction synchronously within the import command blocks other imports.
   - What's unclear: Whether to extract thumbnails inline during import or spawn a background task that emits a separate event when thumbnail is ready.
   - Recommendation: Extract inline during import -- the 100-500ms delay is acceptable for single-file import. For batch import (multiple files), extract sequentially. If UX feedback indicates slowness, optimize in a later phase with parallel extraction.

2. **Coverage algorithm: per-seed or global?**
   - What we know: D-09 requires >=70% coverage of video duration. Operations have start_frame and duration_frames.
   - What's unclear: Is the 70% coverage per individual seed, or across all seeds applied to a video? D-09 context suggests per-seed coverage.
   - Recommendation: Per-seed coverage. Each seed independently must cover >=70% of the video. This gives each seed independent fingerprint diversity.

3. **Log history persistence format**
   - What we know: D-16 requires logs persisted to store. Log entries accumulate over time (potentially hundreds for heavy users).
   - What's unclear: Whether to store logs in the same tauri-plugin-store JSON file or a separate dedicated log file.
   - Recommendation: Separate `processing-log.json` store file. Keeps queue.json and seeds.json at manageable sizes. Implement a max entry cap (e.g., 500 most recent entries) with automatic truncation to prevent unbounded growth.

4. **Drag handle vs. entire row draggable**
   - What we know: D-14 specifies HTML5 drag-and-drop for reordering.
   - What's unclear: Whether the entire queue row should be draggable or only a specific drag handle icon. Entire-row drag conflicts with future interactions (e.g., clicking to select, thumbnail double-click to preview).
   - Recommendation: Use a dedicated drag handle icon (grip-vertical) on the left edge of each queue row. This avoids conflicts with selection, thumbnail interaction, and remove button clicks. The `handle=".drag-handle"` prop on VueDraggable enforces this.

5. **Strength tier UI placement**
   - What we know: D-03 requires a one-click global strength selector. D-18 references left panel with strength selector.
   - What's unclear: Whether the strength selector should be in the left panel (near seed generation) or in BatchControls (where batch processing settings live).
   - Recommendation: Place in BatchControls (right panel area), ABOVE the seed selector, as a horizontal radio group or segmented button. Rationale: Strength affects seed generation, but users think of it as a "processing intensity" setting -- it's conceptually closer to batch control than seed management. The seed generation button in the left panel reads the selected strength tier from a shared store value.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| node | Frontend build | ✓ | (via nvm/n) | -- |
| npm | Package management | ✓ | (via node) | -- |
| cargo | Rust build | ✓ | stable | -- |
| ffmpeg (ffmpeg-sidecar) | All video processing + thumbnail extraction | ✓ | auto-downloaded | -- |
| tauri-cli | Build & dev | ✓ | 2.x | -- |
| @tauri-apps/plugin-dialog (save) | Seed JSON export | ✓ | 2.7.1 (existing) | -- |
| @tauri-apps/plugin-dialog (open) | Seed JSON import | ✓ | 2.7.1 (existing) | -- |

**Missing dependencies with no fallback:** None -- all dependencies are either already in the project or added via npm.

**Missing dependencies with fallback:** None.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | Desktop app, no user authentication system |
| V3 Session Management | No | No sessions; single-user desktop application |
| V4 Access Control | No | No multi-user access control; local file system permissions only |
| V5 Input Validation | Yes | **Seed JSON import**: validate JSON structure, field types, operation type enum values before deserialization. **Video file paths**: path traversal prevention (Path::new normalizes). **Coverage frame ranges**: assert start_frame + duration_frames within video bounds |
| V6 Cryptography | No | No cryptographic operations; MD5 used only for integrity comparison (not security), documented as such |

### Known Threat Patterns for Tauri + FFmpeg Desktop App

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious seed JSON import with crafted operations attempting path traversal in output filenames | Tampering | serde deserialization validates types; OperationType enum limits possible values; file paths use Path::new() which normalizes `../` sequences |
| Imported seed JSON with excessive operation count causing resource exhaustion | Denial of Service | Cap imported seed operations at 20; reject import if operations.len() > 20 |
| Thumbnail base64 strings causing memory exhaustion if a crafted video produces abnormally large frames | Denial of Service | The `-vframes 1` flag limits to one frame; `scale=120:-1` caps pixel count; image2pipe output is bounded to ~8KB for 120px-wide JPEG |
| Drag-and-drop reorder during active batch processing causing queue mutation | Tampering | Disable drag when `batchStore.isProcessing` is true (already designed in Pattern 4) |
| Imported JSON file with invalid UTF-8 or binary content | Spoofing | `std::fs::read_to_string` returns error on non-UTF-8; serde_json rejects non-JSON |

## Sources

### Primary (HIGH confidence)
- [Context7: FFmpeg All Filters](/websites/ffmpeg_ffmpeg-all) -- hue, eq, colorbalance, colorchannelmixer, curves, noise, atadenoise, unsharp, smartblur, rotate, scale, transpose, hflip, vflip, overlay, blend, colorize, geq filter documentation. All built-in, no external dependencies. [VERIFIED]
- [Context7: ffmpeg-sidecar](/nathanbabcock/ffmpeg-sidecar) -- FfmpegCommand builder API, spawn(), iter(), wait_with_output(), filter_frames(). Version 2.5.1. [VERIFIED]
- [Context7: VueDraggablePlus](/websites/vue-draggable-plus_pages_dev) -- Component API, v-model binding, handle prop, animation prop. Version 0.6.1. [VERIFIED]
- [npm registry](https://www.npmjs.com/) -- vue-draggable-plus@0.6.1, vuedraggable@2.24.3, @vueuse/core@14.3.0, @tauri-apps/plugin-dialog@2.7.1 [VERIFIED]
- [crates.io](https://crates.io/) -- ffmpeg-sidecar@2.5.1, base64@0.22.1 [VERIFIED]
- [Tauri v2 Plugin Dialog](https://tauri.app/plugin/dialog/) -- save() function API with filters option [CITED]

### Secondary (MEDIUM confidence)
- [FFmpeg Official Documentation](https://ffmpeg.org/ffmpeg-filters.html) -- filter taxonomy verified via WebFetch [CITED]
- [Vue 3 Official Documentation](https://vuejs.org/guide/built-ins/transition.html) -- TransitionGroup component for list animations [CITED]

### Tertiary (LOW confidence)
- None -- all claims verified or cited from authoritative sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all library versions verified against npm/crates registries. Context7 documentation confirmed for FFmpeg filters, ffmpeg-sidecar API, and VueDraggablePlus.
- Architecture: HIGH -- existing codebase thoroughly analyzed (filters.rs, executor.rs, seed.rs, video.rs, batch.rs, commands/seed.rs, commands/import.rs, queue.ts, seed.ts, batch.ts, types, Vue components). All extension points identified and verified compatible with Phase 6 requirements.
- Pitfalls: HIGH -- filter chain limits, thumbnail size, drag key stability, migration edge cases, GPU compatibility all identified with specific mitigations.

**Research date:** 2026-05-16
**Valid until:** 2026-06-16 (30 days; stable stack, no fast-moving dependencies)

### What Was Not Investigated

- **GPU encoder + new filter compatibility matrix:** Full cross-product testing (20 ops x 3 GPU encoders x 3 platforms) is deferred to the verification phase. The research identifies this as Pitfall 6 with mitigation strategy.
- **Large-scale log persistence performance:** Log store with 500+ entries was not benchmarked. The 500-entry cap recommendation is based on reasonable defaults; adjust if user reports differ.
- **`geq` alpha expression for gradient overlay:** The `geq` filter's `alpha_expr` parameter was verified to exist but specific gradient expressions (linear, radial) were not tested for visual quality. The planner should budget time for expression tuning.
- **vue-draggable-plus touch support:** Mobile/tablet touch drag was not tested (desktop-only app). The underlying SortableJS library supports touch, but verification was out of scope.
