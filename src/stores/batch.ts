import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { BatchProgress, BatchResult } from '@/types/batch';

export const useBatchStore = defineStore('batch', () => {
  const progress = ref<BatchProgress>({
    total: 0,
    completed: 0,
    succeeded: 0,
    failed: 0,
    currentFile: null,
  });
  const isProcessing = ref(false);
  const lastResult = ref<BatchResult | null>(null);

  const hasProgress = computed(() => progress.value.total > 0);
  const isComplete = computed(() => !isProcessing.value && hasProgress.value);
  const overallPercent = computed(() => {
    if (progress.value.total === 0) return 0;
    return Math.round((progress.value.completed / progress.value.total) * 100);
  });

  /** Update progress snapshot from batch-progress event or get_batch_status command. */
  function setProgress(p: BatchProgress) {
    progress.value = { ...p };
    // Derive isProcessing: processing is active when completed < total and total > 0
    isProcessing.value = p.total > 0 && p.completed < p.total;
  }

  /** Called when batch starts (sets initial progress with total). */
  function startProcessing(total: number) {
    progress.value = { total, completed: 0, succeeded: 0, failed: 0, currentFile: null };
    isProcessing.value = true;
    lastResult.value = null;
  }

  /** Called when batch completes or is cancelled. */
  function stopProcessing(result: BatchResult) {
    isProcessing.value = false;
    lastResult.value = result;
    progress.value.currentFile = null;
  }

  /** Reset to idle state (called on app init or explicit reset). */
  function resetBatch() {
    progress.value = { total: 0, completed: 0, succeeded: 0, failed: 0, currentFile: null };
    isProcessing.value = false;
    lastResult.value = null;
  }

  return {
    progress,
    isProcessing,
    lastResult,
    hasProgress,
    isComplete,
    overallPercent,
    setProgress,
    startProcessing,
    stopProcessing,
    resetBatch,
  };
});
