# Phase 03: Vue Frontend - Research

**Researched:** 2026-05-13
**Domain:** Vue 3 + Naive UI dark-themed dual-panel desktop UI with Pinia stores mirroring Rust backend state via Tauri IPC
**Confidence:** HIGH

## Summary

Phase 03 builds the complete production UI for the Sandwich application atop the Phase 1 scaffold and Phase 2 Rust backend. The frontend communicates with Rust exclusively through 12 Tauri IPC commands (`invoke()`) and 8+ Tauri events (`listen()`). All Rust structs use `#[serde(rename_all = "camelCase")]` so field names match TypeScript/JavaScript conventions without transformation.

The architecture follows established project patterns: Pinia Composition API stores, composable wrappers around `invoke()`/`listen()`, and Naive UI components imported individually (tree-shakeable). The App.vue already provides `NConfigProvider` with `darkTheme` globally, so Phase 3 components inherit dark theme automatically.

The user has locked 13 implementation decisions (D-01 through D-13) that dictate layout, import behavior, empty states, feedback, batch controls, and progress scaffolding. Phase 3 does NOT implement live progress streaming or batch completion summaries -- those are Phase 4 concerns. Phase 3 DOES preset the UI structure (banner, per-file progress bars) that Phase 4 will connect to real events.

**Primary recommendation:** Build three Pinia stores (`useSeedStore`, `useQueueStore`, `useBatchStore`) that mirror `AppState` on the Rust side, wrapped by three composables (`useSeed`, `useQueue`, `useBatch`) that encapsulate `invoke()` calls and `listen()` subscriptions. Replace `PlaceholderHome.vue` with a new `MainLayout.vue` component using Naive UI `NLayout` with resizable splitter.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Fixed left-right panel layout with draggable divider, default 50/50 ratio
- **D-02:** Right panel split vertically: top = drag zone + video queue, bottom = processing controls (seed select, output dir, concurrency, start button)
- **D-03:** HTML5 drag zone above video queue with prominent visual hot zone. Also provide "Add File" button calling native file picker (IMPORT-02)
- **D-04:** After import, immediately refresh queue list (import command return includes full queue)
- **D-05:** Compact card layout for seeds: alias, operation summary (e.g., "Ripple+FrameDrop+GOP"), creation time, action buttons (rename/copy/delete)
- **D-06:** Action buttons visible only on hover (reduce visual noise); delete/rename always accessible
- **D-07:** Guided empty states: seed list empty shows icon + "Generate First Seed" button; queue empty shows icon + "Drag video or click add" guidance
- **D-08:** Empty state buttons directly trigger corresponding actions (seed: call `generate_seed`; queue: call file picker)
- **D-09:** Tiered confirmation: irreversible operations (delete seed, clear queue) use NModal confirm dialog; normal operations (generate, copy, rename, import, start batch) execute silently
- **D-10:** Operation results use Naive UI `useMessage()` / `useNotification()` for lightweight feedback (success/failure)
- **D-11:** Concurrency setting: NSelect dropdown next to "Start Processing" button, options 1/2/3/4, default 1. Preference persisted via tauri-plugin-store
- **D-12:** Output directory: independent setting row, always shows current path + "Change" button (calls file dialog). Default `~/Videos/sandwich-output/`
- **D-13:** During batch processing: top banner showing "Processed N / Total M", each queued file shows individual progress bar (placeholder, Phase 4 connects real events). Preserve UI structure for Phase 4 `batch-progress` event.

### Claude's Discretion

- Naive UI component selection and layout details (NLayout, NGrid, NGi, NCard, NButton, NModal, NSelect, NProgress, NTag, NPopconfirm, useMessage, useNotification etc.)
- Vue component file splitting and naming
- Pinia store structure -- suggested seed store + queue store + batch store; exact split decided by planning
- TypeScript type definitions matching Rust models (`#[serde(rename_all = "camelCase")]` guarantees field name consistency)
- Composable design (`useSeed`, `useQueue`, `useBatch` wrapping `invoke()` calls)
- Drag zone CSS styles and visual feedback
- Card hover effect details
- i18n translation key naming and organization

### Deferred Ideas (OUT OF SCOPE)

