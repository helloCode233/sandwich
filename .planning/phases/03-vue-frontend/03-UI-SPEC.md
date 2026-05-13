---
phase: 03
phase_name: "Vue Frontend"
status: approved
design_system: "Naive UI 2.44.1"
tool: "naive-ui"
created: "2026-05-13"
---

# UI-SPEC: Phase 03 - Vue Frontend

## Design System

| Field | Value | Source |
|-------|-------|--------|
| Component library | Naive UI 2.44.1 | Phase 1 D-32, D-33; CLAUDE.md tech stack |
| Atomic CSS | UnoCSS 66.6.8 (presetUno) | Phase 1 D-06; uno.config.ts |
| Icon library | lucide-vue-next 1.0.0 | Established pattern in FFmpegStatus.vue, FFmpegDownload.vue |
| Theme | Naive UI darkTheme (NConfigProvider) | Phase 1 D-32; App.vue line 53 |
| I18n | vue-i18n 11.4.2 (zh-CN + en) | Phase 1 D-13; src/utils/i18n.ts |
| State management | Pinia 3.0.4 (Composition API stores) | Established pattern in src/stores/ffmpeg.ts |
| Font family | Naive UI defaults (system font stack) | No custom font configured |

## Spacing

8-point scale with Naive UI NSpace compatibility. All values in pixels.

| Token | Value | Usage |
|-------|-------|-------|
| xs | 4 | Inline icon gap, tag padding inner |
| sm | 8 | Compact element gap, badge offset, form label gap |
| md | 16 | Card internal padding, modal body padding |
| lg | 24 | Card-to-card gap in list, panel section gap |
| xl | 32 | Major section division, modal title-to-body |
| 2xl | 48 | Page-level section gap |

### Touch Targets

- Icon-only buttons (hover actions on seed cards, remove-from-queue): minimum 32x32px clickable area
- Standard buttons (NButton `size="medium"`): Naive UI default ~34px height
- Drag-drop zone: minimum 120px tall active area

## Typography

### Size Scale

| Token | Size | Weight | Line Height | Usage |
|-------|------|--------|-------------|-------|
| caption | 12px | 400 | 1.5 | Timestamps, file size metadata, operation count tag |
| body-sm | 14px | 400 | 1.5 | Seed operation summary, video metadata (duration/resolution), queue item details |
| body | 16px | 400 | 1.5 | Primary content: seed alias, video filename, form labels, button text, modal body |
| heading | 24px | 600 | 1.2 | Panel section titles, seed alias in expanded card view |

### Weights

| Weight | Token | Usage |
|--------|-------|-------|
| 400 | regular | Body text, metadata, descriptions, labels |
| 600 | semibold | Headings, card alias, CTA button text, selected count badge |

### Implementation

- Use Naive UI `NText` with `depth` prop for hierarchy (depth="1" = primary, depth="2" = secondary, depth="3" = tertiary)
- Use UnoCSS utility classes (`text-sm`, `text-xs`, `font-bold`) only for non-Naive-UI elements (e.g., custom drag zone overlay text)
- No custom headings outside Naive UI typography scale

## Color

### 60/30/10 Contract

| Role | Color | Token Reference | Coverage |
|------|-------|-----------------|----------|
| **60% Dominant surface** | `#101014` | `bg-[#101014]` | Full-page background behind all content |
| **30% Secondary** | `#1a1a1f` (approx) | Naive UI darkTheme card color | NCard surfaces, sidebar/panel backgrounds, NModal overlay, NSelect dropdown, NDataTable rows |
| **10% Accent** | `#2080f0` | Naive UI primary (blue) | See reserved-for list below |

### Accent Reserved For

The accent color `#2080f0` (Naive UI `type="primary"`) is **exclusively** used on these elements:

1. Primary CTA buttons: "生成种子" / "Generate Seed", "开始处理" / "Start Processing"
2. Selected seed card border (2px solid accent when card is active/selected)
3. Drag-drop zone active highlight border (during dragover)
4. Progress bar fill (NProgress during batch processing)
5. Current step indicator in multi-step flows

### Second Semantic Color

| Role | Color | Naive UI Type | Reserved For |
|------|-------|---------------|--------------|
| Destructive | `#d03050` | `type="error"` | Delete seed confirmation button, Clear queue confirmation button, Cancel batch button, failed file status indicator |

