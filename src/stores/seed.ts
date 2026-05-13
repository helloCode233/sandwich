import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Seed } from '@/types/seed';

export const useSeedStore = defineStore('seed', () => {
  const seeds = ref<Seed[]>([]);
  const selectedSeedId = ref<string | null>(null);

  const selectedSeed = computed(
    () => seeds.value.find((s) => s.id === selectedSeedId.value) ?? null,
  );
  const seedCount = computed(() => seeds.value.length);

  /** Replace entire seed list. Clears selection if selected seed removed. */
  function setSeeds(list: Seed[]) {
    seeds.value = list;
    if (selectedSeedId.value && !list.find((s) => s.id === selectedSeedId.value)) {
      selectedSeedId.value = null;
    }
  }

  function addSeed(seed: Seed) {
    seeds.value.push(seed);
  }

  function removeSeed(id: string) {
    seeds.value = seeds.value.filter((s) => s.id !== id);
    if (selectedSeedId.value === id) selectedSeedId.value = null;
  }

  function selectSeed(id: string | null) {
    selectedSeedId.value = id;
  }

  return {
    seeds,
    selectedSeedId,
    selectedSeed,
    seedCount,
    setSeeds,
    addSeed,
    removeSeed,
    selectSeed,
  };
});