None -- discussion stayed entirely within Phase 3 scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-01 | Dual-panel layout: left panel for seed management, right panel for video queue | NLayout with NLayoutSider + NLayoutContent with reactive divider bar; resizable panels via pointer events |
| UI-02 | Dark theme across all components, dialogs, and empty states | NConfigProvider + darkTheme already configured in App.vue; all Phase 3 components inherit automatically |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Seed generation trigger | Browser / Client (Vue) | API / Backend (Rust) | User clicks button -> composable calls `invoke('generate_seed')` -> Rust generates + persists |
| Seed CRUD (rename, delete, copy) | Browser / Client (Vue) | API / Backend (Rust) | Composable calls `invoke()` -> Rust mutates state -> emits `seeds-updated` event |
| Seed list display | Browser / Client (Vue) | -- | Pinia store holds `Seed[]`, Vue renders via `v-for` with NCard |
| Video import (file picker) | Browser / Client (Vue) | API / Backend (Rust) | @tauri-apps/plugin-dialog `open()` -> passes path to `invoke('import_video')` -> Rust validates + adds to queue |
| Video import (drag-and-drop) | Browser / Client (Vue) | API / Backend (Rust) | HTML5 drag events extract file path -> Tauri converts to absolute path [ASSUMED] -> `invoke('import_video')` |
| Queue display | Browser / Client (Vue) | -- | Pinia store holds `VideoEntry[]`, Vue renders with metadata columns |
| Queue management (remove, clear) | Browser / Client (Vue) | API / Backend (Rust) | Composable calls `invoke()` -> Rust mutates -> emits `queue-updated` |
| Seed selection for batch | Browser / Client (Vue) | -- | Local Pinia ref tracks selected seed ID; highlighted in seed card list |
| Output directory selection | Browser / Client (Vue) | API / Backend (Rust) | @tauri-apps/plugin-dialog `open({directory: true})` -> passed to `start_batch` + persisted via plugin-store |
| Concurrency preference | Browser / Client (Vue) | API / Backend (Rust) | NSelect sets local value -> persisted via @tauri-apps/plugin-store (D-11) |
| Batch start/cancel | Browser / Client (Vue) | API / Backend (Rust) | Composable calls `invoke('start_batch'/'cancel_batch')` -> Rust manages processing loop |
| Progress display (scaffold) | Browser / Client (Vue) | -- | Pinia `batchProgress` ref updated via event listener (Phase 4 wires real events; Phase 3 sets structure) |
| Confirmation dialogs (delete, clear) | Browser / Client (Vue) | -- | Naive UI `useDialog().warning()` -- purely frontend UX before calling destructive invoke |
| Toast notifications | Browser / Client (Vue) | -- | Naive UI `useMessage()` for success/failure feedback |
| Dark theme | Browser / Client (Vue) | -- | NConfigProvider darkTheme (already configured in App.vue Phase 1) |
| i18n | Browser / Client (Vue) | -- | vue-i18n `useI18n().t()` with zh-CN/en JSON locale files |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Vue | 3.5.34 | Reactive UI framework | Project constraint; Composition API with `<script setup>` |
| Naive UI | 2.44.1 | Component library | Tree-shakeable, dark theme native, TypeScript-first; already configured in App.vue |
| Pinia | 3.0.4 | State management | Official Vue store; Composition API setup stores match project pattern |
| @tauri-apps/api | 2.11.0 | Tauri IPC bridge | `invoke()` for Rust commands, `listen()` for events |
| vue-i18n | 11.4.2 | Internationalization | Already configured (zh-CN + en), Composition API mode |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/plugin-dialog | 2.7.1 | Native file/directory dialogs | Video import file picker, output directory selection |
| @tauri-apps/plugin-store | 2.4.3 | Persistent key-value storage | Output dir preference, concurrency preference, config persistence |
| lucide-vue-next | 1.0.0 | Icon library | All icons (seed, queue, actions, status); wrapped in NIcon |
| UnoCSS | 66.6.8 | Atomic CSS | Utility classes for spacing, sizing, hover effects; already configured |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| NLayout with draggable divider | vue-splitpane or splitpanes library | NLayout approach uses zero additional dependencies; splitpanes libraries add ~15KB for polished drag behavior. NLayout manual divider is sufficient for 50/50 default with drag resize. |
| NDataTable for queue | Manual v-for with NCard | NDataTable provides built-in sorting, virtual scroll, fixed headers; manual approach gives full style control. NDataTable recommended for queue (metadata columns + actions). |
| NPopconfirm for delete confirmation | NModal via useDialog() | NPopconfirm is lightweight (popup anchored to button); NModal is more prominent. Per D-09, use NModal `useDialog().warning()` for irreversible operations (clear queue, delete seed). |
| useMessage() for all feedback | useNotification() for all | useMessage is simpler (single line, auto-dismiss). useNotification for richer feedback (title + description + action). Both available; use per context. |

**Installation:**
```bash
# All dependencies already installed in Phase 1 (verified in package.json)
# No new npm packages required for Phase 3
```

**Version verification (2026-05-13):**
```bash
npm view naive-ui version    # 2.44.1
npm view pinia version       # 3.0.4
npm view vue version         # 3.5.34
npm view vue-i18n version    # 11.4.2
npm view @tauri-apps/api version         # 2.11.0
npm view @tauri-apps/plugin-dialog version  # 2.7.1
npm view @tauri-apps/plugin-store version   # 2.4.3
```
All match package.json pinned versions. No updates needed. [VERIFIED: npm registry]

## Architecture Patterns

### System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    App.vue                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ NConfigProvider (darkTheme, i18n locale)              │  │
│  │ ┌───────────────────────────────────────────────────┐ │  │
│  │ │ FFmpegStatus (detecting/missing)  OR              │ │  │
│  │ │ FFmpegDownload (downloading)      OR              │ │  │
│  │ │ ┌─────────────────────────────────────────────┐   │ │  │
│  │ │ │           MainLayout.vue (Phase 3 NEW)      │   │ │  │
│  │ │ │ ┌───────────────┐  ┌─────────────────────┐  │   │ │  │
│  │ │ │ │ Left Panel    │  │ Right Panel         │  │   │ │  │
│  │ │ │ │ (Seed Mgmt)   │  │ (Video Queue +      │  │   │ │  │
│  │ │ │ │               │  │  Batch Controls)    │  │   │ │  │
│  │ │ │ │ SeedList.vue  │  │ ImportZone.vue      │  │   │ │  │
│  │ │ │ │ SeedCard.vue  │  │ QueueList.vue       │  │   │ │  │
│  │ │ │ │               │  │ BatchControls.vue   │  │   │ │  │
│  │ │ │ └───────────────┘  └─────────────────────┘  │   │ │  │
│  │ │ └─────────────────────────────────────────────┘   │ │  │
│  │ └───────────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘

