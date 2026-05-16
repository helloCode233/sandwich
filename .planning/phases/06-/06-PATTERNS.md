# Phase 06: 增强指纹修改 - Pattern Map

**Mapped:** 2026-05-16
**Files analyzed:** 25 (new + modified)
**Analogs found:** 25 / 25

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/ffmpeg/filters.rs` (extend) | utility (FFmpeg filter builder) | transform | itself (7 existing builder functions) | exact |
| `src-tauri/src/models/seed.rs` (extend) | model | N/A (data struct) | itself (OperationType enum + Seed struct) | exact |
| `src-tauri/src/models/video.rs` (extend) | model | N/A (data struct) | itself (VideoEntry struct + serde attrs) | exact |
| `src-tauri/src/models/batch.rs` (extend) | model | N/A (data struct) | itself (FileSuccess struct pattern) | exact |
| `src-tauri/src/commands/seed.rs` (extend) | controller (Tauri command) | request-response | itself (generate_seed + persist_seeds) | exact |
| `src-tauri/src/commands/import.rs` (extend) | controller (Tauri command) | request-response | itself (import_video command) | exact |
| `src-tauri/src/commands/batch.rs` (extend) | controller (Tauri command) | event-driven | itself (start_batch cancel pattern) | exact |
| `src-tauri/src/commands/export_seed.rs` (new) | controller (Tauri command) | CRUD + file-I/O | `src-tauri/src/commands/seed.rs` | role-match |
| `src-tauri/src/migrations/seed_v2.rs` (new) | utility (migration) | batch-transform | `src-tauri/src/commands/seed.rs` (persist pattern) | data-flow-match |
| `src/components/seed/SeedCard.vue` (extend) | component (Vue SFC) | user-interaction -> store | itself (hover action buttons) | exact |
| `src/components/seed/SeedList.vue` | (unchanged — no Phase 6 modifications needed; strength badge + buttons on individual SeedCard components) | | | |
| `src/components/batch/BatchControls.vue` (extend) | component (Vue SFC) | user-interaction -> IPC | itself (NSelect + NButton pattern) | exact |
| `src/components/queue/QueueList.vue` (extend) | component (Vue SFC) | user-interaction -> store | itself (v-for + metadata display) | exact |
| `src/components/log/LogPanel.vue` (new) | component (Vue SFC) | computed/reactive display | `src/components/batch/BatchSummary.vue` | role-match |
| `src/components/MainLayout.vue` (extend) | component (layout) | N/A (layout structure) | itself (sider + content layout) | exact |
| `src/stores/seed.ts` (extend) | store (Pinia) | CRUD | itself (defineStore pattern) | exact |
| `src/stores/batch.ts` | (unchanged — isProcessing ref already exists from Phase 4/5; logStore is separate new store) | | | |
| `src/stores/queue.ts` (extend) | store (Pinia) | CRUD | itself (addEntry/removeEntry pattern) | exact |
| `src/stores/log.ts` (new) | store (Pinia) | event-driven | `src/stores/batch.ts` | role-match |
| `src/types/seed.ts` (extend) | type definition | N/A | itself (Seed + Operation interfaces) | exact |
| `src/types/video.ts` (extend) | type definition | N/A | itself (VideoEntry interface) | exact |
| `src/types/batch.ts` (extend) | type definition | N/A | itself (PerFileProgress interface) | exact |
| `src/types/log.ts` (new) | type definition | N/A | `src/types/batch.ts` (PerFileProgress interface) | role-match |
| `src/locales/zh-CN.json` (extend) | config (i18n) | N/A | itself (seed/batch key patterns) | exact |
| `src/locales/en.json` (extend) | config (i18n) | N/A | itself (mirrors zh-CN structure) | exact |

---

## Pattern Assignments

### 1. `src-tauri/src/ffmpeg/filters.rs` (extend: 13 builder fns + match arms)

**Analog:** `src-tauri/src/ffmpeg/filters.rs` (itself, lines 8-179)

**Imports pattern** (lines 1-6):
```rust
//! FFmpeg filter chain builders for all operation types.
use crate::models::seed::{Operation, OperationType};
```

**Core builder pattern** (lines 10-41, `build_math_overlay_filter`):
```rust
/// Build FFmpeg filter arguments for [operation name].
/// SEED-04: [safety constraint description].
pub fn build_[name]_filter(op: &Operation) -> Result<Vec<String>, String> {
    let param: f64 = op.params["param_name"].as_f64().unwrap_or(default);
    // Clamp to safety constraints (now tier-driven per D-03)
    let param = param.clamp(min, max);
    // Build filter expression
    let filter = format!("filter_name=option1={}:option2={}", val1, val2);
    Ok(vec!["-vf".to_string(), filter])
}
```

**Dispatch match pattern** (lines 122-133, `build_filter_args`):
```rust
pub fn build_filter_args(op: &Operation) -> Result<Vec<String>, String> {
    match op.op_type {
        OperationType::MathOverlay => build_math_overlay_filter(op),
        OperationType::PixelShift => build_pixel_shift_filter(op),
        // ... 7 existing variants ...
        // + 13 new variants added here
    }
}
```

**Separated dispatch pattern** (lines 148-179, `build_filter_args_separated`):
```rust
pub fn build_filter_args_separated(op: &Operation) -> Result<(FilterKind, Vec<String>), String> {
    match op.op_type {
        OperationType::MathOverlay => {
            let args = build_math_overlay_filter(op)?;
            let expr = args.get(1).cloned().unwrap_or_default();
            Ok((FilterKind::VideoFilter(expr), args))
        }
        // Most new ops (color, noise, geometric, blend) return FilterKind::VideoFilter
        // Follow existing pattern, extracting the 2nd arg as expression
        _ => {
            let args = build_filter_args(op)?;
            Ok((FilterKind::Other(args.clone()), args))
        }
    }
}
```

**Test pattern** (lines 182-297, `#[cfg(test)] mod tests`):
```rust
fn make_op(op_type: OperationType, params: serde_json::Value) -> Operation {
    Operation { op_type, start_frame: 0, duration_frames: 0, params }
}

#[test]
fn test_[filter_name]() {
    let op = make_op(OperationType::[Variant], serde_json::json!({"param": value}));
    let args = build_[name]_filter(&op).unwrap();
    assert!(args[0] == "-vf");  // video filter ops
    assert!(args[1].contains("filter_name="));
}
```

