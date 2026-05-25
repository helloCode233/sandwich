import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { FfmpegInfo, DownloadProgress, FfmpegStatus } from '@/types/ffmpeg';

export const useFfmpegStore = defineStore('ffmpeg', () => {
  const status = ref<FfmpegStatus>('detecting');
  const version = ref<string | null>(null);
  const path = ref<string | null>(null);
  const downloadProgress = ref<DownloadProgress>({
    percent: 0,
    downloadedBytes: 0,
    totalBytes: 0,
    speedBytesPerSec: 0,
    stage: 'connecting',
  });
  const downloadError = ref<string | null>(null);
  const retryCount = ref(0);
  const targetDir = ref<string | null>(null);

  // GPU encoder state (null = not yet checked, string = encoder type, "" = confirmed CPU only)
  const gpuEncoder = ref<string | null>(null);
  const gpuChecked = ref(false);

  const isReady = computed(() => status.value === 'found' || status.value === 'verified');
  const hasGpu = computed(() => gpuEncoder.value !== null && gpuEncoder.value !== '');
  const gpuLabel = computed(() => {
    if (!gpuEncoder.value) return 'CPU';
    switch (gpuEncoder.value) {
      case 'Nvenc':
        return 'NVIDIA NVENC';
      case 'Amf':
        return 'AMD AMF';
      case 'VideoToolbox':
        return 'VideoToolbox';
      case 'Vaapi':
        return 'VAAPI';
      default:
        return gpuEncoder.value;
    }
  });
  const needsDownload = computed(() => status.value === 'missing' || status.value === 'outdated');
  const isDownloading = computed(
    () => status.value === 'downloading' || status.value === 'verifying',
  );

  function setFfmpegInfo(info: FfmpegInfo) {
    version.value = info.version;
    path.value = info.path;
    if (info.found) {
      if (info.outdated) {
        status.value = 'outdated';
      } else {
        status.value = 'found';
      }
    } else {
      status.value = 'missing';
    }
  }

  function setGpuEncoder(encoder: string | null) {
    gpuEncoder.value = encoder;
    gpuChecked.value = true;
  }

  function setDownloadProgress(p: DownloadProgress) {
    downloadProgress.value = p;
    if (p.stage === 'complete') {
      status.value = 'verified';
    } else if (p.stage === 'error') {
      status.value = 'error';
      downloadError.value = null; // error message comes from the Rust error return, handled in composable
    }
  }

  function resetDownload() {
    downloadProgress.value = {
      percent: 0,
      downloadedBytes: 0,
      totalBytes: 0,
      speedBytesPerSec: 0,
      stage: 'connecting',
    };
    downloadError.value = null;
    retryCount.value = 0;
    status.value = 'missing';
  }

  return {
    status,
    version,
    path,
    downloadProgress,
    downloadError,
    retryCount,
    targetDir,
    isReady,
    needsDownload,
    isDownloading,
    gpuEncoder,
    gpuChecked,
    hasGpu,
    gpuLabel,
    setFfmpegInfo,
    setGpuEncoder,
    setDownloadProgress,
    resetDownload,
  };
});
