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

  it('initializes with empty seeds, null selection, zero count', () => {
    const store = useSeedStore();
    expect(store.seeds).toEqual([]);
    expect(store.selectedSeedId).toBeNull();
    expect(store.selectedSeed).toBeNull();
    expect(store.seedCount).toBe(0);
  });

  it('setSeeds replaces the seed list and clears stale selection', () => {
    const store = useSeedStore();
    store.selectSeed('gone-id');
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    expect(store.seeds).toHaveLength(2);
    expect(store.seedCount).toBe(2);
    // Stale selection cleared
    expect(store.selectedSeedId).toBeNull();
  });

  it('selectSeed sets id and selectedSeed computed returns correct seed', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    store.selectSeed('b');
    expect(store.selectedSeedId).toBe('b');
    expect(store.selectedSeed?.alias).toBe('Beta');
  });

  it('selectSeed(null) deselects', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha')]);
    store.selectSeed('a');
    store.selectSeed(null);
    expect(store.selectedSeedId).toBeNull();
    expect(store.selectedSeed).toBeNull();
  });

  it('removeSeed removes the seed and clears selection if it was selected', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha'), mockSeed('b', 'Beta')]);
    store.selectSeed('b');
    store.removeSeed('b');
    expect(store.seeds).toHaveLength(1);
    expect(store.seeds[0].id).toBe('a');
    // Selection cleared because removed seed was selected
    expect(store.selectedSeedId).toBeNull();
  });

  it('addSeed appends a seed to the list', () => {
    const store = useSeedStore();
    store.setSeeds([mockSeed('a', 'Alpha')]);
    store.addSeed(mockSeed('b', 'Beta'));
    expect(store.seeds).toHaveLength(2);
    expect(store.seeds[1].alias).toBe('Beta');
  });
});
