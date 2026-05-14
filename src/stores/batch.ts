import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { BatchProgress, BatchResult, PerFileProgress } from '@/types/batch';

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
  const perFileProgress = ref<Map<string, PerFileProgress>>(new Map());
  const cancelling = ref(false);

  const hasProgress = computed(() => progress.value.total > 0);
  const isComplete = computed(() => !isProcessing.value && hasProgress.value);
  const overallPercent = computed(() => {
    if (progress.value.total === 0) return 0;
    return Math.round((progress.value.completed / progress.value.total) * 100);
  });
  const currentFileProgress = computed(() => {
    if (!progress.value.currentFile) return null;
    return perFileProgress.value.get(progress.value.currentFile) ?? null;
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
    cancelling.value = false;
  }

  /** Reset to idle state (called on app init or explicit reset). */
  function resetBatch() {
    progress.value = { total: 0, completed: 0, succeeded: 0, failed: 0, currentFile: null };
    isProcessing.value = false;
    lastResult.value = null;
    cancelling.value = false;
    perFileProgress.value = new Map();
  }

  /** Update per-file progress from batch-file-progress event. */
  function setPerFileProgress(p: PerFileProgress) {
    perFileProgress.value.set(p.file, { ...p });
    // Map.set doesn't trigger Vue reactivity -- replace with new Map to force update
    perFileProgress.value = new Map(perFileProgress.value);
  }

  /** Transition to cancelling state (triggered by batch-cancelling event). */
  function setCancelling(value: boolean) {
    cancelling.value = value;
  }

  return {
    progress,
    isProcessing,
    lastResult,
    perFileProgress,
    cancelling,
    hasProgress,
    isComplete,
    overallPercent,
    currentFileProgress,
    setProgress,
    startProcessing,
    stopProcessing,
    resetBatch,
    setPerFileProgress,
    setCancelling,
  };
});
