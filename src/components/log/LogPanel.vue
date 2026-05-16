<script setup lang="ts">
import { NInput, NSelect, NText, NScrollbar, NTag, NButton, NIcon, NPopconfirm } from 'naive-ui';
import { Trash2 } from 'lucide-vue-next';
import { useLogStore } from '@/stores/log';
import { useI18n } from 'vue-i18n';

const logStore = useLogStore();
const { t } = useI18n();

const statusFilterOptions = [
  { label: t('log.all'), value: 'all' },
  { label: t('log.success'), value: 'success' },
  { label: t('log.failure'), value: 'failure' },
];

function formatDuration(ms: number): string {
  if (ms === 0) return '-';
  const totalSec = Math.round(ms / 1000);
  const minutes = Math.floor(totalSec / 60);
  const seconds = totalSec % 60;
  return t('log.duration', { minutes, seconds });
}

function formatTime(iso: string): string {
  // Extract HH:MM:SS from ISO 8601
  return iso.slice(11, 19);
}

function onClear() {
  logStore.clearEntries();
}
</script>

<template>
  <div class="log-panel flex flex-col h-full">
    <!-- Filter bar -->
    <div class="flex items-center gap-2 px-3 py-2 shrink-0 border-b border-[#2a2a36]">
      <NInput
        v-model:value="logStore.searchQuery"
        :placeholder="t('log.searchPlaceholder')"
        size="small"
        clearable
        class="flex-1"
      />
      <NSelect
        v-model:value="logStore.statusFilter"
        :options="statusFilterOptions"
        size="small"
        :style="{ width: '100px' }"
      />
      <NText depth="3" class="text-xs shrink-0">
        {{ logStore.filteredEntries.length }} / {{ logStore.entryCount }}
      </NText>
      <NPopconfirm @positive-click="onClear">
        <template #trigger>
          <NButton size="tiny" quaternary :disabled="logStore.entryCount === 0">
            <template #icon>
              <NIcon :size="14">
                <Trash2 />
              </NIcon>
            </template>
          </NButton>
        </template>
        {{ t('log.clearConfirm') }}
      </NPopconfirm>
    </div>

    <!-- Empty state -->
    <div
      v-if="logStore.entryCount === 0"
      class="flex-1 flex flex-col items-center justify-center gap-3 text-center p-8"
    >
      <div class="text-4xl opacity-20">&#128196;</div>
      <NText depth="3" class="text-sm">
        {{ t('log.empty') }}
      </NText>
    </div>

    <!-- No filter results -->
    <div
      v-else-if="logStore.filteredEntries.length === 0"
      class="flex-1 flex items-center justify-center p-8"
    >
      <NText depth="3" class="text-sm">
        {{ t('log.noResults') }}
      </NText>
    </div>

    <!-- Log entries (scrollable) -->
    <NScrollbar v-else class="flex-1">
      <div class="space-y-1 px-3 py-1">
        <div
          v-for="entry in logStore.filteredEntries"
          :key="entry.id"
          class="log-entry flex items-start gap-2 py-1.5 px-2 rounded text-xs"
          :class="entry.status === 'success' ? 'hover:bg-[#1a2a1a]' : 'hover:bg-[#2a1a1a]'"
        >
          <!-- Status icon -->
          <NTag
            :type="entry.status === 'success' ? 'success' : 'error'"
            :bordered="false"
            size="tiny"
            class="shrink-0 mt-0.5"
          >
            {{ entry.status === 'success' ? '\u2713' : '\u2717' }}
          </NTag>

          <!-- Content -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <NText strong class="truncate text-xs">
                {{ entry.file }}
              </NText>
              <NText depth="3" class="text-[11px] shrink-0">
                {{ entry.seedAlias }}
              </NText>
              <NText depth="3" class="text-[11px] shrink-0 ml-auto">
                {{ formatTime(entry.timestamp) }}
              </NText>
            </div>
            <div class="flex items-center gap-2 mt-0.5 text-[11px]">
              <NTag :type="entry.modified ? 'success' : 'warning'" :bordered="false" size="tiny">
                {{ entry.modified ? t('log.modified') : t('log.unchanged') }}
              </NTag>
              <!-- MD5 abbreviated -->
              <NText depth="3" class="font-mono">
                {{ entry.md5Before.slice(0, 8) }} &rarr; {{ entry.md5After.slice(0, 8) }}
              </NText>
              <!-- Error message if failed -->
              <NText v-if="entry.errorMessage" type="error" class="truncate">
                {{ t('log.error') }}: {{ entry.errorMessage }}
              </NText>
              <!-- Duration -->
              <NText depth="3" class="ml-auto shrink-0">
                {{ formatDuration(entry.durationMs) }}
              </NText>
            </div>
            <!-- Output path (success only) -->
            <NText
              v-if="entry.outputPath"
              depth="3"
              class="text-[10px] font-mono truncate block mt-0.5"
            >
              {{ entry.outputPath }}
            </NText>
          </div>
        </div>
      </div>
    </NScrollbar>
  </div>
</template>