---

### 2. `src-tauri/src/models/seed.rs` (extend: OperationType enum + StrengthTier + Seed fields)

**Analog:** `src-tauri/src/models/seed.rs` (itself, lines 1-53)

**Struct serde pattern** (lines 5-16, Seed struct):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seed {
    pub id: String,
    pub alias: String,
    pub operations: Vec<Operation>,
    pub created_at: String,
    // NEW: pub strength_tier: StrengthTier,  // #[serde(default)] for backward compat
}
```

**Enum serde pattern** (lines 36-53, OperationType):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    // Existing 7 variants (keep as-is):
    MathOverlay,
    PixelShift,
    FrameDrop,
    GopModify,
    MetadataErase,
    AudioTweak,
    Remux,
    // NEW: 13+ variants for Phase 6:
    // Color processing: HueRotate, SaturationAdjust, BrightnessContrast, ColorBalance
    // Noise texture: FilmGrain, GaussianBlur, Sharpen
    // Geometric: MicroRotate, TinyScale, Flip
    // Blend overlay: SolidColorOverlay, GradientOverlay, WatermarkBlend
}
```

**NEW StrengthTier enum pattern** (follows OperationType pattern from lines 36-53):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StrengthTier {
    Conservative,  // serde: "conservative"
    Standard,      // serde: "standard"
    Aggressive,    // serde: "aggressive"
}

impl Default for StrengthTier {
    fn default() -> Self { StrengthTier::Standard }
}
```

**Forward compatibility:** New fields on `Seed` must use `#[serde(default)]` per D-20 to ensure old app versions reading new-format seeds don't fail deserialization.

---

### 3. `src-tauri/src/models/video.rs` (extend: thumbnail_base64 + order_index)

**Analog:** `src-tauri/src/models/video.rs` (itself, lines 7-15)

**Existing struct + serde pattern** (lines 7-15):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoEntry {
    pub filename: String,
    pub filepath: String,
    pub metadata: VideoMetadata,
    pub status: VideoStatus,
    // NEW:
    // #[serde(default, skip_serializing_if = "Option::is_none")]
    // pub thumbnail_base64: Option<String>,
    // pub order_index: u32,  // NEW: 0-based display order
}
```

**`#[serde(default)]` on new fields** ensures backward compatibility when old save files without these fields are loaded. `thumbnail_base64` uses `Option` with `skip_serializing_if` to avoid serializing empty values. `order_index` defaults to `0u32` via serde default.

---

### 4. `src-tauri/src/models/batch.rs` (extend: ProcessingLogEntry struct)

**Analog:** `src-tauri/src/models/batch.rs` (itself, lines 43-73, FileSuccess + PerFileProgress patterns)

**Derive + serde struct pattern** (lines 56-73, FileSuccess):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSuccess {
    pub path: String,
    pub seed_alias: String,
    pub source_file: String,
    pub md5_before: String,
    pub md5_after: String,
    pub modified: bool,
    pub size_bytes: u64,
}
```

**NEW struct matches same pattern:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessingLogEntry {
    pub id: String,           // UUID
    pub timestamp: String,    // ISO 8601
    pub file: String,         // source filename
    pub seed_alias: String,
    pub status: String,       // "success" or "failure"
    pub md5_before: String,
    pub md5_after: String,
    pub modified: bool,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: u64,
}
```

**Serializable-only for events** (lines 75-94, PerFileProgress):
```rust
#[derive(Debug, Clone, Serialize)]  // Note: Serialize only (event emission, not persistence)
#[serde(rename_all = "camelCase")]
```

---

### 5. `src-tauri/src/commands/seed.rs` (extend: strength_tier + coverage + weights)

**Analog:** `src-tauri/src/commands/seed.rs` (itself, lines 1-265)

**Imports pattern** (lines 1-7):
```rust
use rand::prelude::*;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;
use crate::models::seed::{Operation, OperationType, Seed};
use crate::state::AppState;
```

