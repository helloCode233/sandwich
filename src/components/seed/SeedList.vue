<script setup lang="ts">
import { NButton, NIcon, NText, NCard, NScrollbar, NSpace } from 'naive-ui';
import { Sparkles, Plus } from 'lucide-vue-next';
import { useMessage } from 'naive-ui';
import { useSeedStore } from '@/stores/seed';
import { useSeed } from '@/composables/useSeed';
import { useI18n } from 'vue-i18n';
import SeedCard from './SeedCard.vue';

const store = useSeedStore();
const { generateSeed } = useSeed();
const message = useMessage();
const { t } = useI18n();

async function onGenerateSeed() {
  const seed = await generateSeed();
  if (seed) {
    message.success(t('seed.generated', { alias: seed.alias }));
  } else {
    message.error(t('notification.operationFailed', { error: 'Generate failed' }));
  }
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header: title + generate button -->
    <div class="flex items-center justify-between px-4 py-3 shrink-0 border-b border-[#2a2a36]">
      <NText strong class="text-base"> {{ t('seed.title') }} ({{ store.seedCount }}) </NText>
      <NButton type="primary" size="small" @click="onGenerateSeed">
        <template #icon>
          <NIcon :size="16">
            <Plus />
          </NIcon>
        </template>
        {{ t('seed.generate') }}
      </NButton>
    </div>

    <!-- Empty State (D-07, D-08) -->
    <div v-if="store.seedCount === 0" class="flex-1 flex items-center justify-center p-6">
      <NCard :bordered="false" class="text-center">
        <NSpace vertical align="center" :size="16">
          <NIcon :size="48" color="#2080f0">
            <Sparkles />
          </NIcon>
          <NText depth="2">
            {{ t('seed.empty') }}
          </NText>
          <NButton type="primary" size="large" @click="onGenerateSeed">
            <template #icon>
              <NIcon :size="18">
                <Sparkles />
              </NIcon>
            </template>
            {{ t('seed.emptyCta') }}
          </NButton>
        </NSpace>
      </NCard>
    </div>

    <!-- Seed List (scrollable) -->
    <NScrollbar v-else class="flex-1">
      <div class="px-4 py-3 space-y-2">
        <SeedCard v-for="s in store.seeds" :key="s.id" :seed="s" />
      </div>
    </NScrollbar>
  </div>
</template>
