import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useSeedStore } from '@/stores/seed';
import type { Seed } from '@/types/seed';

let unlisten: UnlistenFn | null = null;

export function useSeed() {
  const store = useSeedStore();

  /** Fetch the authoritative seed list from Rust and replace store state. */
  async function loadSeeds(): Promise<void> {
    try {
      const list = await invoke<Seed[]>('list_seeds');
      store.setSeeds(list);
    } catch (err) {
      console.error('Failed to load seeds:', err);
    }
  }

  /** Subscribe to seeds-updated event (invalidation signal) and perform initial load. */
  async function subscribe(): Promise<void> {
    unlisten = await listen('seeds-updated', () => {
      loadSeeds();
    });
    await loadSeeds();
  }

  /** Generate a new random seed. Per D-01: pure random, no user params. */
  async function generateSeed(): Promise<Seed | null> {
    try {
      const seed = await invoke<Seed>('generate_seed');
      store.addSeed(seed);
      return seed;
    } catch (err) {
      console.error('Failed to generate seed:', err);
      return null;
    }
  }

  /** Rename a seed's alias. Returns true on success, false on failure. */
  async function renameSeed(seedId: string, newAlias: string): Promise<boolean> {
    try {
      await invoke('rename_seed', { seedId, newAlias });
      return true;
    } catch (err) {
      console.error('Failed to rename seed:', err);
      return false;
    }
  }

  /** Delete a seed by ID. Updates store optimistically on success. */
  async function deleteSeed(seedId: string): Promise<boolean> {
    try {
      await invoke('delete_seed', { seedId });
      store.removeSeed(seedId);
      return true;
    } catch (err) {
      console.error('Failed to delete seed:', err);
      return false;
    }
  }

  /** Copy a seed with re-randomized parameters. Returns the new seed or null. */
  async function copySeed(seedId: string): Promise<Seed | null> {
    try {
      const seed = await invoke<Seed>('copy_seed', { seedId });
      store.addSeed(seed);
      return seed;
    } catch (err) {
      console.error('Failed to copy seed:', err);
      return null;
    }
  }

  function unsubscribe(): void {
    unlisten?.();
  }

  return { loadSeeds, subscribe, generateSeed, renameSeed, deleteSeed, copySeed, unsubscribe };
}