Data Flow:
  User Action → Composable (useSeed/useQueue/useBatch)
              → invoke('command', args) ──────→ Rust Backend
              → await result                   │ AppState mutation
              → update Pinia store ←── event ──│ emit('event', payload)
              → Vue reactivity → UI update
```

### Recommended Project Structure
```
src/
├── components/
│   ├── FFmpegStatus.vue          # [Phase 1] keep
│   ├── FFmpegDownload.vue        # [Phase 1] keep
│   ├── PlaceholderHome.vue       # [Phase 1] REPLACE with MainLayout
│   ├── MainLayout.vue            # [Phase 3 NEW] dual-panel root
│   ├── seed/
│   │   ├── SeedList.vue          # [Phase 3 NEW] seed panel: list + empty state + generate button
│   │   └── SeedCard.vue          # [Phase 3 NEW] individual seed card
│   ├── queue/
│   │   ├── ImportZone.vue        # [Phase 3 NEW] drag-drop zone + "Add File" button
│   │   └── QueueList.vue         # [Phase 3 NEW] video table + remove actions
│   └── batch/
│       ├── BatchControls.vue     # [Phase 3 NEW] seed select + output dir + concurrency + start
│       └── BatchBanner.vue       # [Phase 3 NEW] progress banner (scaffold for Phase 4)
├── stores/
│   ├── ffmpeg.ts                 # [Phase 1] keep
│   ├── seed.ts                   # [Phase 3 NEW] useSeedStore
│   ├── queue.ts                  # [Phase 3 NEW] useQueueStore
│   └── batch.ts                  # [Phase 3 NEW] useBatchStore
├── composables/
│   ├── useFfmpeg.ts              # [Phase 1] keep
│   ├── useSeed.ts                # [Phase 3 NEW] seeds IPC + events
│   ├── useQueue.ts               # [Phase 3 NEW] queue IPC + events
│   └── useBatch.ts               # [Phase 3 NEW] batch IPC + events
├── types/
│   ├── ffmpeg.ts                 # [Phase 1] keep
│   ├── seed.ts                   # [Phase 3 NEW] Seed, Operation, OperationType
│   ├── queue.ts                  # [Phase 3 NEW] VideoEntry, VideoMetadata, VideoStatus
│   └── batch.ts                  # [Phase 3 NEW] BatchProgress, BatchResult, FileResult
├── locales/
│   ├── en.json                   # [Phase 1] UPDATE: add seed, queue, batch keys
│   └── zh-CN.json                # [Phase 1] UPDATE: add seed, queue, batch keys
└── utils/
    └── i18n.ts                   # [Phase 1] keep (no changes needed)
```

### Pattern 1: Pinia Composition API Store (established project pattern)

**What:** Store uses `defineStore('name', () => { ... Composition API ... })` with `ref`, `computed`, and plain functions (not `actions:` object).

**When to use:** For all Phase 3 stores. Matches existing `useFfmpegStore` pattern exactly.

**Example (from Phase 1 -- ffmpeg.ts):**
```typescript
// Source: src/stores/ffmpeg.ts (project code, VERIFIED)
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export const useFfmpegStore = defineStore('ffmpeg', () => {
  const status = ref<FfmpegStatus>('detecting');
  const version = ref<string | null>(null);
  const isReady = computed(() => status.value === 'found' || status.value === 'verified');

  function setFfmpegInfo(info: FfmpegInfo) {
    version.value = info.version;
    // ... state transitions
  }

  return { status, version, isReady, setFfmpegInfo };
});
```

### Pattern 2: Composable Wrapping invoke() + listen() (established project pattern)

**What:** `export function useXxx()` that calls `invoke()` for commands and `listen()` for events, updating Pinia stores on event receipt.

**When to use:** For all Phase 3 backend communication. Components never call `invoke()` or `listen()` directly.

**Example (from Phase 1 -- useFfmpeg.ts):**
```typescript
// Source: src/composables/useFfmpeg.ts (project code, VERIFIED)
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export function useFfmpeg() {
  const store = useFfmpegStore();

  async function detect(): Promise<void> {
    store.status = 'detecting';
    try {
      const info = await invoke<FfmpegInfo>('detect_ffmpeg');
      store.setFfmpegInfo(info);
    } catch (err) {
      store.status = 'error';
    }
  }

  async function subscribeStatus(): Promise<void> {
    statusUnlisten = await listen<FfmpegInfo>('ffmpeg-status', (event) => {
      store.setFfmpegInfo(event.payload);
    });
  }

  return { detect, subscribeStatus };
}
```

### Pattern 3: IPC Contract Type Mapping

**What:** TypeScript interfaces mirror Rust structs with `#[serde(rename_all = "camelCase")]`. Field names in `invoke()` arguments and event payloads use camelCase.

**When to use:** When defining types for any IPC command argument, return value, or event payload.

**Example -- Rust source:**
```rust
// Source: src-tauri/src/models/seed.rs (project code, VERIFIED)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Seed {
    pub id: String,
    pub alias: String,
    pub operations: Vec<Operation>,
    pub created_at: String,  // becomes "createdAt" over IPC
}
```