### Status Colors (Naive UI built-in)

| Status | Color | Usage |
|--------|-------|-------|
| Success | `#18a058` | Seed generated notification, video imported notification, batch complete |
| Warning | `#f0a020` | Low disk space warning, invalid video entry in queue |
| Error | `#d03050` | IPC call failure notification, file processing error |
| Info | `#2080f0` | General informational notifications |

## Icons

All icons from lucide-vue-next, wrapped in Naive UI `NIcon`. Concrete icon assignments:

| Icon | Usage |
|------|-------|
| `Sparkles` | Seed generation (generate button, empty state illustration) |
| `Clapperboard` | Video queue (empty state illustration) |
| `FolderOpen` | Output directory selector button |
| `Plus` | Add video button, generate seed icon |
| `Trash2` | Delete seed, remove from queue |
| `Copy` | Duplicate seed |
| `Pencil` | Rename seed |
| `Play` | Start batch processing |
| `Square` | Cancel/stop batch processing |
| `GripVertical` | Resize handle between panels |
| `AlertCircle` | Warning states, invalid file indicator |
| `CheckCircle` | Success notification, valid file indicator |
| `Upload` | Drag-drop visual cue |
| `Zap` | Seed selected/active indicator |

Icon sizing:
- Inline (within buttons, tags): 16px
- Standalone (empty state illustrations): 48px
- Card action buttons (hover): 16px

## Component Inventory

### Phase 3 Components (New)

```
src/
├── components/
│   ├── seed/
│   │   ├── SeedPanel.vue          # Left panel container (seed list + header)
│   │   ├── SeedCard.vue           # Individual seed card (alias, op summary, actions)
│   │   ├── SeedCardActions.vue    # Hover-revealed action buttons (rename, copy, delete)
│   │   └── SeedEmptyState.vue     # Empty state with generate CTA
│   ├── queue/
│   │   ├── VideoPanel.vue         # Right panel upper section (drop zone + queue list)
│   │   ├── DropZone.vue           # HTML5 drag-and-drop import area
│   │   ├── VideoList.vue          # Video queue scrollable list
│   │   ├── VideoListItem.vue      # Single video entry (metadata, status, remove)
│   │   └── VideoEmptyState.vue    # Empty state with drag instruction + file picker CTA
│   ├── batch/
│   │   ├── BatchControls.vue      # Right panel lower sticky section
│   │   └── BatchBanner.vue        # Top banner during processing (N/M processed)
│   └── layout/
│       └── MainLayout.vue         # Dual-panel resizable layout with NLayout
├── composables/
│   ├── useSeed.ts                 # Encapsulates invoke() for seed commands + seeds-updated event
│   ├── useQueue.ts                # Encapsulates invoke() for queue commands + queue-updated / video-imported events
│   └── useBatch.ts                # Encapsulates invoke() for batch commands + batch-progress / batch-complete / batch-cancelled events
├── stores/
│   ├── seed.ts                    # Pinia store: seed list, selected seed ID, CRUD actions
│   ├── queue.ts                   # Pinia store: video queue list, import/remove actions
│   └── batch.ts                   # Pinia store: batch config/status, progress snapshot
└── types/
    ├── seed.ts                    # Seed, Operation, OperationType (mirrors Rust models)
    ├── video.ts                   # VideoEntry, VideoMetadata, VideoStatus (mirrors Rust models)
    └── batch.ts                   # BatchConfig, BatchProgress, BatchResult, FileResult (mirrors Rust models)
```

### Naive UI Components Used

| Component | Usage |
|-----------|-------|
| `NLayout`, `NLayoutContent`, `NLayoutSider` | Main dual-panel layout with resizable split |
| `NCard` | Seed cards, batch control container |
| `NButton` | All actions (primary, default, error variants) |
| `NModal` | Destructive confirmations (delete seed, clear queue) |
| `NSelect` | Concurrency selector (1/2/3/4) |
| `NInput` | Seed rename inline input |
| `NTag` | Operation type tags on seed cards, video validity status |
| `NProgress` | Per-video progress bar structure (Phase 4 ready) |
| `NSpace` | All internal component spacing |
| `NText` | All text content (via depth prop for hierarchy) |
| `NIcon` | Icon wrapper (lucide icons) |
| `NSpin` | Loading indicator for IPC calls |
| `NScrollbar` | Queue list scrolling |
| `useMessage()` | Lightweight success/error toasts |
| `useNotification()` | Persistent notifications (low disk space warning) |

