import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
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

  return {
    entries,
    entryCount,
    validCount,
    invalidCount,
    setEntries,
    addEntry,
    removeEntry,
    clearQueue,
  };
});
