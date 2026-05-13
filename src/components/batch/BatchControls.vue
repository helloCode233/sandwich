<script setup lang="ts">
/* global console */
import { ref, computed, onMounted } from 'vue';
import { NSelect, NButton, NIcon, NText, NSpace } from 'naive-ui';
import { Play, Square, FolderOpen } from 'lucide-vue-next';
import { useMessage } from 'naive-ui';
import { open } from '@tauri-apps/plugin-dialog';
import { Store } from '@tauri-apps/plugin-store';
import { useSeedStore } from '@/stores/seed';
import { useQueueStore } from '@/stores/queue';
import { useBatchStore } from '@/stores/batch';
import { useBatch } from '@/composables/useBatch';
import { useI18n } from 'vue-i18n';

const seedStore = useSeedStore();
const queueStore = useQueueStore();
const batchStore = useBatchStore();
const { startBatch, cancelBatch } = useBatch();
const message = useMessage();
const { t } = useI18n();

const concurrency = ref<number>(1);
const outputDir = ref<string>('');
let store: Store | null = null;

/** Seed options for NSelect: value=id, label=alias */
const seedOptions = computed(() =>
  seedStore.seeds.map((s) => ({
    label: s.alias,
    value: s.id,
  })),
);

/** Start disabled when no seed selected or queue has no valid entries. */
const startDisabled = computed(() => !seedStore.selectedSeedId || queueStore.validCount === 0);

/** Load persisted preferences from tauri-plugin-store on mount. */
async function loadPreferences() {
  try {
    store = await Store.load('sandwich-config.json');
    const savedConcurrency = await store.get<number>('concurrency');
    if (savedConcurrency && [1, 2, 3, 4].includes(savedConcurrency)) {
      concurrency.value = savedConcurrency;
    }
    const savedOutputDir = await store.get<string>('output_dir');
    if (savedOutputDir) {
      outputDir.value = savedOutputDir;
    } else {
      // Default per D-12: ~/Videos/sandwich-output/
      outputDir.value = t('batch.defaultOutputDir');
    }
  } catch (err) {
    console.warn('Failed to load batch preferences:', err);
    outputDir.value = t('batch.defaultOutputDir');
  }
}

/** Persist concurrency choice to tauri-plugin-store per D-11. */
async function onConcurrencyChange(value: number) {
  concurrency.value = value;
  try {
    if (store) {
      await store.set('concurrency', value);
      await store.save();
    }
  } catch (err) {
    console.error('Failed to save concurrency preference:', err);
  }
}

/** Open native directory picker for output directory per D-12. */
async function onChangeOutputDir() {
  const dir = await open({
    directory: true,
    multiple: false,
    title: t('batch.outputDir'),
  });
  if (dir && typeof dir === 'string') {
    outputDir.value = dir;
    message.success(t('batch.changeDir'));
    // Persist output dir
    try {
      if (store) {
        await store.set('output_dir', dir);
        await store.save();
      }
    } catch (err) {
      console.error('Failed to persist output directory:', err);
    }
  }
}

/** Start batch processing. */
async function onStart() {
  if (!seedStore.selectedSeedId) {
    message.warning(t('batch.noSeedSelected'));
    return;
  }
  if (queueStore.validCount === 0) {
    message.warning(t('batch.queueEmpty'));
    return;
  }
  if (batchStore.isProcessing) {
    message.warning(t('batch.alreadyRunning'));
    return;
  }

  // Resolve output dir: replace ~ with HOME
  let resolvedDir = outputDir.value;
  if (resolvedDir.startsWith('~/')) {
    // In Tauri, Rust resolves ~ automatically. We pass as-is.
  }

  const ok = await startBatch(seedStore.selectedSeedId, resolvedDir);
  if (ok) {
    // Progress will come via batch-progress events
  } else {
    message.error(t('notification.operationFailed', { error: 'Batch start failed' }));
  }
}

/** Cancel in-progress batch. */
async function onCancel() {
  const ok = await cancelBatch();
  if (!ok) {
    message.error(t('notification.operationFailed', { error: 'Cancel failed' }));
  }
}

// Concurrency options per D-11
const concurrencyOptions = [
  { label: '1', value: 1 },
  { label: '2', value: 2 },
  { label: '3', value: 3 },
  { label: '4', value: 4 },
];

onMounted(() => {
  loadPreferences();
});
</script>

<template>
  <div class="px-4 py-3">
    <NText strong class="text-sm block mb-3">
      {{ t('batch.title') }}
    </NText>

    <NSpace vertical :size="12">
      <!-- Seed Selector -->
      <NSelect
        :value="seedStore.selectedSeedId"
        :options="seedOptions"
        :placeholder="t('batch.selectSeed')"
        :disabled="batchStore.isProcessing"
        filterable
        clearable
        @update:value="(v: string | null) => seedStore.selectSeed(v)"
      />

      <!-- Concurrency (D-11) -->
      <NSelect
        :value="concurrency"
        :options="concurrencyOptions"
        :placeholder="t('batch.concurrency')"
        :disabled="batchStore.isProcessing"
        @update:value="(v: number) => onConcurrencyChange(v)"
      />

      <!-- Output Directory (D-12) -->
      <div class="flex items-center gap-2">
        <NText depth="3" class="text-xs flex-1 truncate">
          {{ outputDir }}
        </NText>
        <NButton
          size="small"
          quaternary
          :disabled="batchStore.isProcessing"
          @click="onChangeOutputDir"
        >
          <template #icon>
            <NIcon :size="14">
              <FolderOpen />
            </NIcon>
          </template>
          {{ t('batch.changeDir') }}
        </NButton>
      </div>

      <!-- Start / Cancel Button -->
      <NButton
        v-if="!batchStore.isProcessing"
        type="primary"
        size="large"
        block
        :disabled="startDisabled"
        @click="onStart"
      >
        <template #icon>
          <NIcon :size="18">
            <Play />
          </NIcon>
        </template>
        {{ t('batch.start') }}
      </NButton>
      <NButton v-else type="error" size="large" block @click="onCancel">
        <template #icon>
          <NIcon :size="18">
            <Square />
          </NIcon>
        </template>
        {{ t('batch.cancel') }}
      </NButton>
    </NSpace>
  </div>
</template>
