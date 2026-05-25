import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSeedStore } from '@/stores/seed';
import type { Seed } from '@/types/seed';

describe('useSeedStore', () => {
  beforeEach(() => {
    // Fresh Pinia instance for each test to avoid state leakage
    setActivePinia(createPinia());
  });

  const mockSeed = (id: string, alias: string): Seed => ({
    id,
    alias,
    operations: [],
    createdAt: new Date().toISOString(),
  });

  it('initializes with empty seeds, empty selection, zero count', () => {
    const store = useSeedStore();
    expect(store.seeds).toEqual([]);
    expect(store.selectedSeedIds).toEqual([]);
    expect(store.selectedSeeds).toEqual([]);
    expect(store.seedCount).toBe(0);
    expect(store.hasSelection).toBe(false);
  });

  it('setSeeds replaces the seed list and clears stale selection', () => {
    const store = useSeedStore();
    store.toggleSeed('gone-id');
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    expect(store.seeds).toHaveLength(2);
    expect(store.seedCount).toBe(2);
    // Stale selection cleared (gone-id no longer in list)
    expect(store.selectedSeedIds).toEqual([]);
  });

  it('toggleSeed adds and removes seed from selection', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    store.toggleSeed('b');
    expect(store.selectedSeedIds).toEqual(['b']);
    expect(store.selectedSeeds).toHaveLength(1);
    expect(store.selectedSeeds[0].alias).toBe('Beta');
    // Toggle again removes
    store.toggleSeed('b');
    expect(store.selectedSeedIds).toEqual([]);
  });

  it('deselectAll clears multi-selection', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    store.toggleSeed('a');
    store.toggleSeed('b');
    expect(store.selectedSeedIds).toHaveLength(2);
    store.deselectAll();
    expect(store.selectedSeedIds).toEqual([]);
    expect(store.selectedSeeds).toEqual([]);
  });

  it('selectAll selects every seed', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    store.selectAll();
    expect(store.selectedSeedIds).toEqual(['a', 'b']);
    expect(store.hasSelection).toBe(true);
  });

  it('removeSeed removes the seed and clears it from selection', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    store.toggleSeed('b');
    store.removeSeed('b');
    expect(store.seeds).toHaveLength(1);
    expect(store.seeds[0].id).toBe('a');
    // Selection cleared because removed seed was selected
    expect(store.selectedSeedIds).toEqual([]);
  });

  it('addSeed appends a seed to the list', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha')]);
    store.addSeed(mockSeed('b', 'Beta'));
    expect(store.seeds).toHaveLength(2);
    expect(store.seeds[1].alias).toBe('Beta');
  });
});