### Focal Point

The primary visual anchor of the main screen depends on state:
- **No seeds present**: SeedPanel empty state CTA ("生成第一个种子") is the focal point — centered in left panel, primary accent button
- **Seeds present, no videos**: DropZone in right panel is the focal point — largest interactive region with animated border on dragover
- **Both present, idle**: The "开始处理" / "Start Processing" CTA button in BatchControls is the focal point — largest primary button, full-width, at the natural end of the user's left-to-right scan path

## Layout Contract

### Dual-Panel Structure

```
+------------------------------------------------------------+
| NLayout (has-sider)                                        |
| +----------------------+-----------------------------------+ |
| | NLayoutSider (left)  | NLayoutContent (right)            | |
| | min: 280px           |                                   | |
| | default: 50%         | +-------------------------------+ | |
| | max: 60%             | | VideoPanel                    | | |
| | resizable: true      | | +---------------------------+ | | |
| |                      | | | DropZone (min 120px)      | | | |
| | SeedPanel            | | | +---------------------------+ | | |
| | +------------------+ | | | VideoList (scrollable)       | | |
| | | Header + CTA     | | | | VideoListItem              | | |
| | +------------------+ | | | VideoListItem              | | |
| | | SeedCard         | | | | ...                        | | |
| | | SeedCard         | | | +---------------------------+ | | |
| | | SeedCard         | | +-------------------------------+ | |
| | | ...              | | +-------------------------------+ | |
| | +------------------+ | | | BatchControls (sticky bottom)| | |
| |                      | | | Seed selector | Concurrency   | | |
| |                      | | | Output dir   | Start button  | | |
| |                      | | +-------------------------------+ | |
| +----------------------+-----------------------------------+ |
+------------------------------------------------------------+
```

### Panel Specifications

- **Splitter**: Draggable vertical bar between panels, 4px wide, hover highlight
- **Left panel (seeds)**: Scrollable vertical list of SeedCards. Header fixed at top (title + generate button). Panel background `#1a1a1f`, card background slightly lighter
- **Right panel (videos + batch)**:
  - Upper section: DropZone (120px min, full-width) + VideoList (scrollable, fills remaining space)
  - Lower section: BatchControls (sticky bottom, fixed height ~180px, always visible)
- **Window**: 1200x800 default, 900x600 minimum (Phase 1 D-12)

## Seed Card Contract

### Card States

| State | Visual |
|-------|--------|
| Default | `NCard` with muted border (darkTheme default), shows alias bold, operation summary tags, creation time |
| Hover | Card border brightens to `#2080f0` (accent), action buttons appear (rename/copy/delete) via CSS opacity transition |
| Selected | 2px solid `#2080f0` border, background slightly elevated (Naive UI hover color), persists independently of hover |
| Selected + Hover | Maintains solid accent border, action buttons visible |

### Card Content

```
+------------------------------------------+
| alias (heading, 600 weight)    [actions] |
| opTag1  opTag2  opTag3                   |
| created_at (caption, depth="3")          |
+------------------------------------------+
```

- **Alias**: Bold (600), 16px, single line with ellipsis overflow
- **Operation tags**: `NTag` with `type="info"` and `:bordered="false"`, one per operation type, compact size. Display up to 3 tags; if more than 3 ops, show "+N more" tag
- **Timestamp**: Caption size (12px), depth="3", format: relative time for today ("5分钟前"), absolute date otherwise ("2026-05-13")

### Action Buttons (Hover Reveal)

Displayed via CSS transition on card hover (`opacity: 0 -> 1`, `transform: translateX(-4px) -> 0`):

| Action | Icon | Behavior |
|--------|------|----------|
| Rename | `Pencil` | Inline NInput replaces alias text, Enter to confirm, Esc to cancel |
| Copy | `Copy` | Silently duplicates seed with re-randomized params, useMessage() success toast |
| Delete | `Trash2` | Opens NModal confirmation dialog |

## Video Queue Item Contract

### Item Layout

```
+----------------------------------------------------+
| filename (body, 600)                    [remove btn]|
| duration | resolution | size | codec     [status]  |
+----------------------------------------------------+
```

