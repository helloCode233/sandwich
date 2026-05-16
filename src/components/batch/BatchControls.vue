<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { NSelect, NButton, NIcon, NText, NSpace, useDialog } from 'naive-ui';
import { Play, Square, FolderOpen } from 'lucide-vue-next';
import { useMessage } from 'naive-ui';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
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
const dialog = useDialog();
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
const startDisabled = computed(
  () => seedStore.selectedSeedIds.length === 0 || queueStore.validCount === 0,
);

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
  const dir = await openDialog({
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

/** Open the output directory in the system file manager. */
async function onOpenOutputDir() {
  if (!outputDir.value) return;
  try {
    await invoke('open_file_manager', { path: outputDir.value });
  } catch (err) {
    console.error('Failed to open output directory:', err);
    message.error(t('notification.operationFailed', { error: String(err) }));
  }
}

/** Start batch processing. */
async function onStart() {
  if (seedStore.selectedSeedIds.length === 0) {
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
  if (!outputDir.value || outputDir.value.trim() === '') {
    message.warning(t('batch.noOutputDir'));
    return;
  }

  const totalJobs = queueStore.entryCount * seedStore.selectedSeedIds.length;
  const ok = await startBatch(seedStore.selectedSeedIds, outputDir.value, totalJobs);
  if (!ok) {
    message.error(t('notification.operationFailed', { error: 'Batch start failed' }));
  }
}

/** Cancel in-progress batch with confirmation dialog. */
function onCancel() {
  dialog.warning({
    title: t('batch.cancelConfirmTitle'),
    content: t('batch.cancelConfirmBody'),
    positiveText: t('batch.cancel'),
    negativeText: t('batch.keepProcessing'),
    onPositiveClick: async () => {
      const ok = await cancelBatch();
      if (!ok) {
        message.error(t('notification.operationFailed', { error: 'Cancel failed' }));
      }
    },
  });
}

// Strength tier options for seed generation (D-03, D-07)
const strengthTierOptions = [
  { label: t('seed.strength.conservative'), value: 'conservative' },
  { label: t('seed.strength.standard'), value: 'standard' },
  { label: t('seed.strength.aggressive'), value: 'aggressive' },
];

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
      <!-- Strength Tier Selector (D-03, D-07) -->
      <div>
        <NText depth="2" class="text-xs mb-1 block">
          {{ t('seed.strengthTier') }}
        </NText>
        <NSelect
          v-model:value="seedStore.strengthTier"
          :options="strengthTierOptions"
          :placeholder="t('seed.strengthTier')"
          :disabled="batchStore.isProcessing"
        />
      </div>

      <!-- Seed Selector — multi-select (Phase 5: MULTI-01) -->
      <NSelect
        v-model:value="seedStore.selectedSeedIds"
        :options="seedOptions"
        :placeholder="t('batch.selectSeeds')"
        :disabled="batchStore.isProcessing"
        multiple
        filterable
        clearable
      />

      <!-- Concurrency (D-11) -->
      <div>
        <NText depth="2" class="text-xs mb-1 block">
          {{ t('batch.concurrency') }}
        </NText>
        <NSelect
          :value="concurrency"
          :options="concurrencyOptions"
          :placeholder="t('batch.concurrency')"
          :disabled="batchStore.isProcessing"
          @update:value="(v: number) => onConcurrencyChange(v)"
        />
        <NText depth="3" class="text-[11px] mt-1">
          {{ t('batch.concurrencyDesc') }}
        </NText>
      </div>

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
        <NButton size="small" quaternary @click="onOpenOutputDir">
          {{ t('batch.openDir') }}
        </NButton>
      </div>

      <!-- Start / Cancel Button -->
      <NButton
        v-if="!batchStore.isProcessing && !batchStore.cancelling"
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
      <NButton v-else-if="batchStore.cancelling" type="error" size="large" block disabled loading>
        {{ t('batch.cancelling') }}
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
