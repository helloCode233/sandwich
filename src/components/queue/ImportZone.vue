<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { NButton, NIcon, NText } from 'naive-ui';
import { Upload, FolderOpen } from 'lucide-vue-next';
import { useMessage } from 'naive-ui';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { useQueue } from '@/composables/useQueue';
import { useBatchStore } from '@/stores/batch';
import { useI18n } from 'vue-i18n';

const { importVideo } = useQueue();
const message = useMessage();
const batchStore = useBatchStore();
const { t } = useI18n();

const isDragging = ref(false);

let unlistenDragDrop: UnlistenFn | null = null;

/** Supported video file extensions for file dialog filter. */
const VIDEO_EXTENSIONS = ['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv', 'wmv'];

onMounted(async () => {
  // Use Tauri's native drag-drop API which provides real file paths on all platforms
  unlistenDragDrop = await getCurrentWindow().onDragDropEvent((event) => {
    if (event.payload.type === 'enter' || event.payload.type === 'over') {
      isDragging.value = true;
    } else if (event.payload.type === 'drop') {
      isDragging.value = false;
      const paths = event.payload.paths;
      for (const path of paths) {
        importFile(path);
      }
    } else if (event.payload.type === 'leave') {
      isDragging.value = false;
    }
  });
});

onUnmounted(() => {
  unlistenDragDrop?.();
});

async function onAddFileClick() {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: 'Video Files',
        extensions: VIDEO_EXTENSIONS,
      },
    ],
  });
  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    for (const path of paths) {
      await importFile(path);
    }
  }
}

async function importFile(filepath: string) {
  const result = await importVideo(filepath);
  if ('entry' in result) {
    message.success(t('queue.imported', { filename: result.entry.filename }));
  } else {
    message.error(result.error);
  }
}
</script>

<template>
  <div
    v-if="!batchStore.isProcessing"
    class="import-zone"
    :class="{ 'import-zone--dragging': isDragging }"
  >
    <NIcon :size="48" :color="isDragging ? '#2080f0' : undefined">
      <Upload />
    </NIcon>
    <NText class="mt-2">
      {{ isDragging ? t('import.dropActive') : t('import.dropHere') }}
    </NText>
    <NText depth="3" class="text-xs mt-1">
      {{ t('import.supportedFormats') }}
    </NText>
    <NButton class="mt-3" @click="onAddFileClick">
      <template #icon>
        <NIcon :size="16">
          <FolderOpen />
        </NIcon>
      </template>
      {{ t('queue.addVideo') }}
    </NButton>
  </div>
</template>

<style scoped>
.import-zone {
  border: 2px dashed var(--n-border-color);
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  transition:
    border-color 0.2s,
    background-color 0.2s;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 120px;
  margin: 0 16px 12px;
}
.import-zone--dragging {
  border-color: #2080f0;
  border-style: solid;
  background-color: rgba(32, 128, 240, 0.08);
}
</style>