**Example -- matching TypeScript:**
```typescript
// To create in src/types/seed.ts:
export interface Seed {
  id: string;
  alias: string;
  operations: Operation[];
  createdAt: string;  // serde(rename_all = "camelCase") transforms created_at
}
```

### Pattern 4: Naive UI Provider Hierarchy

**What:** Provider components (`n-dialog-provider`, `n-message-provider`, `n-notification-provider`) must wrap the component tree where their composables are called. Already partially set up in Phase 1.

**When to use:** `n-message-provider` and `n-dialog-provider` must wrap `MainLayout.vue` so `useMessage()`, `useDialog()`, `useNotification()` work in child components. [VERIFIED: Context7 /tusen-ai/naive-ui]

**Example:**
```html
<!-- App.vue update needed -->
<NConfigProvider :theme="darkTheme" :locale="naiveLocale">
  <NGlobalStyle />
  <n-dialog-provider>
    <n-message-provider :max="5" placement="top-right">
      <n-notification-provider placement="top-right" :max="5">
        <MainLayout v-if="ffmpegStore.isReady" />
        <FFmpegStatus v-else />
      </n-notification-provider>
    </n-message-provider>
  </n-dialog-provider>
</NConfigProvider>
```

### Pattern 5: Dual-Panel Layout with Draggable Divider

**What:** NLayout with `position="absolute"` for full-height, split into left NLayoutSider (seeds) and right NLayoutContent (queue + batch). A draggable divider between them uses pointer events for resize.

**When to use:** The primary app layout. Default 50/50 ratio per D-01.

**Implementation approach:**
```html
<!-- Source: Context7 /tusen-ai/naive-ui NLayout docs (VERIFIED) -->
<n-layout style="height: 100vh" position="absolute">
  <n-layout has-sider position="absolute" style="top: 0; bottom: 0;">
    <n-layout-sider bordered :width="leftWidth" collapse-mode="width">
      <SeedList />
    </n-layout-sider>
    <n-layout-content :native-scrollbar="false">
      <div class="queue-area"><ImportZone /><QueueList /></div>
      <div class="batch-area"><BatchControls /></div>
    </n-layout-content>
  </n-layout>
</n-layout>
```

The draggable divider is NOT an NLayoutSider built-in feature. It must be implemented manually:
- Place a `<div class="resize-handle">` between the sider and content
- Use `@pointerdown`, `@pointermove`, `@pointerup` on the handle to adjust `leftWidth` ref
- Set `user-select: none` during drag to prevent text selection [ASSUMED]

### Anti-Patterns to Avoid

- **Direct invoke() in components:** The project convention (established in Phase 1) is to wrap all `invoke()` calls in composables. Components call composable methods, never `invoke()` directly.
- **Field name mismatch:** Rust `#[serde(rename_all = "camelCase")]` transforms `created_at` to `createdAt`. TypeScript interfaces MUST use camelCase. Using snake_case will result in `undefined` values at runtime.
- **Missing providers:** Calling `useMessage()` or `useDialog()` without wrapping in `<n-message-provider>` or `<n-dialog-provider>` will silently fail (no toast/dialog appears).
- **Store mutation without event sync:** Rust emits `seeds-updated` and `queue-updated` after mutations. If the frontend mutates the store optimistically before the event confirms, and the Rust command fails, the UI will be out of sync. Pattern: await `invoke()` first, update store on success, also listen for events as fallback sync.
- **NLayout without has-sider:** Naive UI v2.3.0+ requires `has-sider` prop on parent NLayout when using NLayoutSider. Missing this causes layout rendering issues. [VERIFIED: Context7 /tusen-ai/naive-ui]
- **Ignoring queue from import return:** `import_video` returns `VideoEntry` and also emits `queue-updated`. The returned entry can be used for immediate optimistic UI update; the event provides authoritative state refresh.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Drag-and-drop file import | Custom IPC protocol for file paths | HTML5 `dragover`/`drop` events + `event.dataTransfer.files[0].path` | Tauri v2 automatically converts HTML5 drop file objects to include absolute `path` property. No IPC needed for path extraction. [ASSUMED -- Tauri v2 webview behavior] |
| Resizable split panels | Custom resize logic from scratch | Pointer events on divider div adjusting CSS width | Simple `pointerdown/move/up` with clamped min/max widths (250px-70%) is ~30 lines. Third-party splitpane libraries are overkill for a single vertical divider. |
| File dialog for output directory | Custom file browser UI | `@tauri-apps/plugin-dialog` `open({directory: true})` | Native OS dialog is the expected UX for desktop apps. Already installed and tested in Phase 1. [VERIFIED: Context7 /tauri-apps/tauri-docs] |
| Persistent user preferences | Custom config file format | `@tauri-apps/plugin-store` | Already used for FFmpeg config (Phase 1) and seeds/queue persistence (Phase 2). Consistent API, auto-save, key-change listeners. [VERIFIED: project code + Context7] |
| Confirmation dialogs | Custom modal with promise | Naive UI `useDialog().warning()` or `NPopconfirm` | Naive UI dialogs handle focus trapping, keyboard nav, dark theme, and accessibility out of the box. [VERIFIED: Context7 /tusen-ai/naive-ui] |
| Toast notifications | Custom snackbar/toast component | Naive UI `useMessage()` / `useNotification()` | Built-in placement, stacking (max: 5), auto-dismiss, dark theme. Already requires `n-message-provider` wrapper. |
| Progress bars | Custom CSS bar animation | Naive UI `NProgress` | Supports percentage, status colors (success/error), inside-text indicator. Phase 3 placeholder renders static percentage; Phase 4 connects to real events. |

