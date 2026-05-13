import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useBatchStore } from '@/stores/batch';
import type { BatchProgress, BatchResult, FileResult } from '@/types/batch';

let progressUnlisten: UnlistenFn | null = null;
let fileErrorUnlisten: UnlistenFn | null = null;
let completeUnlisten: UnlistenFn | null = null;
let cancelledUnlisten: UnlistenFn | null = null;

export function useBatch() {
  const store = useBatchStore();

  /** Subscribe to batch events. No initial load — batch state is idle by default. */
  async function subscribe(): Promise<void> {
    progressUnlisten = await listen<BatchProgress>('batch-progress', (event) => {
      store.setProgress(event.payload);
    });

    fileErrorUnlisten = await listen<FileResult>('batch-file-error', (event) => {
      // Store does not track individual file errors independently.
      // The UI layer (composable consumer) should show useMessage() error toast.
      // We log here; caller can attach additional listeners.
      console.warn('Batch file error:', event.payload.file, event.payload.error);
    });

    completeUnlisten = await listen<BatchResult>('batch-complete', (event) => {
      store.stopProcessing(event.payload);
    });

    cancelledUnlisten = await listen<BatchResult>('batch-cancelled', (event) => {
      store.stopProcessing(event.payload);
    });
  }

  /** Start batch processing. Per D-11: concurrency is read from store by Rust, NOT passed here. */
  async function startBatch(seedId: string, outputDir: string): Promise<boolean> {
    try {
      // Get current queue size from the queue store to initialize progress total.
      // The executor will compute this from store state via the caller.
      await invoke('start_batch', { seedId, outputDir });
      return true;
    } catch (err) {
      console.error('Failed to start batch:', err);
      return false;
    }
  }

  /** Cancel an in-progress batch. Completed files are preserved. */
  async function cancelBatch(): Promise<boolean> {
    try {
      await invoke('cancel_batch');
      return true;
    } catch (err) {
      console.error('Failed to cancel batch:', err);
      return false;
    }
  }

  /** Poll current batch status (useful on app re-focus to sync state). */
  async function getBatchStatus(): Promise<BatchProgress | null> {
    try {
      const progress = await invoke<BatchProgress>('get_batch_status');
      store.setProgress(progress);
      return progress;
    } catch (err) {
      console.error('Failed to get batch status:', err);
      return null;
    }
  }

  function unsubscribe(): void {
    progressUnlisten?.();
    fileErrorUnlisten?.();
    completeUnlisten?.();
    cancelledUnlisten?.();
  }

  return { subscribe, startBatch, cancelBatch, getBatchStatus, unsubscribe };
}