**Weighted random pattern** (lines 11-26, `pick_operation_type`):
```rust
fn pick_operation_type(rng: &mut impl Rng) -> OperationType {
    let roll: u32 = rng.random_range(1..=100);
    match roll {
        1..=30 => OperationType::MathOverlay,
        31..=42 => OperationType::PixelShift,
        // ... existing 7 types ...
        // NEW: 13+ types with new weight distribution (D-17):
        // Color (new): ~20% => 16..=35
        // Noise (new): ~15%  => 36..=50
        // Geometric (new): ~15% => 51..=65
        // Blend (new): ~10%  => 66..=75
        // Existing + remaining old: ~25% => 76..=100
        _ => unreachable!("roll is 1..=100"),
    }
}
```

**Command signature + state access pattern** (lines 34-73, `generate_seed`):
```rust
#[tauri::command]
pub async fn generate_seed(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    // NEW param:
    // strength: String,  // "conservative" | "standard" | "aggressive"
) -> Result<Seed, String> {
    let mut rng = rand::rng();
    // Tier-dependent step count: conservative=5..=7, standard=6..=9, aggressive=8..=12
    let step_count = rng.random_range(min_steps..=max_steps);
    // ... generate operations, validate coverage >=70%, build seed
}

// State lock + persist + emit pattern (lines 60-71):
{
    let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    app_state.seeds.push(seed.clone());
}
persist_seeds(&app)?;
let _ = app.emit("seeds-updated", ());
Ok(seed)
```

**Write-through persist pattern** (lines 253-265, `persist_seeds`):
```rust
fn persist_seeds(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    let store = app.store("seeds.json").map_err(|e| format!("Failed to open seeds store: {}", e))?;
    let json = serde_json::to_value(&app_state.seeds).map_err(|e| format!("Serialization error: {}", e))?;
    store.set("seeds", json);
    store.save().map_err(|e| format!("Failed to save seeds: {}", e))?;
    Ok(())
}
```

**generate_operation frame assignment** (modified from lines 77-89):
```rust
// OLD: non-FrameDrop ops had (0, 0) = full video
// NEW (D-09): every op gets random start_frame/duration_frames
fn generate_operation(rng: &mut impl Rng, op_type: OperationType, total_frames: u32) -> Operation {
    let start = rng.random_range(0..total_frames);
    let remaining = total_frames - start;
    let dur = if remaining == 0 { 0 } else { rng.random_range(1..=remaining) };
    (start, dur)
}
```

---

### 6. `src-tauri/src/commands/export_seed.rs` (NEW: export + import)

**Analog:** `src-tauri/src/commands/seed.rs` (controller role, state lock + persist + emit pattern)

**Imports pattern** (copy from `seed.rs` lines 1-7 + add file I/O):
```rust
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;
use crate::models::seed::Seed;
use crate::state::AppState;
```

**Command pattern** (copy from `seed.rs` lines 149-172, `rename_seed`):
```rust
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
    // Cap operations at 20 (security: prevent resource exhaustion)
    if seed.operations.len() > 20 {
        return Err(format!("Imported seed has {} operations (max 20)", seed.operations.len()));
    }
    // Push + persist + emit (same pattern as seed.rs lines 60-71)
    {
        let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
        app_state.seeds.push(seed.clone());
    }
    persist_seeds(&app)?;
    let _ = app.emit("seeds-updated", ());
    Ok(seed)
}
```

**Note:** Imported seeds must be validated before insertion (D-12 + security). Check: valid OperationType variants, no path traversal in params, operations.len() <= 20.

---

### 7. `src-tauri/src/commands/import.rs` (extend: thumbnail extraction)

**Analog:** `src-tauri/src/commands/import.rs` (itself, lines 28-85)

**Existing import_video pattern** (lines 28-85):
```rust
#[tauri::command]
pub async fn import_video(
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
    filepath: String,
) -> Result<VideoEntry, String> {
    // 1. Validate extension (D-12)
    // 2. Check file exists
    // 3. Get FFmpeg dir
    // 4. Run ffprobe for metadata
    // 5. Check disk space
    // 6. Build VideoEntry
    // NEW: 7. Extract thumbnail via ffmpeg -ss 1 -vframes 1 -vf scale=120:-1
    // NEW: 8. Base64 encode result -> entry.thumbnail_base64
    // 9. Push to queue
    // 10. Persist + emit
    Ok(entry)
}
```

**Thumbnail extraction insertion point** (after line 69 `VideoEntry { ... }` construction, before queue push):

The thumbnail extraction uses `ffmpeg-sidecar` FfmpegCommand with `-ss 1 -vframes 1 -vf scale=120:-1 -f image2pipe -vcodec mjpeg -` and `wait_with_output()`. The `base64` crate is NOT in Cargo.toml -- it must either be added (`base64 = "0.22"`) or the thumbnail step can use existing infrastructure. Check: `md5` crate is present but `base64` is not. Planners must decide between adding `base64` crate or using `data_encoding` or a manual approach.

**Persist pattern** (lines 149-161, `persist_queue_import`): This function already persists the full queue. After adding thumbnails, the queue.json will contain base64 data -- the ~2-8KB per entry should be tested for total JSON size when queues reach 100+ entries.

---

### 8. `src-tauri/src/migrations/seed_v2.rs` (NEW: legacy seed migration)