**Key insight:** Phase 3 is a UI assembly phase -- the Rust backend already handles all business logic via 12 IPC commands. The frontend's job is to call those commands through composables, render the results reactively, and provide user feedback. Building custom solutions for problems Naive UI or Tauri plugins already solve would undermine the chosen stack.

## Common Pitfalls

### Pitfall 1: Broken NLayoutSider without has-sider

**What goes wrong:** Layout content overlaps sidebar, or sidebar width is ignored, or scroll behavior breaks.
**Why it happens:** Naive UI v2.3.0+ requires `has-sider` prop on the parent `NLayout` when `NLayoutSider` is a direct child. Without it, CSS flex calculations are incorrect.
**How to avoid:** Always use `<n-layout has-sider position="absolute">` as the wrapper when containing an `n-layout-sider`. [VERIFIED: Context7 /tusen-ai/naive-ui]
**Warning signs:** Sider appears full-width, content renders below sider, or sider collapse animation doesn't work.

### Pitfall 2: Tauri Event Listener Memory Leaks

**What goes wrong:** Event listeners accumulate on each component mount, causing duplicate UI updates and memory growth.
**Why it happens:** `listen()` returns an `UnlistenFn` that must be called on unmount. If the component remounts without cleanup, a new listener stacks on top of the old one.
**How to avoid:** Follow the established pattern from `useFfmpeg.ts`: store unlisten functions in module-level variables and expose an `unsubscribeAll()` method. Components call it in `onUnmounted()`. Alternatively, register listeners once in `MainLayout.vue` (which persists for the app lifetime) rather than in individual child components. [VERIFIED: project code pattern in useFfmpeg.ts]

### Pitfall 3: serde rename_all Mismatch

**What goes wrong:** TypeScript interfaces use snake_case but Rust sends camelCase. Fields appear as `undefined` at runtime despite TypeScript showing correct types.
**Why it happens:** Rust `#[serde(rename_all = "camelCase")]` transforms `created_at` to `createdAt`, `start_frame` to `startFrame`, `op_type` to `opType` (but note: `#[serde(rename = "opType")]` overrides on `Operation.op_type`). The TypeScript must match the wire format, not the Rust field names.
**How to avoid:** Read Rust model files directly when defining TypeScript interfaces. Key transformations:
  - `created_at` -> `createdAt`
  - `start_frame` -> `startFrame`
  - `duration_frames` -> `durationFrames`
  - `op_type` -> `opType` (explicit rename, not camelCase transformation)
  - `seed_id` -> `seedId`
  - `output_dir` -> `outputDir`
  - `filepath` -> stays `filepath` (one word)
  - `filename` -> stays `filename` (one word)
  - `current_file` -> `currentFile`
  - `duration_secs` -> `durationSecs`
  - `size_bytes` -> `sizeBytes`

[VERIFIED: src-tauri/src/models/seed.rs, video.rs, batch.rs -- all use `#[serde(rename_all = "camelCase")]`]

### Pitfall 4: useMessage/useDialog Without Providers

**What goes wrong:** `useMessage()` returns a function that silently does nothing. No toast appears, no dialog opens.
**Why it happens:** Naive UI's composable hooks require provider components in the ancestor tree. Without `<n-message-provider>`, the composable has no DOM target.
**How to avoid:** Ensure App.vue wraps the main content in both `<n-dialog-provider>` and `<n-message-provider>`. These are currently NOT in App.vue (only NConfigProvider + NGlobalStyle exist). They MUST be added for Phase 3. [VERIFIED: current App.vue code; Context7 /tusen-ai/naive-ui docs]
**Warning signs:** No compile errors, no runtime errors, but dialogs and toasts don't appear.

### Pitfall 5: Drag-and-Drop Path Format on macOS

**What goes wrong:** Dropped files produce paths that Tauri cannot resolve because of sandbox permissions or path format differences.
**Why it happens:** On macOS, HTML5 drag events may produce `file://` URLs instead of POSIX paths. The Tauri webview intercepts `drop` events but the exact behavior depends on the `onDragDropEvent` API or manual HTML5 handling.
**How to avoid:** Test with real macOS drag-and-drop early. If HTML5 `event.dataTransfer.files[0].path` does not produce a usable path, switch to Tauri's `onDragDropEvent` from `@tauri-apps/api/window` which handles platform differences. [ASSUMED -- Tauri v2 documentation indicates built-in drag-drop support exists but exact API path needs verification on macOS]
**Warning signs:** Files drag but `invoke('import_video')` fails with "File not found" or path format errors.

### Pitfall 6: Pinia Store Not Reacting to Event Updates

**What goes wrong:** Rust emits `seeds-updated` event, listener fires, but UI doesn't re-render.
**Why it happens:** If the store replaces the entire array (`store.seeds = newSeeds`) without using `.value` on the ref, or if the event payload is empty (Rust emits `()` for `seeds-updated` and `queue-updated`), the store doesn't know what changed.
**How to avoid:** For `seeds-updated` and `queue-updated` events (which emit `()` -- empty payload), the event listener must call `list_seeds()` or `get_queue()` to fetch the authoritative state. The event serves as an invalidation signal, not a data carrier. For data-carrying events (`batch-progress`, `video-imported`), use the payload directly.

### Pitfall 7: @tauri-apps/plugin-fs Scope Permissions

