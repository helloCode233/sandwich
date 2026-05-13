<script setup lang="ts">
import { ref } from 'vue';
import { NCard, NButton, NIcon, NText, NTag, NPopconfirm, NInput } from 'naive-ui';
import { Pencil, Copy, Trash2, Zap } from 'lucide-vue-next';
import { useMessage } from 'naive-ui';
import type { Seed } from '@/types/seed';
import { useSeedStore } from '@/stores/seed';
import { useSeed } from '@/composables/useSeed';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
  seed: Seed;
}>();

const store = useSeedStore();
const { renameSeed, deleteSeed, copySeed } = useSeed();
const message = useMessage();
const { t } = useI18n();

const isHovered = ref(false);
const isRenaming = ref(false);
const renameValue = ref('');

const isSelected = () => store.selectedSeedId === props.seed.id;

/** Display up to 3 operation type tags; show "+N more" if >3. */
function visibleOps() {
  return props.seed.operations.slice(0, 3).map((o) => o.opType);
}
function overflowCount() {
  return Math.max(0, props.seed.operations.length - 3);
}

function onSelect() {
  // Toggle selection: click same card deselects, click different selects
  store.selectSeed(isSelected() ? null : props.seed.id);
}

function startRename() {
  renameValue.value = props.seed.alias;
  isRenaming.value = true;
}

async function confirmRename() {
  const trimmed = renameValue.value.trim();
  if (!trimmed) {
    message.warning(t('seed.aliasEmpty'));
    return;
  }
  const ok = await renameSeed(props.seed.id, trimmed);
  if (ok) {
    message.success(t('seed.renamed', { alias: trimmed }));
  } else {
    message.error(t('notification.operationFailed', { error: 'Rename failed' }));
  }
  isRenaming.value = false;
}

function cancelRename() {
  isRenaming.value = false;
}

async function onCopy() {
  const newSeed = await copySeed(props.seed.id);
  if (newSeed) {
    message.success(t('seed.copied', { alias: newSeed.alias }));
  } else {
    message.error(t('notification.operationFailed', { error: 'Copy failed' }));
  }
}

async function onDelete() {
  const ok = await deleteSeed(props.seed.id);
  if (ok) {
    message.success(t('seed.deleted'));
  } else {
    message.error(t('notification.operationFailed', { error: 'Delete failed' }));
  }
}
</script>

<template>
  <NCard
    :bordered="true"
    :class="[
      'cursor-pointer transition-all duration-200',
      isSelected() ? 'border-[#2080f0]! border-2!' : 'border-transparent',
    ]"
    @click="onSelect"
    @mouseenter="isHovered = true"
    @mouseleave="isHovered = false"
  >
    <div class="flex items-center justify-between gap-2">
      <div class="flex-1 min-w-0">
        <!-- Selected indicator + Alias -->
        <div class="flex items-center gap-1.5">
          <NIcon v-if="isSelected()" :size="14" color="#2080f0">
            <Zap />
          </NIcon>
          <!-- Inline rename: NInput replaces alias text when renaming -->
          <NInput
            v-if="isRenaming"
            :value="renameValue"
            :placeholder="t('seed.aliasPlaceholder')"
            size="small"
            class="flex-1"
            @update:value="(v: string) => (renameValue = v)"
            @keyup.enter="confirmRename"
            @keyup.esc="cancelRename"
            @blur="cancelRename"
          />
          <NText v-else strong class="truncate">
            {{ props.seed.alias }}
          </NText>
        </div>

        <!-- Operation type tags -->
        <div class="flex items-center gap-1 mt-1.5 flex-wrap">
          <NTag v-for="op in visibleOps()" :key="op" type="info" :bordered="false" size="small">
            {{ op }}
          </NTag>
          <NTag v-if="overflowCount() > 0" :bordered="false" size="small">
            +{{ overflowCount() }} more
          </NTag>
        </div>

        <!-- Creation timestamp -->
        <NText depth="3" class="text-xs block mt-1">
          {{ props.seed.createdAt }}
        </NText>
      </div>

      <!-- Action buttons (hover reveal per D-06) -->
      <div
        v-show="isHovered"
        class="flex items-center gap-1 shrink-0"
        :class="['transition-opacity duration-200', isHovered ? 'opacity-100' : 'opacity-0']"
        @click.stop
      >
        <NButton size="tiny" quaternary @click="startRename">
          <template #icon>
            <NIcon :size="16">
              <Pencil />
            </NIcon>
          </template>
        </NButton>
        <NButton size="tiny" quaternary @click="onCopy">
          <template #icon>
            <NIcon :size="16">
              <Copy />
            </NIcon>
          </template>
        </NButton>
        <NPopconfirm @positive-click="onDelete">
          <template #trigger>
            <NButton size="tiny" quaternary type="error">
              <template #icon>
                <NIcon :size="16">
                  <Trash2 />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t('seed.deleteConfirm', { alias: props.seed.alias }) }}
        </NPopconfirm>
      </div>
    </div>
  </NCard>
</template>
