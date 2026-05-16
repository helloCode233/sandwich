import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ProcessingLogEntry } from '@/types/log';

export const useLogStore = defineStore('log', () => {
  const entries = ref<ProcessingLogEntry[]>([]);
  const searchQuery = ref('');
  const statusFilter = ref<'all' | 'success' | 'failure'>('all');

  /** Total log entry count. */
  const entryCount = computed(() => entries.value.length);

  /** Successfully processed entries. */
  const successCount = computed(() => entries.value.filter((e) => e.status === 'success').length);

  /** Failed entries. */
  const failureCount = computed(() => entries.value.filter((e) => e.status === 'failure').length);

  /** Filtered and searchable entries for the log panel UI. */
  const filteredEntries = computed(() => {
    let result = entries.value;
    // Status filter
    if (statusFilter.value !== 'all') {
      result = result.filter((e) => e.status === statusFilter.value);
    }
    // Text search across filename and seed alias
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(
        (e) => e.file.toLowerCase().includes(q) || e.seedAlias.toLowerCase().includes(q),
      );
    }
    return result;
  });

  /** Add a single log entry (called from batch-log event listener).
   *  Prepends to list (newest first). Caps at 500 entries to prevent unbounded growth. */
  function addEntry(entry: ProcessingLogEntry) {
    entries.value.unshift(entry);
    if (entries.value.length > 500) {
      entries.value = entries.value.slice(0, 500);
    }
  }

  /** Replace entire log (used when loading persisted logs on startup). */
  function setEntries(list: ProcessingLogEntry[]) {
    entries.value = list;
  }

  /** Clear all log entries. */
  function clearEntries() {
    entries.value = [];
  }

  return {
    entries,
    searchQuery,
    statusFilter,
    entryCount,
    successCount,
    failureCount,
    filteredEntries,
    addEntry,
    setEntries,
    clearEntries,
  };
});