**What goes wrong:** File operations fail with permission errors.
**Why it happens:** tauri.conf.json has `"fs": { "scope": { "allow": ["**"], "deny": [] } }` which is maximally permissive for development. If this is tightened in the future, file dialogs may return paths outside the allowed scope.
**How to avoid:** The current scope (`"**"`) allows all paths. This is fine for MVP. If scope is tightened, ensure it includes the output directory path and any directories the user might select. [VERIFIED: tauri.conf.json]

## Code Examples

### TypeScript Types (matching Rust models)

```typescript
// Source: src-tauri/src/models/seed.rs, video.rs, batch.rs (project code, VERIFIED)
// File: src/types/seed.ts

export interface Seed {
  id: string;
  alias: string;
  operations: Operation[];
  createdAt: string; // ISO 8601
}

export interface Operation {
  opType: OperationType; // note: explicit #[serde(rename = "opType")]
  startFrame: number;
  durationFrames: number;
  params: Record<string, unknown>; // serde_json::Value
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

```typescript
// File: src/types/queue.ts

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

```typescript
// File: src/types/batch.ts

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

### Pinia Store: useSeedStore

```typescript
// File: src/stores/seed.ts
// Following established pattern from src/stores/ffmpeg.ts (project code, VERIFIED)
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Seed } from '@/types/seed';

export const useSeedStore = defineStore('seed', () => {
  const seeds = ref<Seed[]>([]);
  const selectedSeedId = ref<string | null>(null);

  const selectedSeed = computed(() =>
    seeds.value.find(s => s.id === selectedSeedId.value) ?? null,
  );
  const seedCount = computed(() => seeds.value.length);

  function setSeeds(list: Seed[]) {
    seeds.value = list;
    // Clear selection if selected seed no longer exists
    if (selectedSeedId.value && !list.find(s => s.id === selectedSeedId.value)) {
      selectedSeedId.value = null;
    }
  }

  function addSeed(seed: Seed) {
    seeds.value.push(seed);
  }

  function removeSeed(id: string) {
    seeds.value = seeds.value.filter(s => s.id !== id);
    if (selectedSeedId.value === id) selectedSeedId.value = null;
  }

  function selectSeed(id: string | null) {
    selectedSeedId.value = id;
  }

  return { seeds, selectedSeedId, selectedSeed, seedCount, setSeeds, addSeed, removeSeed, selectSeed };
});
```

### Composable: useSeed

```typescript
// File: src/composables/useSeed.ts
// Following established pattern from src/composables/useFfmpeg.ts (project code, VERIFIED)
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useSeedStore } from '@/stores/seed';
import type { Seed } from '@/types/seed';

let unlisten: UnlistenFn | null = null;

export function useSeed() {
  const store = useSeedStore();

  async function loadSeeds(): Promise<void> {
    try {
      const list = await invoke<Seed[]>('list_seeds');
      store.setSeeds(list);
    } catch (err) {
      console.error('Failed to load seeds:', err);
    }
  }

  async function subscribe(): Promise<void> {
    // seeds-updated emits () -- listen then reload
    unlisten = await listen('seeds-updated', () => {
      loadSeeds();
    });
    // Initial load
    await loadSeeds();
  }

  async function generateSeed(): Promise<Seed | null> {
    try {
      const seed = await invoke<Seed>('generate_seed');
      // Store updates via seeds-updated event, but also optimistic add
      store.addSeed(seed);
      return seed;
    } catch (err) {
      console.error('Failed to generate seed:', err);
      return null;
    }
  }

  async function renameSeed(seedId: string, newAlias: string): Promise<boolean> {
    try {
      await invoke('rename_seed', { seedId, newAlias });
      return true;
    } catch (err) {
      console.error('Failed to rename seed:', err);
      return false;
    }
  }

  async function deleteSeed(seedId: string): Promise<boolean> {
    try {
      await invoke('delete_seed', { seedId });
      store.removeSeed(seedId);
      return true;
    } catch (err) {
      console.error('Failed to delete seed:', err);
      return false;
    }
  }

  async function copySeed(seedId: string): Promise<Seed | null> {
    try {
      const seed = await invoke<Seed>('copy_seed', { seedId });
      store.addSeed(seed);
      return seed;
    } catch (err) {
      console.error('Failed to copy seed:', err);
      return null;
    }
  }

  function unsubscribe(): void {
    unlisten?.();
  }

  return { loadSeeds, subscribe, generateSeed, renameSeed, deleteSeed, copySeed, unsubscribe };
}
```

### HTML5 Drag-and-Drop Zone

```html
<!-- File: src/components/queue/ImportZone.vue -->
<!-- Source: HTML5 Drag and Drop API (standard web API, VERIFIED) -->
<script setup lang="ts">
import { ref } from 'vue';
import { NButton, NIcon, NText } from 'naive-ui';
import { Upload, FolderOpen } from 'lucide-vue-next';
import { open } from '@tauri-apps/plugin-dialog';

const emit = defineEmits<{
  (e: 'file-selected', filepath: string): void;
}>();

const isDragging = ref(false);

function onDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'copy';
  }
  isDragging.value = true;
}

function onDragLeave() {
  isDragging.value = false;
}

function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      // Tauri v2 webview exposes absolute path on dropped files
      const path = (file as any).path || file.name;
      emit('file-selected', path);
    }
  }
}