**Analog:** `src-tauri/src/commands/seed.rs` (persist_seeds pattern, lines 253-265) + `src-tauri/src/lib.rs` (startup spawn pattern, lines 58-95)

**Module declaration pattern** (existing `mod.rs` line 1-6):
```rust
// src-tauri/src/mod.rs (or lib.rs `mod commands;` adjacent)
pub mod seed_v2;  // ADD to lib.rs modules list
```

**Migration function signature** (follows startup spawn from lib.rs lines 58-95):
```rust
/// Migrate legacy seeds (without strength_tier) to new format.
/// Runs once on app startup. Idempotent -- checks migration marker.
pub fn migrate_seeds(app: &AppHandle) -> Result<usize, String> {
    let state = app.state::<Mutex<AppState>>();
    let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;

    // D-19: Skip if no seeds
    if app_state.seeds.is_empty() { return Ok(0); }

    // Check migration marker in store
    let store = app.store("seeds.json")?;
    if store.get("migration_v2_applied").is_some() { return Ok(0); }

    let mut migrated = 0;
    for seed in &mut app_state.seeds {
        // serde default handles missing field, but explicit migration ensures
        // strength_tier is set even if struct default would be Standard
        // (migration logic depends on whether the field already exists via serde default)
        migrated += 1;
    }

    // Persist and mark migration done
    let json = serde_json::to_value(&app_state.seeds)?;
    store.set("seeds", json);
    store.set("migration_v2_applied", true);
    store.save()?;

    Ok(migrated)
}
```

**Call site in lib.rs** (after line 94, before Phase 5 GPU detection):
```rust
// --- Phase 6: Legacy seed migration (D-19) ---
let handle = app.handle().clone();
tauri::async_runtime::spawn(async move {
    match migrations::seed_v2::migrate_seeds(&handle) {
        Ok(0) => {} // no migration needed
        Ok(n) => { let _ = handle.emit("seeds-migrated", n); }
        Err(e) => { eprintln!("Seed migration error: {}", e); }
    }
});
```

---

### 9. `src/components/seed/SeedCard.vue` (extend: strength badge + export/import)

**Analog:** `src/components/seed/SeedCard.vue` (itself, lines 1-168)

**Hover button pattern** (lines 132-165, existing rename/copy/delete buttons):
```vue
<div v-show="isHovered" class="flex items-center gap-1 shrink-0" @click.stop>
  <NButton size="tiny" quaternary @click="onEvent">
    <template #icon>
      <NIcon :size="16"><SomeIcon /></NIcon>
    </template>
  </NButton>
</div>
```

**NEW export/import buttons** follow exact same pattern, placed before/after existing rename button:
```vue
<!-- Export button -->
<NButton size="tiny" quaternary @click="onExport">
  <template #icon><NIcon :size="16"><Download /></NIcon></template>
</NButton>
<!-- Import button -->
<NButton size="tiny" quaternary @click="onImport">
  <template #icon><NIcon :size="16"><Upload /></NIcon></template>
</NButton>
```

**NEW strength tier badge** (small tag displayed next to alias name, before operation tags):
```vue
<NTag v-if="seed.strengthTier" :type="strengthTagType" :bordered="false" size="tiny">
  {{ t(`seed.strength.${seed.strengthTier}`) }}
</NTag>
```

**Import handling** (calls `dialog.open()` with JSON filter, then invokes `import_seed`):
```typescript
import { open as openDialog } from '@tauri-apps/plugin-dialog';

async function onImport() {
  const path = await openDialog({
    filters: [{ name: 'Seed JSON', extensions: ['json'] }],
    multiple: false,
  });
  if (path && typeof path === 'string') {
    const seed = await importSeed(path);
    if (seed) message.success(t('seed.imported', { alias: seed.alias }));
  }
}
```

---

### 10. `src/components/batch/BatchControls.vue` (extend: strength tier selector)

**Analog:** `src/components/batch/BatchControls.vue` (itself, lines 1-251)

**NSelect pattern** (lines 168-176, seed selector):
```vue
<NSelect
  v-model:value="seedStore.selectedSeedIds"
  :options="seedOptions"
  :placeholder="t('batch.selectSeeds')"
  :disabled="batchStore.isProcessing"
  multiple filterable clearable
/>
```

**NEW strength tier selector** (single-select, 3 options):
```vue
<NSelect
  v-model:value="strengthTier"
  :options="strengthTierOptions"
  :placeholder="t('seed.strengthTier')"
  :disabled="batchStore.isProcessing"
  @update:value="onStrengthChange"
/>
```

**Options definition pattern** (lines 148-153, concurrencyOptions):
```typescript
const strengthTierOptions = [
  { label: t('seed.strength.conservative'), value: 'conservative' },
  { label: t('seed.strength.standard'), value: 'standard' },
  { label: t('seed.strength.aggressive'), value: 'aggressive' },
];
```

**Placement per D-18 + research recommendation:** ABOVE the seed selector NSelect in BatchControls area.

---

### 11. `src/stores/seed.ts` (extend: strengthTier + export/import)

**Analog:** `src/stores/seed.ts` (itself, lines 1-62)

