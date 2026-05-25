import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useQueueStore } from '@/stores/queue';
import type { VideoEntry } from '@/types/video';

let queueUpdatedUnlisten: UnlistenFn | null = null;
let videoImportedUnlisten: UnlistenFn | null = null;
let debugLogUnlisten: UnlistenFn | null = null;

interface DebugLogEvent {
  file: string;
  level: string;
  message: string;
}

export function useQueue() {
  const store = useQueueStore();

  /** Fetch the authoritative queue list from Rust. */
  async function loadQueue(): Promise<void> {
    try {
      const list = await invoke<VideoEntry[]>('get_queue');
      store.setEntries(list);
    } catch (err) {
      console.error('Failed to load queue:', err);
    }
  }

  /** Subscribe to queue events and perform initial load. */
  async function subscribe(): Promise<void> {
    // queue-updated emits () — invalidation signal, re-fetch authority
    queueUpdatedUnlisten = await listen('queue-updated', () => {
      loadQueue();
    });
    // video-imported emits VideoEntry — use payload for optimistic update
    videoImportedUnlisten = await listen<VideoEntry>('video-imported', (event) => {
      store.addEntry(event.payload);
    });
    // ffmpeg-debug-log emits diagnostic info from Rust (import, probe, etc.)
    debugLogUnlisten = await listen<DebugLogEvent>('ffmpeg-debug-log', (event) => {
      console.log(`[${event.payload.file}] ${event.payload.message}`);
    });
    await loadQueue();
  }

  /** Import a video file into the queue. Called after file dialog or drag-drop. */
  async function importVideo(filepath: string): Promise<{ entry: VideoEntry } | { error: string }> {
    try {
      const entry = await invoke<VideoEntry>('import_video', { filepath });
      return { entry };
    } catch (err) {
      const msg = typeof err === 'string' ? err : String(err);
      console.error('Failed to import video:', msg);
      return { error: msg };
    }
  }

  /** Remove a single video from the queue by its position index. */
  async function removeFromQueue(index: number): Promise<boolean> {
    try {
      await invoke('remove_from_queue', { index });
      store.removeEntry(index);
      return true;
    } catch (err) {
      console.error('Failed to remove from queue:', err);
      return false;
    }
  }

  /** Clear all videos from the queue. Per D-09: caller should confirm first. */
  async function clearQueue(): Promise<boolean> {
    try {
      await invoke('clear_queue');
      store.clearQueue();
      return true;
    } catch (err) {
      console.error('Failed to clear queue:', err);
      return false;
    }
  }

  function unsubscribe(): void {
    queueUpdatedUnlisten?.();
    videoImportedUnlisten?.();
    debugLogUnlisten?.();
  }

  return { loadQueue, subscribe, importVideo, removeFromQueue, clearQueue, unsubscribe };
}
