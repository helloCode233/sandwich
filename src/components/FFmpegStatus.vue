<script setup lang="ts">
import { onMounted } from 'vue';
import { NSpin, NCard, NButton, NTag, NSpace, NIcon, NText } from 'naive-ui';
import { AlertCircle, CheckCircle, Download } from 'lucide-vue-next';
import { useFfmpegStore } from '@/stores/ffmpeg';
import { useFfmpeg } from '@/composables/useFfmpeg';
import { useI18n } from 'vue-i18n';

const store = useFfmpegStore();
const { detect, subscribeStatus, subscribeReady } = useFfmpeg();
const { t } = useI18n();

onMounted(async () => {
  await subscribeStatus();
  await subscribeReady();
  await detect();
});
</script>

<template>
  <div class="h-screen flex items-center justify-center bg-[#101014]">
    <NCard :bordered="true" class="w-96 max-w-[90vw]">
      <NSpace vertical align="center" :size="16">
        <!-- Detecting state -->
        <template v-if="store.status === 'detecting'">
          <NSpin :size="32" />
          <NText>{{ t('ffmpeg.detecting') }}</NText>
        </template>

        <!-- Found / Verified state -->
        <template v-else-if="store.status === 'found' || store.status === 'verified'">
          <NIcon :size="48" color="#18a058">
            <CheckCircle />
          </NIcon>
          <NText type="success" strong>{{ t('ffmpeg.found') }}</NText>
          <NTag type="success" :bordered="false">
            {{ store.version }}
          </NTag>
          <NText depth="3" class="text-xs">{{ t('ffmpeg.autoTransit') }}</NText>
        </template>

        <!-- Missing / Outdated state -->
        <template v-else-if="store.status === 'missing' || store.status === 'outdated'">
          <NIcon :size="48" color="#f0a020">
            <AlertCircle />
          </NIcon>
          <NText type="warning" strong>
            {{
              store.status === 'outdated'
                ? t('ffmpeg.outdated', { version: store.version })
                : t('ffmpeg.notFound')
            }}
          </NText>
          <NButton type="primary" size="large" @click="$emit('start-download')">
            <template #icon>
              <NIcon><Download /></NIcon>
            </template>
            {{ t('ffmpeg.download') }}
          </NButton>
        </template>

        <!-- Error state -->
        <template v-else-if="store.status === 'error'">
          <NIcon :size="48" color="#d03050">
            <AlertCircle />
          </NIcon>
          <NText type="error">{{ store.downloadError || t('ffmpeg.errorDetecting') }}</NText>
          <NButton @click="detect()">{{ t('common.retry') }}</NButton>
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