**DefineStore Composition API pattern** (lines 5-61):
```typescript
export const useSeedStore = defineStore('seed', () => {
  const seeds = ref<Seed[]>([]);
  const selectedSeedIds = ref<string[]>([]);
  // NEW:
  const strengthTier = ref<'conservative' | 'standard' | 'aggressive'>('standard');

  // ... computed, actions, return { ... }
});
```

---

### 12. `src/stores/log.ts` (NEW: processing log store)

**Analog:** `src/stores/batch.ts` (exact same pattern -- defineStore + ref + computed)

**Store structure** (copy from `batch.ts` lines 1-91):
```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ProcessingLogEntry } from '@/types/log';

export const useLogStore = defineStore('log', () => {
  const entries = ref<ProcessingLogEntry[]>([]);
  const searchQuery = ref('');
  const statusFilter = ref<'all' | 'success' | 'failure'>('all');

  const entryCount = computed(() => entries.value.length);
  const filteredEntries = computed(() => {
    let result = entries.value;
    if (statusFilter.value !== 'all') {
      result = result.filter(e => e.status === statusFilter.value);
    }
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(e =>
        e.file.toLowerCase().includes(q) || e.seedAlias.toLowerCase().includes(q)
      );
    }
    return result;
  });

  const successCount = computed(() => entries.value.filter(e => e.status === 'success').length);
  const failureCount = computed(() => entries.value.filter(e => e.status === 'failure').length);

  function addEntry(entry: ProcessingLogEntry) {
    entries.value.unshift(entry);
    // Cap at 500 entries to prevent unbounded growth
    if (entries.value.length > 500) {
      entries.value = entries.value.slice(0, 500);
    }
  }

  function setEntries(list: ProcessingLogEntry[]) { entries.value = list; }
  function clearEntries() { entries.value = []; }

  return { entries, searchQuery, statusFilter, entryCount, filteredEntries, successCount, failureCount, addEntry, setEntries, clearEntries };
});
```

**Event listener subscription** (follows `useBatch` composable pattern, lines 22-25):
```typescript
// In composables/useLog.ts (NEW file, follows useBatch.ts pattern)
import { listen } from '@tauri-apps/api/event';
logUnlisten = await listen<LogEntryPayload>('batch-log', (event) => {
  store.addEntry({ ...event.payload, id: uuid(), /* ... */ });
});
```

---

### 13. `src/stores/queue.ts` (extend: reorder action)

**Analog:** `src/stores/queue.ts` (itself, lines 1-44)

**Existing CRUD pattern** (lines 17-27):
```typescript
function addEntry(entry: VideoEntry) { entries.value.push(entry); }
function removeEntry(index: number) {
  if (index >= 0 && index < entries.value.length) {
    entries.value.splice(index, 1);
  }
}
```

**NEW reorder action** (for VueDraggable `v-model` + persistence):
```typescript
/** Reorder entries (called after drag-and-drop). Persists new order via IPC. */
async function reorderEntries(newOrder: VideoEntry[]) {
  entries.value = newOrder;
  // Emit to Rust to persist new order to queue.json
  try {
    await invoke('reorder_queue', { entries: newOrder });
  } catch (err) {
    console.error('Failed to persist queue reorder:', err);
  }
}
```

**Note:** `reorder_queue` requires a NEW Rust command in `commands/queue.rs` (or adjacent) that accepts the full reordered entries array and persists it. Follows the exact pattern of `clear_queue` / `remove_from_queue` from `queue.rs` lines 32-84.

---

### 14. `src/components/queue/QueueList.vue` (extend: VueDraggable + thumbnails)

**Analog:** `src/components/queue/QueueList.vue` (itself, lines 1-252)

**Existing v-for pattern** (lines 171-241, queue items):
```vue
<div v-for="(entry, index) in store.entries" :key="entry.filepath" class="queue-item">
  <div class="flex items-center justify-between gap-3 py-2 px-3 rounded-md bg-[#1a1a1f]">
    <!-- File info -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <NText strong class="truncate text-sm">{{ entry.filename }}</NText>
        <NTag ...>{{ ... }}</NTag>
      </div>
      <NText depth="3" class="text-xs">{{ metadataLine(entry) }}</NText>
    </div>
    <!-- Remove button -->
    <NButton ...>
  </div>
</div>
```

**VueDraggablePlus wrapper** (per RESEARCH.md Pattern 4, lines 351-391):
```vue
<script setup lang="ts">
import { VueDraggable } from 'vue-draggable-plus';
// ...
const queueStore = useQueueStore();
</script>

<template>
  <VueDraggable
    v-model="queueStore.entries"
    :animation="150"
    handle=".drag-handle"
    :disabled="batchStore.isProcessing"
    @end="onReorderPersist"
  >
    <!-- existing v-for loop moves INSIDE VueDraggable -->
    <div v-for="(entry, index) in queueStore.entries" :key="entry.filepath" class="queue-item">
      <!-- NEW: drag handle grip icon on left edge -->
      <NIcon class="drag-handle cursor-grab" :size="14">
        <GripVertical />
      </NIcon>
      <!-- NEW: thumbnail img (before filename) -->
      <img
        v-if="entry.thumbnailBase64"
        :src="'data:image/jpeg;base64,' + entry.thumbnailBase64"
        class="w-12 h-7 object-cover rounded shrink-0"
      />
      <!-- existing content ... -->
    </div>
  </VueDraggable>
</template>
```

