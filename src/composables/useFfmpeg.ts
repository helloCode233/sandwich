import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { useFfmpegStore } from '@/stores/ffmpeg';
import type { FfmpegInfo, DownloadProgress } from '@/types/ffmpeg';

let progressUnlisten: UnlistenFn | null = null;
let statusUnlisten: UnlistenFn | null = null;
let readyUnlisten: UnlistenFn | null = null;

export function useFfmpeg() {
  const store = useFfmpegStore();

  /** Detect FFmpeg on startup. Called once from App.vue on mount. */
  async function detect(): Promise<void> {
    store.status = 'detecting';
    try {
      const info = await invoke<FfmpegInfo>('detect_ffmpeg');
      store.setFfmpegInfo(info);
    } catch (err) {
      store.status = 'error';
      store.downloadError = String(err);
    }
  }

  /** Subscribe to download progress events from Rust. */
  async function subscribeProgress(): Promise<void> {
    progressUnlisten = await listen<DownloadProgress>(
      'ffmpeg-download-progress',
      (event) => {
        store.setDownloadProgress(event.payload);
      },
    );
  }

  /** Subscribe to initial status event (emitted from Rust setup hook). */
  async function subscribeStatus(): Promise<void> {
    statusUnlisten = await listen<FfmpegInfo>('ffmpeg-status', (event) => {
      store.setFfmpegInfo(event.payload);
    });
  }

  /** Subscribe to ffmpeg-ready event (emitted after verification). */
  async function subscribeReady(): Promise<void> {
    readyUnlisten = await listen<FfmpegInfo>('ffmpeg-ready', (event) => {
      store.setFfmpegInfo(event.payload);
      store.status = 'verified';
    });
  }

  /** Open native directory picker for user to choose FFmpeg storage directory. */
  async function selectDirectory(): Promise<string | null> {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 FFmpeg 存储目录 / Choose FFmpeg storage directory',
    });
    if (selected && typeof selected === 'string') {
      store.targetDir = selected;
      return selected;
    }
    return null;
  }

  /** Start downloading FFmpeg to the given directory. */
  async function startDownload(targetDir: string): Promise<void> {
    store.status = 'downloading';
    store.downloadError = null;
    try {
      await invoke('start_download', { targetDir });
      // On success, the 'ffmpeg-ready' event will update status to 'verified'
    } catch (err) {
      store.status = 'error';
      store.downloadError = String(err);
      store.retryCount += 1;
    }
  }

  /** Cancel an active download. */
  async function cancelDownload(): Promise<void> {
    try {
      await invoke('cancel_download');
      store.resetDownload();
    } catch (err) {
      console.error('Failed to cancel download:', err);
    }
  }

  /** Clean up all event listeners. Call on component unmount. */
  function unsubscribeAll(): void {
    progressUnlisten?.();
    statusUnlisten?.();
    readyUnlisten?.();
  }

  return {
    detect,
    subscribeProgress,
    subscribeStatus,
    subscribeReady,
    selectDirectory,
    startDownload,
    cancelDownload,
    unsubscribeAll,
  };
}