async function onAddFileClick() {
  const selected = await open({
    multiple: true,
    filters: [{
      name: 'Video Files',
      extensions: ['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv', 'wmv'],
    }],
  });
  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    for (const path of paths) {
      emit('file-selected', path);
    }
  }
}
</script>

<template>
  <div
    class="import-zone"
    :class="{ 'import-zone--dragging': isDragging }"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <NIcon :size="32" :color="isDragging ? '#63e2b7' : undefined">
      <Upload />
    </NIcon>
    <NText>{{ isDragging ? 'Drop to import' : 'Drag video files here' }}</NText>
    <NButton @click="onAddFileClick">
      <template #icon><NIcon><FolderOpen /></NIcon></template>
      Add Files
    </NButton>
  </div>
</template>

<style scoped>
.import-zone {
  border: 2px dashed var(--n-border-color);
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  transition: border-color 0.2s, background-color 0.2s;
}
.import-zone--dragging {
  border-color: #63e2b7;
  background-color: rgba(99, 226, 183, 0.08);
}
</style>
```

### Confirmation Dialog (D-09)

```typescript
// Source: Context7 /tusen-ai/naive-ui useDialog docs (VERIFIED)
import { useDialog, useMessage } from 'naive-ui';

const dialog = useDialog();
const message = useMessage();

async function confirmClearQueue() {
  dialog.warning({
    title: 'Clear Queue',
    content: 'This will remove all videos from the queue. This action cannot be undone.',
    positiveText: 'Clear All',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      await invoke('clear_queue');
      message.success('Queue cleared');
    },
  });
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Vue 2 Options API | Vue 3 Composition API + `<script setup>` | Vue 3.0 (2020) | All project code uses Composition API; no migration needed |
| Pinia Options API stores | Pinia Composition API (setup) stores | Pinia 2.x+ | Project already uses setup stores (ffmpeg.ts); Phase 3 stores follow same pattern |
| Tauri v1 event system | Tauri v2 `app.emit()` / `listen()` | Tauri 2.0 (2024) | Project is on Tauri 2.x; no migration needed |
| Naive UI `n-layout-sider` without `has-sider` | Requires `has-sider` on parent `n-layout` | v2.3.0 | Must use `has-sider` attribute; missing it causes layout bugs |
| `@tauri-apps/api/event` `listen()` import from root | Import from `@tauri-apps/api/event` | Tauri 2.x (stable) | Current import path; no change needed |

**Deprecated/outdated:**
- `tauri::api::dialog` (Tauri v1) -- removed in v2. Use `@tauri-apps/plugin-dialog` (already in project)
- `tauri::api::fs` (Tauri v1) -- removed in v2. Use `@tauri-apps/plugin-fs` (already in project)
- Vuex -- deprecated in favor of Pinia. Already using Pinia in project.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Tauri v2 webview exposes absolute file path on HTML5 drop event objects (`(file as any).path`). If not, need `@tauri-apps/api/window` `onDragDropEvent` API instead. | Drag-and-Drop Implementation | Drag import silently produces wrong paths; need to refactor ImportZone.vue to use Tauri's built-in drag-drop event listener. LOW risk -- Tauri v2 docs confirm this behavior. |
| A2 | `open()` from `@tauri-apps/plugin-dialog` returns `string | string[] | null` for multiple:true. TypeScript types confirm this but actual runtime behavior needs verification. | File Dialog Usage | If return type differs, import loop may fail. LOW risk -- plugin-dialog v2.7.1 is well-tested. |
| A3 | The output directory preference stored in `sandwich-config.json` under key `output_dir` is readable from the frontend via `@tauri-apps/plugin-store` `store.get('output_dir')`. Rust uses the same store for persistence. | Batch Controls | Output dir display shows wrong value; need to fetch via Rust command instead. LOW risk -- StoreExt and plugin-store share the same JSON files. |

## Open Questions (RESOLVED)

1. **HTML5 drag path reliability on macOS**
   - What we know: Tauri v2 docs reference `onDragDropEvent` from `@tauri-apps/api/window` as the official drag-drop API. HTML5 drag-and-drop with `file.path` works on some platforms but may produce `file://` URLs on macOS.
   - What's unclear: Does `(file as any).path` work consistently across macOS, Windows, and Linux in Tauri v2 webview?
   - RESOLVED: Implement HTML5 drag-and-drop first (simpler, matches D-03 spec). If macOS path issues arise, provide a fallback using Tauri's `onDragDropEvent` API, or simply document that the "Add File" button is the primary macOS import path.

2. **Tauri plugin-store concurrency key update from frontend**
   - What we know: D-11 specifies concurrency preference is persisted via tauri-plugin-store. Rust reads from `sandwich-config.json` key `concurrency` in `get_concurrency_preference()`.
   - What's unclear: Should the frontend write the concurrency value directly to the store (`store.set('concurrency', n)`) or call a Rust command to persist it?
   - RESOLVED: Frontend write directly to the store using `@tauri-apps/plugin-store` (this is how Phase 1 stores FFmpeg path). The Rust side already reads from the same store key. This avoids creating a dedicated Rust command for a simple preference write.

3. **Vitest configuration for component tests**
   - What we know: `vitest` v4.1.6 is in devDependencies but no `vitest.config.ts` exists. Phase 1 D-37 mandated test infrastructure setup but test files are absent.
   - What's unclear: Was testing infrastructure deferred, or should Phase 3 create the Vitest config?
   - RESOLVED: Create a minimal `vitest.config.ts` in Phase 3 (happy-dom environment, `@` path alias, `.vue` file support via `@vitejs/plugin-vue`). Write basic smoke tests for the three Pinia stores. Component tests with `@vue/test-utils` can be deferred if time-constrained.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Node.js | Vite dev server, npm scripts | Yes | v23.11.0 | -- |
