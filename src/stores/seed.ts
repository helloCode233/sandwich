import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Seed } from '@/types/seed';

export const useSeedStore = defineStore('seed', () => {
  const seeds = ref<Seed[]>([]);
  const selectedSeedIds = ref<string[]>([]);
  /** Currently selected strength tier for seed generation (D-07).
   *  Persisted in component; not stored in Rust backend per-seed until generation. */
  const strengthTier = ref<'conservative' | 'standard' | 'aggressive'>('standard');

  const selectedSeeds = computed(() =>
    seeds.value.filter((s) => selectedSeedIds.value.includes(s.id)),
  );
  const seedCount = computed(() => seeds.value.length);
  const hasSelection = computed(() => selectedSeedIds.value.length > 0);

  /** Replace entire seed list. Clears selection if a selected seed was removed. */
  function setSeeds(list: Seed[]) {
    seeds.value = list;
    selectedSeedIds.value = selectedSeedIds.value.filter((id) => list.some((s) => s.id === id));
  }

  function addSeed(seed: Seed) {
    seeds.value.push(seed);
  }

  function removeSeed(id: string) {
    seeds.value = seeds.value.filter((s) => s.id !== id);
    selectedSeedIds.value = selectedSeedIds.value.filter((sid) => sid !== id);
  }

  function toggleSeed(id: string) {
    const idx = selectedSeedIds.value.indexOf(id);
    if (idx >= 0) {
      selectedSeedIds.value.splice(idx, 1);
    } else {
      selectedSeedIds.value.push(id);
    }
  }

  function selectAll() {
    selectedSeedIds.value = seeds.value.map((s) => s.id);
  }

  function deselectAll() {
    selectedSeedIds.value = [];
  }

  return {
    seeds,
    selectedSeedIds,
    selectedSeeds,
    seedCount,
    hasSelection,
    setSeeds,
    addSeed,
    removeSeed,
    toggleSeed,
    selectAll,
    deselectAll,
    strengthTier,
  };
});