- **Filename**: 16px, 600 weight, ellipsis overflow for long names
- **Metadata row**: 14px body-sm, depth="2", format: "05:32 | 1920x1080 | 128.5 MB | h264"
- **Status tag**: `NTag` `type="success"` for Valid, `type="warning"` for Invalid
- **Remove button**: `NButton` with `Trash2` icon, `type="error"` `:text="true"` (icon-only minimal)

### Metadata Formatting

| Field | Format | Example |
|-------|--------|---------|
| Duration | MM:SS (or HH:MM:SS if >= 1 hour) | "05:32", "01:22:15" |
| Resolution | WxH | "1920x1080" |
| Size | Human-readable (auto unit) | "128.5 MB", "2.1 GB" |
| Codec | Uppercase codec name | "H264", "HEVC" |
| FPS | One decimal | "29.97" |

## Drag-Drop Zone Contract

### States

| State | Visual |
|-------|--------|
| Default (idle) | Dashed border (1px, `#333`), centered icon + instructional text, subtle background `rgba(32,128,240,0.03)` |
| Dragover | Border upgrades to 2px solid `#2080f0` (accent), background intensifies to `rgba(32,128,240,0.08)`, icon scales up 10%, text changes to "释放以导入" / "Drop to import" |
| Invalid drop | Brief red flash border (300ms `#d03050`), then returns to idle |

### Content

```
+------------------------------------------------------+
|                                                      |
|              [Upload icon, 48px]                     |
|                                                      |
|        拖拽视频到此处 / Drop videos here              |
|        支持 MP4, MOV, AVI, MKV, WEBM, FLV, WMV        |
|                                                      |
|              [添加视频 / Add Videos]                  |
|                                                      |
+------------------------------------------------------+
```

- Drop zone also functions as a click target to open file dialog (same as "添加视频" button)
- File dialog extension filter mirrors `SUPPORTED_EXTENSIONS` from import.rs

## Batch Controls Contract

### Layout (Sticky Bottom)

```
+------------------------------------------------------+
| 处理控制 / Processing Controls                       |
|                                                      |
| 种子: [NSelect: seed list by alias]                   |
| 并发: [NSelect: 1 | 2 | 3 | 4]  默认: 1              |
| 输出: /Users/.../sandwich-output/  [更改 / Change]    |
|                                                      |
| [开始处理 / Start Processing]  (primary, full-width)  |
+------------------------------------------------------+
```

- **Seed selector**: `NSelect` populated from seed store, shows alias, emits seed ID
- **Concurrency**: `NSelect` with options 1-4, default 1, persists to `tauri-plugin-store`
- **Output directory**: Read-only path display (elided middle if too long) + `NButton` "更改" using `FolderOpen` icon
- **Start button**: `NButton type="primary" size="large" block`, disabled when no seed selected or queue empty
- Entire section: `position: sticky; bottom: 0` with `bg-[#1a1a1f]` and top border

### Processing State (Batch Running)

When batch is active, BatchControls transforms:
- Start button replaced by "取消处理 / Cancel Processing" (`NButton type="error"`)
- BatchBanner appears at top of right panel: "处理中: N / M" with aggregate NProgress
- Each VideoListItem shows per-file `NProgress` bar (structure ready for Phase 4 live events)
- Seed selector and concurrency disabled during processing

## Copywriting Contract

### Primary CTAs

| Context | zh-CN | en |
|---------|-------|-----|
| Seed panel empty state | 生成第一个种子 | Generate Your First Seed |
| Seed panel header button | 生成种子 | Generate Seed |
| Queue empty state button | 添加视频 | Add Videos |
| Batch start button | 开始处理 | Start Processing |
| Batch cancel button | 取消处理 | Cancel Processing |
| Output dir change button | 更改目录 | Change Directory |

### Empty States

| Context | Icon | Title (zh-CN) | Title (en) | CTA |
|---------|------|---------------|------------|-----|
| No seeds | `Sparkles` (48px) | 还没有种子 | No Seeds Yet | 生成第一个种子 |
| No videos | `Clapperboard` (48px) | 还没有视频 | No Videos Yet | 拖拽视频到此处或点击添加 |
| All seeds deleted | `Sparkles` (48px) | 种子列表为空 | Seed List Empty | 生成种子 |
| Queue cleared | `Clapperboard` (48px) | 视频队列为空 | Video Queue Empty | 拖拽视频到此处或点击添加 |