**Key stability:** `:key="entry.filepath"` is already correct (stable unique identifier per RESEARCH.md Pitfall 3).

---

### 15. `src/components/log/LogPanel.vue` (NEW: inline log panel)

**Analog:** `src/components/batch/BatchSummary.vue` (computed-driven display + NScrollbar list + filter display)

**Computed + conditional display pattern** (lines 11-47 in BatchSummary.vue):
```vue
<script setup lang="ts">
import { computed } from 'vue';
import { NText, NScrollbar, NInput, NSelect } from 'naive-ui';
import { useLogStore } from '@/stores/log';
import { useI18n } from 'vue-i18n';

const logStore = useLogStore();
const { t } = useI18n();

const statusFilterOptions = [
  { label: t('log.all'), value: 'all' },
  { label: t('log.success'), value: 'success' },
  { label: t('log.failure'), value: 'failure' },
];
</script>

<template>
  <div class="log-panel flex flex-col h-full">
    <!-- Filter bar -->
    <div class="flex items-center gap-2 px-3 py-2 shrink-0 border-b border-[#2a2a36]">
      <NInput v-model:value="logStore.searchQuery" :placeholder="t('log.searchPlaceholder')" size="small" clearable />
      <NSelect v-model:value="logStore.statusFilter" :options="statusFilterOptions" size="small" style="width: 100px" />
      <NText depth="3" class="text-xs shrink-0">{{ logStore.filteredEntries.length }} / {{ logStore.entryCount }}</NText>
    </div>
    <!-- Log entries (scrollable) -->
    <NScrollbar class="flex-1">
      <div class="space-y-1 px-3 py-1">
        <div v-for="entry in logStore.filteredEntries" :key="entry.id" class="log-entry">
          <!-- timestamp | file | seed | md5 before->after | status badge -->
        </div>
      </div>
    </NScrollbar>
  </div>
</template>
```

**Layout integration in MainLayout.vue** (copy from lines 86-100):
The log panel should be a sibling to the QueueList in the right panel, toggled via NTabs (Queue | Log). Replace the current single QueueList area with:
```vue
<NTabs v-model:value="rightPanelTab" type="line" size="small" class="queue-area-tabs">
  <NTabPane name="queue" :tab="t('queue.title')">
    <QueueList />
  </NTabPane>
  <NTabPane name="log" :tab="t('log.title')">
    <LogPanel />
  </NTabPane>
</NTabs>
```

---

### 16. `src/types/seed.ts` (extend: strengthTier + OperationType union)

**Analog:** `src/types/seed.ts` (itself, lines 1-24)

**Interface extension pattern** (lines 3-8):
```typescript
export interface Seed {
  id: string;
  alias: string;
  operations: Operation[];
  createdAt: string;
  strengthTier?: 'conservative' | 'standard' | 'aggressive'; // NEW (optional for backward compat)
}
```

**Union type extension pattern** (lines 17-24):
```typescript
export type OperationType =
  | 'mathOverlay' | 'pixelShift' | 'frameDrop'  // existing 7
  | 'gopModify' | 'metadataErase' | 'audioTweak' | 'remux'
  // NEW: 13+ variants (camelCase matching Rust serde)
  | 'hueRotate' | 'saturationAdjust' | 'brightnessContrast' | 'colorBalance'
  | 'filmGrain' | 'gaussianBlur' | 'sharpen'
  | 'microRotate' | 'tinyScale' | 'flip'
  | 'solidColorOverlay' | 'gradientOverlay' | 'watermarkBlend';
```

---

### 17. `src/types/video.ts` (extend: thumbnailBase64 + orderIndex)

**Analog:** `src/types/video.ts` (itself, lines 1-20)

**Interface extension** (lines 3-8):
```typescript
export interface VideoEntry {
  filename: string;
  filepath: string;
  metadata: VideoMetadata;
  status: VideoStatus;
  thumbnailBase64?: string;  // NEW: optional base64 JPEG, ~2-8KB
  orderIndex?: number;       // NEW: 0-based display order for drag-and-drop
}
```

---

### 18. `src/types/log.ts` (NEW: ProcessingLogEntry)

**Analog:** `src/types/batch.ts` (PerFileProgress interface, lines 34-49)

