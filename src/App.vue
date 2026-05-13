<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import {
  NConfigProvider,
  NGlobalStyle,
  NDialogProvider,
  NMessageProvider,
  NNotificationProvider,
  darkTheme,
  zhCN as naiveZhCN,
  enUS as naiveEnUS,
  dateZhCN,
  dateEnUS,
} from 'naive-ui';
import { useI18n } from 'vue-i18n';
import { useFfmpegStore } from '@/stores/ffmpeg';
import FFmpegStatus from '@/components/FFmpegStatus.vue';
import FFmpegDownload from '@/components/FFmpegDownload.vue';
import MainLayout from '@/components/MainLayout.vue';

const { locale } = useI18n();
const ffmpegStore = useFfmpegStore();

// Naive UI locale follows vue-i18n locale
const naiveLocale = computed(() => {
  if (locale.value === 'zh-CN') return naiveZhCN;
  return naiveEnUS;
});
const naiveDateLocale = computed(() => {
  if (locale.value === 'zh-CN') return dateZhCN;
  return dateEnUS;
});

// Current view state based on FFmpeg status
const showDownload = ref(false);

function onStartDownload() {
  showDownload.value = true;
}

function onDownloadDone() {
  showDownload.value = false;
}

function onDownloadBack() {
  showDownload.value = false;
}

onMounted(() => {
  // Locale can be detected from navigator.language if desired;
  // for now, default to zh-CN as set in i18n.ts
});
</script>

<template>
  <NConfigProvider :theme="darkTheme" :locale="naiveLocale" :date-locale="naiveDateLocale">
    <NGlobalStyle />
    <n-dialog-provider>
      <n-message-provider :max="5" placement="top-right">
        <n-notification-provider placement="top-right" :max="5">
          <!-- Download page (full-screen overlay when download is active) -->
          <FFmpegDownload v-if="showDownload" @done="onDownloadDone" @back="onDownloadBack" />

          <!-- Main application layout (shown after FFmpeg is verified) -->
          <MainLayout v-else-if="ffmpegStore.isReady" />

          <!-- Status page (default: detecting, missing, found transition, error) -->
          <FFmpegStatus v-else @start-download="onStartDownload" />
        </n-notification-provider>
      </n-message-provider>
    </n-dialog-provider>
  </NConfigProvider>
</template>