### Error States (useMessage / useNotification)

| Error Scenario | zh-CN | en |
|----------------|-------|-----|
| IPC call failure (generic) | 操作失败：{error} | Operation failed: {error} |
| Import unsupported format | 不支持的格式 ".{ext}"，支持的格式：MP4, MOV, AVI, MKV, WEBM, FLV, WMV | Unsupported format ".{ext}". Supported: MP4, MOV, AVI, MKV, WEBM, FLV, WMV |
| Import no video stream | 文件不包含视频流：{filename} | File contains no video stream: {filename} |
| Import file not found | 文件不存在：{path} | File not found: {path} |
| Seed alias empty | 别名不能为空 | Alias cannot be empty |
| Batch no seed selected | 请先选择一个种子 | Please select a seed first |
| Batch queue empty | 队列为空，请先导入视频 | Queue is empty. Import videos first |
| Batch already running | 已有批处理正在进行中，请等待完成或取消 | A batch is already in progress |
| Low disk space | 磁盘空间不足（剩余不足 100 MB） | Low disk space (less than 100 MB available) |

### Destructive Actions

| Action | Confirmation Title (zh-CN) | Confirmation Title (en) | Confirm Button | Cancel Button |
|--------|---------------------------|------------------------|----------------|---------------|
| Delete seed | 确定删除种子「{alias}」？此操作不可撤销。 | Delete seed "{alias}"? This cannot be undone. | 删除 (error type) | 取消 |
| Clear queue | 确定清空所有视频？队列中的 {count} 个视频将被移除。 | Clear all videos? {count} videos will be removed. | 清空 (error type) | 取消 |
| Cancel batch | 确定取消批处理？已完成处理的文件将保留。 | Cancel batch processing? Completed files will be preserved. | 取消处理 (warning type) | 继续处理 |

### Success Notifications

| Action | zh-CN | en |
|--------|-------|-----|
| Seed generated | 种子「{alias}」已生成 | Seed "{alias}" generated |
| Seed renamed | 种子已重命名为「{alias}」 | Seed renamed to "{alias}" |
| Seed copied | 种子「{alias}」已复制 | Seed "{alias}" copied |
| Seed deleted | 种子已删除 | Seed deleted |
| Video imported | 「{filename}」已导入 | "{filename}" imported |
| Video removed | 已移除「{filename}」 | "{filename}" removed |
| Queue cleared | 队列已清空 | Queue cleared |
| Output dir changed | 输出目录已更改为 {dir} | Output directory changed to {dir} |

## Interaction Contract

### Feedback Pyramid

Per CONTEXT.md D-09 and D-10:

| Action Type | Feedback Mechanism | Examples |
|-------------|-------------------|----------|
| **Destructive** (irreversible) | NModal confirmation dialog | Delete seed, Clear queue, Cancel batch |
| **Mutating** (non-destructive) | useMessage() success toast | Generate seed, Copy seed, Rename seed, Import video, Remove video |
| **Error** | useMessage() error toast + descriptive text | IPC failures, validation errors, import failures |
| **Progress** | Inline NProgress + BatchBanner | Batch processing (structure for Phase 4) |

### IPC Call Pattern

All Tauri `invoke()` calls live in composables (`useSeed`, `useQueue`, `useBatch`), never in components directly. Pattern established in Phase 1 `useFfmpeg.ts`:

```typescript
// Composables wrap invoke() and event listeners
// Components consume composables via destructured methods
// Store is updated by composable after invoke() succeeds
```

### Event Listening

Composables subscribe to Rust-emitted events using Tauri `listen()`. Events this phase:

| Event | Composable | Triggers |
|-------|-----------|----------|
| `seeds-updated` | useSeed | Re-fetch seed list via `list_seeds` |
| `queue-updated` | useQueue | Re-fetch queue via `get_queue` |
| `video-imported` | useQueue | Optimistic prepend to queue store |
| `batch-progress` | useBatch | Update batch store progress (Phase 4 live, Phase 3 poll structure) |
| `batch-file-error` | useBatch | useMessage() error toast per file |
| `batch-complete` | useBatch | useMessage() success + reset batch state |
| `batch-cancelled` | useBatch | useMessage() info + reset batch state |
| `low-disk-space` | useQueue | useNotification() warning |

### Icon-Only Button Labels