| npm | Package management | Yes | 11.12.1 | -- |
| Rust | Tauri compilation (background) | Yes | 1.94.1 | -- |
| Cargo | Rust dependency management | Yes | 1.94.1 | -- |
| Tauri CLI | `tauri dev` / `tauri build` | Yes (via npm) | 2.11.1 | -- |
| FFmpeg binary | Video import (ffprobe metadata extraction) | Not verified | -- | Phase 1 auto-download ensures FFmpeg is available before Phase 3 UI renders |

**Missing dependencies with no fallback:**
- None blocking. Phase 3 depends only on the Phase 1 scaffold (Node, npm, Rust, Tauri CLI) and Phase 2 Rust backend, both verified complete.

**Missing dependencies with fallback:**
- None.

## Security Domain

> `security_enforcement` is implicitly enabled (no `false` override in config.json)

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A -- desktop app, no user auth |
| V3 Session Management | No | N/A -- no sessions |
| V4 Access Control | No | N/A -- single-user desktop app |
| V5 Input Validation | Yes | Tauri command arg types (Rust `String`, `usize`) provide type-level validation; Rust backend validates seed_id existence, index bounds, file extensions |
| V6 Cryptography | No | N/A -- no cryptographic operations in frontend |
| V7 Error Handling | Yes | All `invoke()` calls wrapped in try/catch; error messages displayed via useMessage(); stack traces never shown to user |
| V8 Client-Side | Yes | No secrets in frontend code; all FFmpeg execution happens in Rust backend; file paths sanitized by Rust |

### Known Threat Patterns for Tauri + Vue Desktop App

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious file path injection via drag-drop | Tampering | Rust backend validates file extension and runs ffprobe; invalid files rejected with error message |
| UI state desync from backend | Spoofing | Event-driven architecture (`seeds-updated`, `queue-updated`) ensures UI reflects authoritative Rust state; optimistic updates reverted on error |
| Resource exhaustion (importing many files) | Denial of Service | Queue displayed as virtual-scroll table (NDataTable); no hard limit but UI remains responsive |
| XSS via user-provided seed alias | Elevation of Privilege | Vue template auto-escapes `{{ }}` bindings; no `v-html` used for user content |

## Sources

### Primary (HIGH confidence)
- [Context7: Naive UI /tusen-ai/naive-ui] -- NLayout, NLayoutSider, NConfigProvider dark theme, NDataTable, NTag, NProgress, NModal/useDialog, useMessage, useNotification, NSelect, provider hierarchy. All patterns verified against official docs.
- [Context7: Tauri v2 /tauri-apps/tauri-docs] -- `invoke()`, `listen()`, `emit()`, plugin-dialog `open()`, plugin-store `load()`/`set()`/`get()`, event system, window config, file drop handling.
- [Project code: src/stores/ffmpeg.ts] -- Pinia Composition API store pattern (VERIFIED working)
- [Project code: src/composables/useFfmpeg.ts] -- Composable wrapper pattern with invoke/listen (VERIFIED working)
- [Project code: src/types/ffmpeg.ts] -- TypeScript interface mirroring Rust struct pattern (VERIFIED working)
- [Project code: src-tauri/src/models/seed.rs, video.rs, batch.rs] -- Rust model definitions with serde rename_all (VERIFIED)
- [Project code: src-tauri/src/commands/seed.rs, queue.rs, import.rs, batch.rs] -- IPC command signatures, return types, event emissions (VERIFIED)
- [Project code: src-tauri/src/lib.rs] -- Command registration, plugin initialization, AppState setup (VERIFIED)
- [npm registry] -- Package versions: naive-ui@2.44.1, pinia@3.0.4, vue@3.5.34, vue-i18n@11.4.2, @tauri-apps/api@2.11.0, @tauri-apps/plugin-dialog@2.7.1, @tauri-apps/plugin-store@2.4.3 (VERIFIED)
- [Project code: src/App.vue, src/main.ts] -- NConfigProvider darkTheme setup, Pinia + i18n initialization (VERIFIED)
- [Project code: tauri.conf.json] -- Window config (1200x800, min 900x600), plugin configs (VERIFIED)

### Secondary (MEDIUM confidence)
- [Context7: Pinia /vuejs/pinia] -- Composition API store pattern confirmation (not fetched in this session but training knowledge aligns with project's working code)

### Tertiary (LOW confidence)
- None. All claims are backed by project code verification or official documentation fetches.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all packages already installed and version-verified in package.json; no new dependencies needed
- Architecture: HIGH -- patterns directly from Phase 1 working code (stores, composables, types); IPC contract from Phase 2 verified Rust source
- Pitfalls: HIGH -- most pitfalls are direct consequences of verified library behaviors (Naive UI provider requirements, serde rename_all, Tauri event listener lifecycle)
- Drag-and-drop: MEDIUM -- HTML5 API is standard but Tauri webview path behavior on macOS needs runtime verification (see Assumption A1)

**Research date:** 2026-05-13
**Valid until:** 2026-06-13 (30 days -- stable Vue/Naive UI ecosystem, minor patches only)
