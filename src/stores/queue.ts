import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { VideoEntry } from '@/types/video';

export const useQueueStore = defineStore('queue', () => {
  const entries = ref<VideoEntry[]>([]);

  const entryCount = computed(() => entries.value.length);
  const validCount = computed(() => entries.value.filter((e) => e.status === 'valid').length);
  const invalidCount = computed(() => entries.value.filter((e) => e.status === 'invalid').length);

  /** Replace entire queue list. */
  function setEntries(list: VideoEntry[]) {
    entries.value = list;
  }

  /** Add a single entry (used for optimistic import UI update). */
  function addEntry(entry: VideoEntry) {
    entries.value.push(entry);
  }

  /** Remove a single entry by its position index (matching Rust remove_from_queue which takes usize index). */
  function removeEntry(index: number) {
    if (index >= 0 && index < entries.value.length) {
      entries.value.splice(index, 1);
    }
  }

  /** Remove all entries from the queue. */
  function clearQueue() {
    entries.value = [];
  }

  /** Persist reordered entries after drag-and-drop (D-14).
   *  newOrder is the full reordered array from VueDraggable v-model.
   *  Calls Rust backend to persist the new order to queue.json. */
  async function reorderEntries(newOrder: VideoEntry[]) {
    // Update order_index on each entry
    const indexed = newOrder.map((entry, i) => ({ ...entry, orderIndex: i }));
    entries.value = indexed;
    // Persist to Rust backend
    try {
      await invoke('reorder_queue', { entries: indexed });
    } catch (err) {
      console.error('Failed to persist queue reorder:', err);
    }
  }

  return {
    entries,
    entryCount,
    validCount,
    invalidCount,
    setEntries,
    addEntry,
    removeEntry,
    clearQueue,
    reorderEntries,
  };
});
