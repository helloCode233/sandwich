<script setup lang="ts">
import { ref } from 'vue';
import { NButton, NIcon, NText } from 'naive-ui';
import { Upload, FolderOpen } from 'lucide-vue-next';
import { useMessage } from 'naive-ui';
import { open } from '@tauri-apps/plugin-dialog';
import { useQueue } from '@/composables/useQueue';
import { useI18n } from 'vue-i18n';

const { importVideo } = useQueue();
const message = useMessage();
const { t } = useI18n();

const isDragging = ref(false);

/** Supported video file extensions for file dialog filter. */
const VIDEO_EXTENSIONS = ['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv', 'wmv'];

/** Tauri v2 webview exposes the absolute file path on dropped File objects. */
interface TauriFile extends File {
  path?: string;
}

function onDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'copy';
  }
  isDragging.value = true;
}

function onDragLeave(e: DragEvent) {
  // Only set dragging to false if we actually left the zone (not entering a child)
  if (
    e.currentTarget === e.target ||
    !(e.currentTarget as HTMLElement)?.contains(e.relatedTarget as Node)
  ) {
    isDragging.value = false;
  }
}

async function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragging.value = false;

  const files = e.dataTransfer?.files;
  if (!files || files.length === 0) return;

  for (let i = 0; i < files.length; i++) {
    const file = files[i] as TauriFile;
    // Tauri v2 webview exposes absolute path on dropped File objects
    const path = file.path;
    if (path) {
      await importFile(path);
    } else {
      // macOS fallback: if path not available, skip or warn
      console.warn(
        'Dropped file has no path property (macOS may require onDragDropEvent API):',
        file.name,
      );
      message.warning('Unable to get file path. Please use the "Add Videos" button instead.');
    }
  }
}

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
  const entry = await importVideo(filepath);
  if (entry) {
    message.success(t('queue.imported', { filename: entry.filename }));
  } else {
    // importVideo logs detailed error to console; show generic toast
    message.error(t('notification.operationFailed', { error: 'Import failed' }));
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