**Interface pattern** (copy from `batch.ts` lines 34-49):
```typescript
/** Processing log entry persisted to store for history panel (PROD-03).
 *  Mirrors Rust struct ProcessingLogEntry in src-tauri/src/models/batch.rs */
export interface ProcessingLogEntry {
  id: string;
  timestamp: string;    // ISO 8601
  file: string;         // source filename
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

---

### 19. `src/locales/zh-CN.json` + `en.json` (extend: ~30 keys)

**Analog:** `src/locales/zh-CN.json` (itself, key structure for seed/batch sections)

**Existing key structure** (lines 35-117):
```json
{
  "seed": {
    "title": "种子列表",
    "generate": "生成种子",
    "rename": "重命名",
    "copy": "复制",
    "delete": "删除",
    ...
  },
  "batch": {
    "title": "处理控制",
    ...
  }
}
```

**NEW Phase 6 keys** follow same nesting convention (`seed.*`, `batch.*`, `queue.*`, `log.*`):

| Key | zh-CN | en |
|-----|-------|-----|
| `seed.strengthTier` | 强度档位 | Strength Tier |
| `seed.strength.conservative` | 保守 | Conservative |
| `seed.strength.standard` | 标准 | Standard |
| `seed.strength.aggressive` | 激进 | Aggressive |
| `seed.export` | 导出 | Export |
| `seed.import` | 导入 | Import |
| `seed.exported` | 种子已导出 | Seed exported |
| `seed.imported` | 种子「{alias}」已导入 | Seed "{alias}" imported |
| `seed.importFailed` | 导入失败：{error} | Import failed: {error} |
| `seed.migrated` | {count} 个种子已升级 | {count} seeds upgraded |
| `queue.reorder` | 拖拽排序 | Drag to reorder |
| `queue.thumbnail` | 缩略图 | Thumbnail |
| `log.title` | 处理日志 | Processing Log |
| `log.empty` | 暂无日志 | No logs yet |
| `log.searchPlaceholder` | 搜索文件名/种子... | Search file/seed... |
| `log.all` | 全部 | All |
| `log.success` | 成功 | Success |
| `log.failure` | 失败 | Failure |
| `log.duration` | {minutes}分{seconds}秒 | {minutes}m {seconds}s |
| `log.status` | 状态 | Status |
| `log.outputPath` | 输出路径 | Output Path |
| `log.clearConfirm` | 确定清空所有日志？ | Clear all logs? |
| `log.cleared` | 日志已清空 | Logs cleared |

---

### 20. `src/composables/useSeed.ts` (extend: exportSeed/importSeed/generateSeed with strength)

**Analog:** `src/composables/useSeed.ts` (itself, lines 1-81)

**Function pattern** (lines 30-38, `generateSeed`):
```typescript
async function generateSeed(strength: string = 'standard'): Promise<Seed | null> {
  try {
    const seed = await invoke<Seed>('generate_seed', { strength });
    store.addSeed(seed);
    return seed;
  } catch (err) {
    console.error('Failed to generate seed:', err);
    return null;
  }
}
```

**NEW export/import functions** (follow same invoke + try/catch pattern as lines 30-74):
```typescript
async function exportSeed(seedId: string): Promise<boolean> {
  // Frontend: dialog.save() -> path -> invoke('export_seed', { seedId, filepath })
}
async function importSeed(path: string): Promise<Seed | null> {
  // Frontend: receives path from dialog.open() -> invoke('import_seed', { filepath })
}
```

---

## Shared Patterns

### Authentication / Guards
**Source:** Not applicable -- desktop app, no auth. All Tauri commands are local-only.

### Error Handling (Rust)
**Source:** `src-tauri/src/commands/seed.rs` lines 155-157
```rust
.map_err(|e| format!("Lock error: {}", e))?
.ok_or_else(|| format!("Seed not found: {}", seed_id))?
```
**Apply to:** All new Rust commands (`export_seed.rs`, `migrations/seed_v2.rs`). All Result errors return `String` type (not anyhow) for Tauri command compatibility. Use `format!("...", ...)` for error messages.

### Error Handling (Vue)
**Source:** `src/composables/useSeed.ts` lines 33-37
```typescript
catch (err) {
  console.error('Failed to [action]:', err);
  return null;
}
```
**Apply to:** All new composable functions and component event handlers. Components display user-facing errors via `useMessage`.

### State Lock + Persist + Emit (Rust)
**Source:** `src-tauri/src/commands/seed.rs` lines 60-71
```rust
{
    let mut app_state = state.lock().map_err(|e| format!("Lock error: {}", e))?;
    app_state.[mutation];
}
persist_[entity](&app)?;
let _ = app.emit("[event-name]", payload);
```
**Apply to:** All state-mutating Tauri commands.

### Pinia Composition API Store (Vue)
**Source:** `src/stores/seed.ts` lines 5-61
```typescript
export const use[Name]Store = defineStore('name', () => {
  const data = ref<Type[]>([]);
  const computed1 = computed(() => ...);
  function action1() { ... }
  return { data, computed1, action1 };
});
```
**Apply to:** `src/stores/log.ts` (new), extensions to existing stores.

### Vue SFC Component Pattern
**Source:** `src/components/seed/SeedCard.vue` lines 1-168
```vue
<script setup lang="ts">
import { ref } from 'vue';
import { NButton, NIcon, NText, ... } from 'naive-ui';
import { SomeIcon } from 'lucide-vue-next';
import { useStore } from '@/stores/[name]';
import { useI18n } from 'vue-i18n';
const { t } = useI18n();
// props, reactive state, functions
</script>
<template>
  <!-- Naive UI components with Tailwind utility classes -->
</template>
```
**Apply to:** All new/modified Vue components.

### Serde camelCase Convention
**Source:** All Rust model files
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Foo { pub bar_field: String } // -> TS: barField
```
**Apply to:** All new Rust structs (`ProcessingLogEntry`, extended fields on `Seed`, `VideoEntry`).