All icon-only action buttons MUST have a text label via Naive UI `NTooltip` using the corresponding i18n key:

| Location | Buttons | Tooltip i18n Key |
|----------|---------|-----------------|
| SeedCard hover actions | Rename (Pencil) | `seed.rename` |
| SeedCard hover actions | Copy (Copy) | `seed.copy` |
| SeedCard hover actions | Delete (Trash2) | `seed.delete` |
| VideoListItem remove | Remove (Trash2) | `queue.remove` |

`NTooltip` placement: `trigger="hover"` with `placement="top"`. Tooltip text shows on 500ms hover delay.

### Seed Selection

- Clicking a SeedCard selects it (single selection). Only one seed can be selected at a time.
- Selected seed visual: 2px solid `#2080f0` border, background elevated
- BatchControls reads selected seed ID from seed store (computed `selectedSeedId`)
- Deselect by clicking the same card again, or selecting a different card

### Rename Flow

1. Click pencil icon on seed card
2. Alias text transforms to `NInput` inline (same card, same position)
3. Input pre-filled with current alias, auto-focused
4. Enter confirms rename (calls `rename_seed` via composable)
5. Esc cancels, reverts to original alias
6. Empty alias rejected with inline error text

## TypeScript Types Contract

Mirror Rust models with `camelCase` field names (Rust `#[serde(rename_all = "camelCase")]` ensures consistency).

### src/types/seed.ts
```typescript
export interface Seed {
  id: string;
  alias: string;
  operations: Operation[];
  createdAt: string; // ISO 8601
}

export interface Operation {
  opType: OperationType;
  startFrame: number;
  durationFrames: number;
  params: Record<string, unknown>;
}

export type OperationType =
  | 'mathOverlay'
  | 'pixelShift'
  | 'frameDrop'
  | 'gopModify'
  | 'metadataErase'
  | 'audioTweak'
  | 'remux';
```

### src/types/video.ts
```typescript
export interface VideoEntry {
  filename: string;
  filepath: string;
  metadata: VideoMetadata;
  status: VideoStatus;
}

export interface VideoMetadata {
  durationSecs: number;
  width: number;
  height: number;
  sizeBytes: number;
  codec: string;
  fps: number;
}

export type VideoStatus = 'valid' | 'invalid';
```

### src/types/batch.ts
```typescript
export interface BatchConfig {
  seedId: string;
  outputDir: string;
  concurrency: number; // 1-4
}

export interface BatchProgress {
  total: number;
  completed: number;
  succeeded: number;
  failed: number;
  currentFile: string | null;
}

export interface BatchResult {
  succeeded: string[];
  failed: FileResult[];
}

export interface FileResult {
  file: string;
  seed: string;
  error: string;
}
```

## I18n Key Structure

### New Keys (to add to src/locales/zh-CN.json and src/locales/en.json)

