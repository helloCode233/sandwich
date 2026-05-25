<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { NLayout, NLayoutSider, NLayoutContent, NLayoutFooter, NTabs, NTabPane } from 'naive-ui';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useBatchStore } from '@/stores/batch';
import { useFfmpegStore } from '@/stores/ffmpeg';
import { useSeed } from '@/composables/useSeed';
import { useQueue } from '@/composables/useQueue';
import { useBatch } from '@/composables/useBatch';
import { useI18n } from 'vue-i18n';
import SeedList from '@/components/seed/SeedList.vue';
import ImportZone from '@/components/queue/ImportZone.vue';
import QueueList from '@/components/queue/QueueList.vue';
import BatchControls from '@/components/batch/BatchControls.vue';
import BatchBanner from '@/components/batch/BatchBanner.vue';
import BatchSummary from '@/components/batch/BatchSummary.vue';
import LogPanel from '@/components/log/LogPanel.vue';

const batchStore = useBatchStore();
const ffmpegStore = useFfmpegStore();
const seedComposable = useSeed();
const queueComposable = useQueue();
const batchComposable = useBatch();
const { t } = useI18n();

const rightPanelTab = ref<'queue' | 'log'>('queue');

// GPU encoder event listeners
let gpuDetectedUnlisten: UnlistenFn | null = null;
let gpuNotDetectedUnlisten: UnlistenFn | null = null;

// Panel sizing (D-01: default 50/50, draggable divider)
const leftWidth = ref(Math.floor(window.innerWidth / 2));
const isResizing = ref(false);

function onResizePointerdown(e: PointerEvent) {
  e.preventDefault();
  isResizing.value = true;
  document.body.style.userSelect = 'none';
  document.body.style.cursor = 'col-resize';
}

function onResizePointermove(e: PointerEvent) {
  if (!isResizing.value) return;
  const w = e.clientX;
  leftWidth.value = Math.max(250, Math.min(w, Math.floor(window.innerWidth * 0.7)));
}

function onResizePointerup() {
  isResizing.value = false;
  document.body.style.userSelect = '';
  document.body.style.cursor = '';
}

// Attach global move/up listeners when resizing starts
onMounted(() => {
  window.addEventListener('pointermove', onResizePointermove);
  window.addEventListener('pointerup', onResizePointerup);
});

onUnmounted(() => {
  window.removeEventListener('pointermove', onResizePointermove);
  window.removeEventListener('pointerup', onResizePointerup);
});

// Composable subscriptions (app-lifetime — registered once, never duplicated)
onMounted(async () => {
  await seedComposable.subscribe();
  await queueComposable.subscribe();
  await batchComposable.subscribe();
  // GPU encoder status events
  // Payload: unit variants serialize as string (e.g. "VideoToolbox"),
  // struct variants serialize as object (e.g. {"Nvenc": {...}})
  gpuDetectedUnlisten = await listen<string | Record<string, unknown>>(
    'gpu-encoder-detected',
    (event) => {
      const payload = event.payload;
      if (typeof payload === 'string') {
        ffmpegStore.setGpuEncoder(payload);
      } else if (payload && typeof payload === 'object') {
        // Externally-tagged enum: key is the variant name
        const key = Object.keys(payload)[0];
        if (key) ffmpegStore.setGpuEncoder(key);
      }
    },
  );
  gpuNotDetectedUnlisten = await listen('gpu-encoder-not-detected', () => {
    ffmpegStore.setGpuEncoder('');
  });
});

onUnmounted(() => {
  seedComposable.unsubscribe();
  queueComposable.unsubscribe();
  batchComposable.unsubscribe();
  gpuDetectedUnlisten?.();
  gpuNotDetectedUnlisten?.();
});
</script>

<template>
  <n-layout style="height: 100vh; height: 100dvh" position="absolute">
    <n-layout has-sider position="absolute" style="top: 0; bottom: 0">
      <!-- Left Panel: Seed Management -->
      <n-layout-sider bordered :width="leftWidth" collapse-mode="width">
        <SeedList />
      </n-layout-sider>

      <!-- Resize Handle (D-01: draggable divider) -->
      <div
        class="resize-handle"
        :class="{ 'resize-handle--active': isResizing }"
        @pointerdown="onResizePointerdown"
      />

      <!-- Right Panel: Queue + Batch Controls -->
      <n-layout-content :native-scrollbar="false">
        <div class="right-panel">
          <NTabs
            v-model:value="rightPanelTab"
            type="line"
            size="small"
            class="queue-area-tabs flex-1 flex flex-col min-h-0"
          >
            <!-- Queue Tab -->
            <NTabPane name="queue" :tab="t('queue.title')">
              <div class="queue-area">
                <ImportZone />
                <BatchBanner
                  v-if="batchStore.isProcessing || batchStore.cancelling || batchStore.isComplete"
                />
                <BatchSummary v-if="batchStore.isComplete && batchStore.lastResult" />
                <QueueList />
              </div>
            </NTabPane>

            <!-- History Tab (D-16, D-18) -->
            <NTabPane name="log" :tab="t('log.title')">
              <LogPanel />
            </NTabPane>
          </NTabs>

          <!-- Batch Controls (lower section per D-02, sticky footer) -->
          <n-layout-footer class="batch-footer">
            <BatchControls />
          </n-layout-footer>
        </div>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<style scoped>
.resize-handle {
  width: 4px;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color 0.2s;
  flex-shrink: 0;
  z-index: 10;
}
.resize-handle:hover,
.resize-handle--active {
  background-color: #2080f0;
}

.right-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.queue-area {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.batch-footer {
  flex-shrink: 0;
  border-top: 1px solid var(--n-border-color);
  background-color: var(--n-color);
}

.queue-area-tabs {
  min-height: 0;
}
.queue-area-tabs :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
.queue-area-tabs :deep(.n-tab-pane) {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
