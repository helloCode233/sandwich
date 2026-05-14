<script setup lang="ts">
import {
  NButton,
  NIcon,
  NText,
  NTag,
  NScrollbar,
  NProgress,
  useDialog,
  useMessage,
} from 'naive-ui';
import { Clapperboard, Trash2, AlertCircle, CheckCircle, Plus } from 'lucide-vue-next';
import { open } from '@tauri-apps/plugin-dialog';
import { useQueueStore } from '@/stores/queue';
import { useQueue } from '@/composables/useQueue';
import { useI18n } from 'vue-i18n';
import { useBatchStore } from '@/stores/batch';
import type { VideoEntry } from '@/types/video';

const store = useQueueStore();
const { importVideo, removeFromQueue, clearQueue } = useQueue();
const dialog = useDialog();
const message = useMessage();
const { t } = useI18n();

const batchStore = useBatchStore();

function isCurrentFile(filename: string): boolean {
  return batchStore.isProcessing && batchStore.progress.currentFile === filename;
}

function fileProgressFor(filename: string) {
  if (!isCurrentFile(filename)) return null;
  return batchStore.currentFileProgress;
}

const VIDEO_EXTENSIONS = ['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv', 'wmv'];

/** Format duration seconds to MM:SS or HH:MM:SS. */
function formatDuration(secs: number): string {
  if (secs < 0 || !isFinite(secs)) return '--:--';
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = Math.floor(secs % 60);
  if (h > 0) {
    return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

/** Format bytes to human-readable string. */
function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  let size = bytes;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

/** Format codec name to uppercase. */
function formatCodec(codec: string): string {
  return codec.toUpperCase();
}

/** Build metadata string: "05:32 | 1920x1080 | 128.5 MB | H264" */
function metadataLine(entry: VideoEntry): string {
  const m = entry.metadata;
  return [
    formatDuration(m.durationSecs),
    `${m.width}x${m.height}`,
    formatBytes(m.sizeBytes),
    formatCodec(m.codec),
  ].join(' | ');
}

async function onRemove(index: number) {
  const filename = store.entries[index]?.filename || '';
  const ok = await removeFromQueue(index);
  if (ok) {
    message.success(t('queue.removed', { filename }));
  } else {
    message.error(t('notification.operationFailed', { error: 'Remove failed' }));
  }
}

function onClearAll() {
  dialog.warning({
    title: t('queue.clearAll'),
    content: t('queue.clearConfirm', { count: store.entryCount }),
    positiveText: t('queue.clearAll'),
    negativeText: t('common.back'),
    onPositiveClick: async () => {
      const ok = await clearQueue();
      if (ok) {
        message.success(t('queue.cleared'));
      } else {
        message.error(t('notification.operationFailed', { error: 'Clear failed' }));
      }
    },
  });
}

async function onAddVideoClick() {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'Video Files', extensions: VIDEO_EXTENSIONS }],
  });
  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    for (const path of paths) {
      const entry = await importVideo(path);
      if (entry) {
        message.success(t('queue.imported', { filename: entry.filename }));
      }
    }
  }
}
</script>

<template>
  <div class="flex flex-col flex-1 min-h-0 px-4">
    <!-- Header: title + clear all -->
    <div class="flex items-center justify-between py-2 shrink-0">
      <NText strong class="text-sm"> {{ t('queue.title') }} ({{ store.entryCount }}) </NText>
      <NButton
        v-if="store.entryCount > 0 && !batchStore.isProcessing"
        size="tiny"
        quaternary
        type="error"
        @click="onClearAll"
      >
        {{ t('queue.clearAll') }}
      </NButton>
    </div>

    <!-- Empty State (D-07, D-08) -->
    <div v-if="store.entryCount === 0" class="flex-1 flex items-center justify-center py-12">
      <div class="text-center">
        <NIcon :size="48" color="#2080f0">
          <Clapperboard />
        </NIcon>
        <NText depth="2" class="mt-3 block">
          {{ t('queue.empty') }}
        </NText>
        <NText depth="3" class="text-xs mt-1 block">
          {{ t('queue.emptyInstruction') }}
        </NText>
        <NButton
          v-if="!batchStore.isProcessing"
          type="primary"
          class="mt-4"
          @click="onAddVideoClick"
        >
          <template #icon>
            <NIcon :size="16">
              <Plus />
            </NIcon>
          </template>
          {{ t('queue.addVideo') }}
        </NButton>
      </div>
    </div>

    <!-- Queue List -->
    <NScrollbar v-else class="flex-1">
      <div class="space-y-1.5 pb-4">
        <div v-for="(entry, index) in store.entries" :key="entry.filepath" class="queue-item">
          <div class="flex items-center justify-between gap-3 py-2 px-3 rounded-md bg-[#1a1a1f]">
            <!-- File info -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <NText strong class="truncate text-sm">
                  {{ entry.filename }}
                </NText>
                <NTag
                  :type="entry.status === 'valid' ? 'success' : 'warning'"
                  :bordered="false"
                  size="small"
                >
                  <template #icon>
                    <NIcon :size="12">
                      <CheckCircle v-if="entry.status === 'valid'" />
                      <AlertCircle v-else />
                    </NIcon>
                  </template>
                  {{ entry.status === 'valid' ? t('queue.valid') : t('queue.invalid') }}
                </NTag>
              </div>
              <NText depth="3" class="text-xs mt-0.5 block">
                {{ metadataLine(entry) }}
              </NText>
              <div
                v-if="isCurrentFile(entry.filename) && fileProgressFor(entry.filename)"
                class="mt-2"
              >
                <NProgress
                  type="line"
                  :percentage="fileProgressFor(entry.filename)!.percent"
                  indicator-placement="inside"
                  :height="18"
                  :color="fileProgressFor(entry.filename)!.percent === 100 ? '#18a058' : '#2080f0'"
                />
                <div class="flex justify-between mt-0.5">
                  <NText depth="3" class="text-xs">
                    {{
                      t('batch.fileProgress', {
                        current: fileProgressFor(entry.filename)!.currentFrame,
                        total: fileProgressFor(entry.filename)!.totalFrames,
                      })
                    }}
                  </NText>
                  <NText depth="3" class="text-xs">
                    {{
                      t('batch.fileEta', {
                        minutes: Math.ceil(fileProgressFor(entry.filename)!.remainingSeconds / 60),
                      })
                    }}
                  </NText>
                </div>
              </div>
            </div>

            <!-- Remove button -->
            <NButton
              size="tiny"
              quaternary
              type="error"
              :disabled="batchStore.isProcessing"
              @click="onRemove(index)"
            >
              <template #icon>
                <NIcon :size="14">
                  <Trash2 />
                </NIcon>
              </template>
            </NButton>
          </div>
        </div>
      </div>
    </NScrollbar>
  </div>
</template>

<style scoped>
.queue-item {
  transition: background-color 0.15s;
}
</style>
