<script setup lang="ts">
import { NProgress, NText, NSpace } from 'naive-ui';
import { useBatchStore } from '@/stores/batch';
import { useI18n } from 'vue-i18n';

const batchStore = useBatchStore();
const { t } = useI18n();
</script>

<template>
  <div class="batch-banner">
    <NSpace align="center" :size="12">
      <NText depth="2" class="text-xs shrink-0">
        {{ t('batch.processing') }}
      </NText>
      <NProgress
        type="line"
        :percentage="batchStore.overallPercent"
        indicator-placement="inside"
        :height="24"
        :color="batchStore.overallPercent === 100 ? '#18a058' : '#2080f0'"
        class="flex-1"
      />
      <NText depth="2" class="text-xs shrink-0 tabular-nums">
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