### TypeScript Interface Mirroring
**Source:** `src/types/seed.ts` lines 1-8, `src/types/video.ts` lines 1-8
```typescript
// Mirrors Rust structs in src-tauri/src/models/[name].rs
// All field names use camelCase matching #[serde(rename_all = "camelCase")]
export interface Foo {
  barField: string; // Rust: bar_field -> camelCase -> barField
}
```
**Apply to:** `src/types/log.ts` (new), extensions to existing types.

### i18n Key Convention
**Source:** `src/locales/zh-CN.json` and `en.json`
```json
{ "namespace": { "keyName": "translation" } }
// Usage in Vue: t('namespace.keyName')
```
**Apply to:** All new i18n keys follow `[feature].[camelCaseName]` nesting.

### Tauri Command Registration
**Source:** `src-tauri/src/lib.rs` lines 26-51
```rust
.invoke_handler(tauri::generate_handler![
    // Existing commands...
    // NEW Phase 6 commands:
    export_seed,
    import_seed,
    reorder_queue,
])
```
**Apply to:** All new `#[tauri::command]` functions must be registered in `lib.rs`.

### Tauri Store Persistence
**Source:** `src-tauri/src/commands/seed.rs` lines 253-265
```rust
fn persist_seeds(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let app_state = state.lock().map_err(...)?;
    let store = app.store("seeds.json").map_err(...)?;
    let json = serde_json::to_value(&app_state.seeds).map_err(...)?;
    store.set("seeds", json);
    store.save().map_err(...)?;
    Ok(())
}
```
**Apply to:** Queue reorder persistence, log store persistence, seed migration writes.

### Vitest Store Test Pattern
**Source:** `src/stores/__tests__/seed.test.ts` lines 1-79
```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSeedStore } from '@/stores/seed';

describe('useSeedStore', () => {
  beforeEach(() => { setActivePinia(createPinia()); });
  it('should do something', () => { const store = useSeedStore(); ... });
});
```
**Apply to:** Tests for new `logStore`, extended `seedStore` and `queueStore`.

---

## No Analog Found

None. All 25 files have close existing analogs in the codebase. Every file either extends itself (modifications to existing files) or follows a clear pattern from a sibling file in the same role.

| File | Analog Quality | Notes |
|------|---------------|-------|
| `export_seed.rs` | role-match | New controller file, but follows exact `seed.rs` command pattern (state lock + persist + emit) |
| `seed_v2.rs` | data-flow-match | New migration utility, but startup spawn pattern from `lib.rs` + persist from `seed.rs` |
| `log.ts` (store) | role-match | New store, but exact same `defineStore` pattern as `batch.ts` |
| `LogPanel.vue` | role-match | New component, but computed-driven display pattern from `BatchSummary.vue` |
| `log.ts` (types) | role-match | New type file, but interface pattern from `batch.ts` |

---

## Metadata

**Analog search scope:**
- `src-tauri/src/ffmpeg/` (filters.rs, executor.rs)
- `src-tauri/src/models/` (seed.rs, video.rs, batch.rs)
- `src-tauri/src/commands/` (seed.rs, import.rs, batch.rs, queue.rs, mod.rs)
- `src-tauri/src/` (lib.rs, state.rs, Cargo.toml)
- `src/stores/` (seed.ts, batch.ts, queue.ts, __tests__/)
- `src/composables/` (useSeed.ts, useBatch.ts, useQueue.ts)
- `src/types/` (seed.ts, video.ts, batch.ts)
- `src/components/` (seed/SeedCard.vue, seed/SeedList.vue, batch/BatchControls.vue, batch/BatchSummary.vue, queue/QueueList.vue, MainLayout.vue, App.vue)
- `src/locales/` (zh-CN.json, en.json)

**Files scanned:** 26 (source files read for pattern extraction)
**Pattern extraction date:** 2026-05-16

### Key Patterns Identified
1. **Rust filter builder:** Each `build_xxx_filter(&Operation) -> Result<Vec<String>, String>` + match arm in `build_filter_args()` and `build_filter_args_separated()`. FilterKind::VideoFilter for most new ops.
2. **Rust Tauri command:** `#[tauri::command] pub async fn name(State<Mutex<AppState>>, AppHandle, ...) -> Result<T, String>` + state lock mutation + `persist_*(&app)` + `app.emit("event", payload)`.
3. **Pinia store:** `defineStore('name', () => { ref() + computed() + action() -> return { ... } })`
4. **Vue SFC:** `<script setup lang="ts">` + Naive UI imports + lucide-vue-next icons + `useI18n()` for `$t()` + Tailwind utility classes + scoped `<style>`.
5. **Serde camelCase:** All Rust structs use `#[serde(rename_all = "camelCase")]`. TypeScript types use camelCase field names. `#[serde(default)]` on all new fields for backward compatibility.
6. **Tauri store persistence:** `app.store("name.json")` + `store.get/set/save`. Store files: `seeds.json`, `queue.json`, `sandwich-config.json`, and NEW: `processing-log.json`.
7. **Event system:** Rust emits events (`seeds-updated`, `queue-updated`, `batch-progress`, `batch-file-progress`, `batch-log`, etc.), Vue composables listen with `listen<Type>('event-name', callback)`.
8. **Weighted random:** Cumulative probability with `rng.random_range(1..=100)` and match arms. Extend to 100 buckets covering 20+ operation types per D-17 weight distribution.
