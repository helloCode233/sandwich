<script setup lang="ts">
import { computed } from 'vue';
import { NButton, NIcon, NText, NScrollbar } from 'naive-ui';
import { CheckCircle, AlertCircle, XCircle } from 'lucide-vue-next';
import { useBatchStore } from '@/stores/batch';
import { useI18n } from 'vue-i18n';

const batchStore = useBatchStore();
const { t } = useI18n();

const result = computed(() => batchStore.lastResult);

const wasCancelled = computed(() => {
  if (!result.value) return false;
  const totalFiles = result.value.succeeded.length + result.value.failed.length;
  return totalFiles < batchStore.progress.total;
});

const titleKey = computed(() =>
  wasCancelled.value ? 'batch.summary.cancelledTitle' : 'batch.summary.completeTitle',
);

const bodyKey = computed(() =>
  wasCancelled.value ? 'batch.summary.cancelledBody' : 'batch.summary.completeBody',
);

const succeededCount = computed(() => result.value?.succeeded.length ?? 0);
const failedCount = computed(() => result.value?.failed.length ?? 0);
const totalCount = computed(() => succeededCount.value + failedCount.value);

function onClear() {
  batchStore.resetBatch();
}
</script>

<template>
  <div v-if="batchStore.isComplete && result" class="batch-summary">
    <div class="flex items-center gap-2 mb-3">
      <NIcon :size="24" :color="failedCount > 0 ? '#f0a020' : '#18a058'">
        <CheckCircle v-if="failedCount === 0 && !wasCancelled" />
        <AlertCircle v-else />
      </NIcon>
      <NText strong class="text-base">
        {{ t(titleKey) }}
      </NText>
    </div>

    <NText depth="2" class="text-sm block mb-3">
      {{ t(bodyKey, { succeeded: succeededCount, failed: failedCount, total: totalCount }) }}
    </NText>

    <div v-if="succeededCount > 0" class="mb-3">
      <NText strong class="text-sm block mb-1">
        {{ t('batch.summary.succeededSection', { count: succeededCount }) }}
      </NText>
      <NScrollbar style="max-height: 200px">
        <div class="space-y-1">
          <div
            v-for="(outputPath, idx) in result.succeeded"
            :key="'ok-' + idx"
            class="summary-row summary-row--success"
          >
            <NIcon :size="14" color="#18a058" class="shrink-0 mt-0.5">
              <CheckCircle />
            </NIcon>
            <NText depth="2" class="text-xs">
              {{
                t('batch.summary.outputPath', {
                  filename: outputPath.split('/').pop() || outputPath,
                  outputPath,
                })
              }}
            </NText>
          </div>
        </div>
      </NScrollbar>
    </div>

    <div v-if="failedCount > 0" class="mb-3">
      <NText strong class="text-sm block mb-1" type="error">
        {{ t('batch.summary.failedSection', { count: failedCount }) }}
      </NText>
      <NScrollbar style="max-height: 200px">
        <div class="space-y-1">
          <div
            v-for="(fileResult, idx) in result.failed"
            :key="'fail-' + idx"
            class="summary-row summary-row--error"
          >
            <NIcon :size="14" color="#d03050" class="shrink-0 mt-0.5">
              <XCircle />
            </NIcon>
            <NText depth="2" class="text-xs" type="error">
              {{
                t('batch.summary.fileError', {
                  filename: fileResult.file,
                  errorMessage: fileResult.error,
                })
              }}
            </NText>
          </div>
        </div>
      </NScrollbar>
    </div>

    <div class="flex justify-end">
      <NButton size="small" @click="onClear">
        {{ t('batch.summary.clearResults') }}
      </NButton>
    </div>
  </div>
</template>

<style scoped>
.batch-summary {
  background-color: #1a1a24;
  padding: 12px 16px;
  border-bottom: 1px solid #2a2a36;
  max-height: 60vh;
  overflow-y: auto;
}

.summary-row {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 4px;
  background-color: #1a1a1f;
}

.summary-row--success {
  border-left: 2px solid #18a058;
}

.summary-row--error {
  border-left: 2px solid #d03050;
}
</style>
