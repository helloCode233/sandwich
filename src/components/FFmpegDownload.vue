<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue';
import { NCard, NButton, NProgress, NText, NSpace, NIcon } from 'naive-ui';
import { XCircle, RefreshCw, FolderOpen } from 'lucide-vue-next';
import { useFfmpegStore } from '@/stores/ffmpeg';
import { useFfmpeg } from '@/composables/useFfmpeg';
import { useI18n } from 'vue-i18n';

const emit = defineEmits<{
  (e: 'done'): void;
  (e: 'back'): void;
}>();

const store = useFfmpegStore();
const {
  subscribeProgress,
  selectDirectory,
  startDownload,
  getDefaultDir,
  verifyExisting,
  cancelDownload,
  unsubscribeAll,
} = useFfmpeg();
const { t } = useI18n();

/** Stage-aware label for download progress (not binary-status-aware). */
const stageLabel = computed(() => {
  const stage = store.downloadProgress.stage;
  switch (stage) {
    case 'connecting':
      return t('download.connecting');
    case 'downloading':
      return t('download.downloading');
    case 'extracting':
      return t('download.extracting');
    case 'verifying':
      return t('download.verifying');
    default:
      return store.status === 'verifying' ? t('download.verifying') : t('download.downloading');
  }
});

/** When download+verify completes, notify parent to switch views. */
watch(
  () => store.status,
  (newStatus) => {
    if (newStatus === 'verified') {
      emit('done');
    }
  },
);

/** Format bytes to human-readable string. */
function formatBytes(bytes: number): string {
  if (bytes === 0) return '--';
  const units = ['B', 'KB', 'MB', 'GB'];
  let i = 0;
  let size = bytes;
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024;
    i++;
  }
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

/** Format bytes/sec to human-readable speed. */
function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec === 0) return '--';
  return `${formatBytes(bytesPerSec)}/s`;
}

/** Use the default app-data directory: no picker, one-click download. */
async function onUseDefault() {
  try {
    const dir = await getDefaultDir();
    store.targetDir = dir;
    await startDownload(dir);
  } catch (err) {
    store.status = 'error';
    store.downloadError = String(err);
  }
}

/** User chooses a custom directory then starts download. */
async function onChooseCustom() {
  const dir = await selectDirectory();
  if (dir) {
    store.targetDir = dir;
    await startDownload(dir);
  }
}

/** User selects an existing directory with ffmpeg/ffprobe — verify only, no download. */
async function onUseExisting() {
  const dir = await selectDirectory();
  if (dir) {
    store.targetDir = dir;
    const ok = await verifyExisting(dir);
    if (ok) {
      emit('done');
    }
  }
}

/** Retry download with the same directory. */
async function onRetry() {
  if (store.targetDir) {
    await startDownload(store.targetDir);
  } else {
    await onChooseCustom();
  }
}

/** Cancel active download and go back to status page. */
async function onCancel() {
  await cancelDownload();
  emit('back');
}

onMounted(async () => {
  await subscribeProgress();

  // If we already have a targetDir (resume), auto-start download
  if (store.targetDir && store.status === 'selecting-dir') {
    await startDownload(store.targetDir);
  }
  // If status is 'verifying', verifyExisting was called — handled by its own flow
});

onUnmounted(() => {
  unsubscribeAll();
});
</script>

<template>
  <div class="h-screen flex items-center justify-center bg-[#101014]">
    <NCard :bordered="true" class="w-[500px] max-w-[95vw]">
      <NSpace vertical :size="16">
        <!-- Step 1: Choose directory / use default / use existing -->
        <template v-if="store.status === 'selecting-dir' || store.status === 'missing'">
          <NText strong>
            {{ t('download.selectDir') }}
          </NText>

          <!-- Option A: Default path (recommended) -->
          <NButton type="primary" size="large" block @click="onUseDefault">
            {{ t('download.useDefault') }}
          </NButton>
          <NText depth="3" class="text-xs text-center -mt-2">
            {{ t('download.useDefaultDesc') }}
          </NText>

          <!-- Option B: Custom folder download -->
          <NButton size="medium" block @click="onChooseCustom">
            <template #icon>
              <NIcon><FolderOpen /></NIcon>
            </template>
            {{ t('download.useCustom') }}
          </NButton>

          <!-- Option C: Use existing installation -->
          <NButton size="medium" block @click="onUseExisting">
            <template #icon>
              <NIcon><FolderOpen /></NIcon>
            </template>
            {{ t('download.useExisting') }}
          </NButton>
          <NText depth="3" class="text-xs text-center -mt-2">
            {{ t('download.useExistingDesc') }}
          </NText>
        </template>

        <!-- Step 2: Downloading / Verifying -->
        <template v-else-if="store.status === 'downloading' || store.status === 'verifying'">
          <NText strong>
            {{ t('download.progress') }}
          </NText>
          <NProgress
            type="line"
            :percentage="Math.round(store.downloadProgress.percent)"
            :indicator-placement="'inside'"
            :height="28"
            :status="store.status === 'verifying' ? 'success' : 'default'"
            :processing="store.status === 'downloading'"
          />
          <NSpace justify="space-between" class="w-full">
            <NText depth="3">
              {{ formatBytes(store.downloadProgress.downloadedBytes) }}
              /
              {{
                store.downloadProgress.totalBytes > 0
                  ? formatBytes(store.downloadProgress.totalBytes)
                  : '--'
              }}
            </NText>
            <NText depth="3">
              {{ formatSpeed(store.downloadProgress.speedBytesPerSec) }}
            </NText>
          </NSpace>
          <NText depth="3" class="text-xs text-center">
            {{ stageLabel }}
          </NText>
          <NButton v-if="store.status === 'downloading'" type="warning" block @click="onCancel">
            <template #icon>
              <NIcon><XCircle /></NIcon>
            </template>
            {{ t('download.cancel') }}
          </NButton>
        </template>

        <!-- Step 3: Complete (brief flash before auto-transition) -->
        <template v-else-if="store.status === 'verified'">
          <NText type="success" strong>
            {{ t('download.complete') }}
          </NText>
          <NProgress type="line" :percentage="100" :height="28" status="success" />
        </template>

        <!-- Step 4: Error -->
        <template v-else-if="store.status === 'error'">
          <NIcon :size="48" color="#d03050">
            <XCircle />
          </NIcon>
          <NText type="error">
            {{ store.downloadError || t('download.errorDefault') }}
          </NText>

          <!-- Show retry if under 3 attempts, otherwise show manual instructions -->
          <template v-if="store.retryCount < 3">
            <NButton type="primary" @click="onRetry">
              <template #icon>
                <NIcon><RefreshCw /></NIcon>
              </template>
              {{ t('download.retry', { count: store.retryCount, max: 3 }) }}
            </NButton>
          </template>
          <template v-else>
            <NText depth="3" class="text-xs text-center whitespace-pre-wrap">
              {{ t('download.manualDownload', { dir: store.targetDir || '' }) }}
            </NText>
            <NButton @click="emit('back')">
              {{ t('common.back') }}
            </NButton>
          </template>
        </template>
      </NSpace>
    </NCard>
  </div>
</template>

<style scoped>
.h-screen {
  height: 100vh;
  height: 100dvh;
}
</style>
