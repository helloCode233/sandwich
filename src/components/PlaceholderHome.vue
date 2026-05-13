<script setup lang="ts">
import { NCard, NText, NSpace } from 'naive-ui';
import { useFfmpegStore } from '@/stores/ffmpeg';
import { useI18n } from 'vue-i18n';

const store = useFfmpegStore();
const { t } = useI18n();
</script>

<template>
  <div class="h-screen flex items-center justify-center bg-[#101014]">
    <NCard :bordered="true" class="text-center">
      <NSpace vertical align="center" :size="12">
        <h1 class="text-3xl font-bold text-white m-0">Sandwich</h1>
        <NText depth="2">{{ t('common.tagline') }}</NText>
        <NText v-if="store.version" depth="3" class="text-xs"> FFmpeg {{ store.version }} </NText>
        <NText type="success" class="text-sm">
          {{ t('common.ready') }}
        </NText>
        <NText depth="3" class="text-xs mt-4">
          {{ t('common.awaitingFutureFeatures') }}
        </NText>
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
