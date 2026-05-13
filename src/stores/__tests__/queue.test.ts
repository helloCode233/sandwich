import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useQueueStore } from '@/stores/queue';
import type { VideoEntry } from '@/types/video';

const mockEntry = (filepath: string, status: 'valid' | 'invalid' = 'valid'): VideoEntry => ({
  filename: filepath.split('/').pop() || 'video.mp4',
  filepath,
  metadata: {
    durationSecs: 120,
    width: 1920,
    height: 1080,
    sizeBytes: 50000000,
    codec: 'h264',
    fps: 29.97,
  },
  status,
});

describe('useQueueStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('initializes with empty entries, all counts zero', () => {
    const store = useQueueStore();
    expect(store.entries).toEqual([]);
    expect(store.entryCount).toBe(0);
    expect(store.validCount).toBe(0);
    expect(store.invalidCount).toBe(0);
  });

  it('addEntry appends an entry and updates counts', () => {
    const store = useQueueStore();
    store.addEntry(mockEntry('/videos/a.mp4', 'valid'));
    expect(store.entryCount).toBe(1);
    expect(store.validCount).toBe(1);
    expect(store.invalidCount).toBe(0);
  });

  it('validCount and invalidCount are computed separately', () => {
    const store = useQueueStore();
    store.setEntries([
      mockEntry('/videos/a.mp4', 'valid'),
      mockEntry('/videos/b.mp4', 'invalid'),
      mockEntry('/videos/c.mp4', 'valid'),
    ]);
    expect(store.entryCount).toBe(3);
    expect(store.validCount).toBe(2);
    expect(store.invalidCount).toBe(1);
  });

  it('removeEntry splices by index', () => {
    const store = useQueueStore();
    store.setEntries([
      mockEntry('/videos/a.mp4'),
      mockEntry('/videos/b.mp4'),
      mockEntry('/videos/c.mp4'),
    ]);
    store.removeEntry(1);
    expect(store.entries).toHaveLength(2);
    expect(store.entries[1].filepath).toBe('/videos/c.mp4');
  });

  it('removeEntry handles out-of-bounds index gracefully', () => {
    const store = useQueueStore();
    store.setEntries([mockEntry('/videos/a.mp4')]);
    store.removeEntry(99); // out of bounds
    expect(store.entries).toHaveLength(1); // unchanged
  });

  it('clearQueue empties all entries', () => {
    const store = useQueueStore();
    store.setEntries([mockEntry('/videos/a.mp4'), mockEntry('/videos/b.mp4')]);
    store.clearQueue();
    expect(store.entries).toEqual([]);
    expect(store.entryCount).toBe(0);
  });
});
