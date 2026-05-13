import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useBatchStore } from '@/stores/batch';
import type { BatchResult } from '@/types/batch';

describe('useBatchStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('initializes with zero progress, not processing, zero percent', () => {
    const store = useBatchStore();
    expect(store.progress.total).toBe(0);
    expect(store.progress.completed).toBe(0);
    expect(store.isProcessing).toBe(false);
    expect(store.overallPercent).toBe(0);
    expect(store.hasProgress).toBe(false);
    expect(store.isComplete).toBe(false);
    expect(store.lastResult).toBeNull();
  });

  it('startProcessing sets total, isProcessing=true, resets lastResult', () => {
    const store = useBatchStore();
    store.lastResult = { succeeded: [], failed: [] };
    store.startProcessing(10);
    expect(store.progress.total).toBe(10);
    expect(store.progress.completed).toBe(0);
    expect(store.isProcessing).toBe(true);
    expect(store.lastResult).toBeNull(); // reset
  });

  it('setProgress updates progress and derives isProcessing from completed < total', () => {
    const store = useBatchStore();
    store.startProcessing(5);
    store.setProgress({
      total: 5,
      completed: 3,
      succeeded: 3,
      failed: 0,
      currentFile: 'video.mp4',
    });
    expect(store.progress.completed).toBe(3);
    expect(store.isProcessing).toBe(true);
    expect(store.overallPercent).toBe(60);
  });

  it('setProgress sets isProcessing=false when completed >= total', () => {
    const store = useBatchStore();
    store.startProcessing(5);
    store.setProgress({ total: 5, completed: 5, succeeded: 5, failed: 0, currentFile: null });
    expect(store.isProcessing).toBe(false);
    expect(store.overallPercent).toBe(100);
  });

  it('stopProcessing sets isProcessing=false and stores result', () => {
    const store = useBatchStore();
    store.startProcessing(5);
    const result: BatchResult = { succeeded: ['a.mp4'], failed: [] };
    store.stopProcessing(result);
    expect(store.isProcessing).toBe(false);
    expect(store.lastResult).toEqual(result);
    expect(store.progress.currentFile).toBeNull();
  });

  it('resetBatch returns to idle state', () => {
    const store = useBatchStore();
    store.startProcessing(5);
    store.setProgress({ total: 5, completed: 5, succeeded: 5, failed: 0, currentFile: null });
    store.lastResult = { succeeded: ['a.mp4'], failed: [] };
    store.resetBatch();
    expect(store.progress.total).toBe(0);
    expect(store.isProcessing).toBe(false);
    expect(store.lastResult).toBeNull();
  });

  it('overallPercent returns 0 when total is 0 (division by zero safeguard)', () => {
    const store = useBatchStore();
    // With total=0 (initial state), overallPercent must be 0
    expect(store.overallPercent).toBe(0);
  });
});
