<script setup lang="ts">
import { computed } from 'vue';
import { NProgress, NText, NSpace } from 'naive-ui';
import { useBatchStore } from '@/stores/batch';
import { useI18n } from 'vue-i18n';

const batchStore = useBatchStore();
const { t } = useI18n();

const bannerState = computed<'processing' | 'cancelling' | 'complete'>(() => {
  if (batchStore.cancelling) return 'cancelling';
  if (batchStore.isComplete) return 'complete';
  return 'processing';
});

const labelText = computed(() => {
  if (bannerState.value === 'cancelling') return t('batch.cancelling');
  if (bannerState.value === 'complete') {
    return batchStore.lastResult?.failed.length
      ? t('batch.summary.cancelledTitle')
      : t('batch.summary.completeTitle');
  }
  if (batchStore.progress.currentFile) {
    return t('batch.processingFile', { filename: batchStore.progress.currentFile });
  }
  return t('batch.processing');
});

const barColor = computed(() => {
  if (batchStore.overallPercent === 100) return '#18a058';
  return '#2080f0';
});

const barPercent = computed(() => {
  if (batchStore.isComplete) return 100;
  return batchStore.overallPercent;
});
</script>

<template>
  <div class="batch-banner">
    <NSpace align="center" :size="12">
      <NText depth="2" class="text-xs shrink-0">
        {{ labelText }}
      </NText>
      <NProgress
        type="line"
        :percentage="barPercent"
        indicator-placement="inside"
        :height="24"
        :color="barColor"
        class="flex-1"
      />
      <NText v-if="bannerState !== 'complete'" depth="2" class="text-xs shrink-0 tabular-nums">
        {{
          t('batch.progress', {
            completed: batchStore.progress.completed,
            total: batchStore.progress.total,
          })
        }}
      </NText>
    </NSpace>
  </div>
</template>

<style scoped>
.batch-banner {
  background-color: #1a1a24;
  padding: 10px 16px;
  border-bottom: 1px solid #2a2a36;
}
</style>