```
seed:
  title: "种子列表" / "Seeds"
  empty: "还没有种子" / "No Seeds Yet"
  emptyCta: "生成第一个种子" / "Generate Your First Seed"
  generate: "生成种子" / "Generate Seed"
  generated: "种子「{alias}」已生成" / "Seed \"{alias}\" generated"
  rename: "重命名" / "Rename"
  renamed: "种子已重命名为「{alias}」" / "Seed renamed to \"{alias}\""
  copy: "复制" / "Copy"
  copied: "种子「{alias}」已复制" / "Seed \"{alias}\" copied"
  delete: "删除" / "Delete"
  deleteConfirm: "确定删除种子「{alias}」？此操作不可撤销。" / "Delete seed \"{alias}\"? This cannot be undone."
  deleted: "种子已删除" / "Seed deleted"
  aliasPlaceholder: "输入种子别名" / "Enter seed alias"
  aliasEmpty: "别名不能为空" / "Alias cannot be empty"

queue:
  title: "视频队列" / "Video Queue"
  empty: "还没有视频" / "No Videos Yet"
  emptyInstruction: "拖拽视频到此处或点击添加" / "Drop videos here or click to add"
  addVideo: "添加视频" / "Add Videos"
  remove: "移除" / "Remove"
  clearAll: "清空全部" / "Clear All"
  clearConfirm: "确定清空所有视频？队列中的 {count} 个视频将被移除。" / "Clear all videos? {count} videos will be removed."
  cleared: "队列已清空" / "Queue cleared"
  imported: "「{filename}」已导入" / "\"{filename}\" imported"
  removed: "已移除「{filename}」" / "\"{filename}\" removed"
  invalid: "文件已失效" / "File invalid"
  valid: "有效" / "Valid"

import:
  dropHere: "拖拽视频到此处" / "Drop videos here"
  dropActive: "释放以导入" / "Drop to import"
  supportedFormats: "支持 MP4, MOV, AVI, MKV, WEBM, FLV, WMV"
  unsupportedFormat: "不支持的格式 \".{ext}\"，支持的格式：MP4, MOV, AVI, MKV, WEBM, FLV, WMV"
  noVideoStream: "文件不包含视频流：{filename}"
  fileNotFound: "文件不存在：{path}"

batch:
  title: "处理控制" / "Processing Controls"
  selectSeed: "选择种子" / "Select Seed"
  concurrency: "并发数" / "Concurrency"
  outputDir: "输出目录" / "Output Directory"
  changeDir: "更改目录" / "Change Dir"
  start: "开始处理" / "Start Processing"
  cancel: "取消处理" / "Cancel Processing"
  cancelConfirm: "确定取消批处理？已完成处理的文件将保留。" / "Cancel batch processing? Completed files will be preserved."
  noSeedSelected: "请先选择一个种子" / "Please select a seed first"
  queueEmpty: "队列为空，请先导入视频" / "Queue is empty. Import videos first"
  alreadyRunning: "已有批处理正在进行中，请等待完成或取消" / "A batch is already in progress"
  processing: "处理中" / "Processing"
  progress: "{completed}/{total}" / "{completed}/{total}"
  completed: "批处理完成" / "Batch Complete"
  cancelled: "批处理已取消" / "Batch Cancelled"
  defaultOutputDir: "~/Videos/sandwich-output/"

notification:
  lowDiskSpace: "磁盘空间不足（剩余不足 100 MB）" / "Low disk space (less than 100 MB available)"
  operationFailed: "操作失败：{error}" / "Operation failed: {error}"
```

## Style Patterns

### Card Styles (UnoCSS utilities augmenting Naive UI)

- Seed card selected: class `border-[#2080f0]! border-2!` overrides Naive UI default border
- Drag zone idle: class `border-dashed border-[#333]`
- Drag zone active: class `border-solid border-[#2080f0] border-2`
- Background: class `bg-[#101014]` (page), `bg-[#1a1a1f]` (panel surface)
- Text hierarchy via Naive UI `NText depth` prop, not UnoCSS color classes
- Hover transitions: `transition-all duration-200` on interactive elements

### Scrollbar

- Use Naive UI `NScrollbar` for VideoList (consistent dark theme scrollbar)
- Left panel seed list also uses NScrollbar

### Responsive Within Constraints

Window minimum is 900x600 (Phase 1 D-12). At minimum width:
- Left panel: 280px (min-width constraint on NLayoutSider)
- Right panel: fills remainder
- Drop zone compresses vertically but retains 100px minimum
- Seed cards wrap action buttons if needed (fallback: always-visible on narrow)

## Registry

| Registry | URL | Blocks | Safety Gate |
|----------|-----|--------|-------------|
| (none - Naive UI is not a shadcn registry) | -- | -- | N/A - project uses Naive UI 2.44.1, no third-party shadcn registries |

## Pre-Populated From

| Source | Decisions Used |
|--------|---------------|
| CONTEXT.md (03-CONTEXT.md) | D-01 to D-13: layout, import, seed display, empty states, feedback, batch controls, progress structure |
| RESEARCH.md | Not applicable (no separate RESEARCH.md for Phase 3) |
| REQUIREMENTS.md | UI-01 (dual-panel), UI-02 (dark theme) |
| Phase 1 CONTEXT (01-CONTEXT.md) | D-32 (darkTheme), D-33 (Naive UI + UnoCSS + vue-i18n + Pinia), D-12 (window sizing) |
| Existing codebase | Store pattern, composable pattern, type interface convention, i18n structure, UnoCSS config, Naive UI import style, icon library |
| User defaults (sensible defaults) | Spacing scale, typography scale, color 60/30/10 split, icon assignments |

---

*UI-SPEC created: 2026-05-13*
*Ready for: gsd-ui-checker validation, gsd-planner consumption*
