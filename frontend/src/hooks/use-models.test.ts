import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';

const listeners: Record<string, ((event: { payload: unknown }) => void)[]> = {};

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, handler: (event: { payload: unknown }) => void) => {
    if (!listeners[event]) listeners[event] = [];
    listeners[event].push(handler);
    const unlisten = () => {
      const idx = listeners[event].indexOf(handler);
      if (idx >= 0) listeners[event].splice(idx, 1);
    };
    return Promise.resolve(unlisten);
  }),
}));

import { invoke } from '@tauri-apps/api/core';
import { useModels } from '@/hooks/use-models';

const mockedInvoke = vi.mocked(invoke);

const mockModels = [
  {
    id: 'tiny',
    display_name: 'Tiny (~75 MB)',
    filename: 'ggml-tiny.bin',
    size_bytes: 77691713,
    vram_mb: 1000,
    downloaded: true,
  },
  {
    id: 'base',
    display_name: 'Base (~150 MB)',
    filename: 'ggml-base.bin',
    size_bytes: 147951465,
    vram_mb: 1000,
    downloaded: false,
  },
];

describe('useModels', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    for (const key of Object.keys(listeners)) {
      delete listeners[key];
    }
    // Default mock: listModels returns models, getActiveModel returns null
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'list_models') return mockModels;
      if (cmd === 'get_active_model') return null;
      return undefined;
    });
  });

  it('starts in loading state', () => {
    const { result } = renderHook(() => useModels());
    expect(result.current.loading).toBe(true);
    expect(result.current.models).toEqual([]);
  });

  it('loads models on mount', async () => {
    const { result } = renderHook(() => useModels());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.models).toEqual(mockModels);
    expect(result.current.activeModel).toBeNull();
  });

  it('loadModel calls invoke and updates activeModel', async () => {
    const { result } = renderHook(() => useModels());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.loadModel('tiny');
    });

    expect(mockedInvoke).toHaveBeenCalledWith('load_model', { modelId: 'tiny' });
    expect(result.current.activeModel).toBe('tiny');
  });

  it('downloadModel calls invoke and refreshes', async () => {
    const { result } = renderHook(() => useModels());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.downloadModel('base');
    });

    expect(mockedInvoke).toHaveBeenCalledWith('download_model', { modelId: 'base' });
  });

  it('deleteModel calls invoke and refreshes', async () => {
    const { result } = renderHook(() => useModels());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.deleteModel('tiny');
    });

    expect(mockedInvoke).toHaveBeenCalledWith('delete_model', { modelId: 'tiny' });
  });

  it('handles loadModel failure gracefully', async () => {
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === 'list_models') return mockModels;
      if (cmd === 'get_active_model') return null;
      if (cmd === 'load_model') throw new Error('Model load failed');
      return undefined;
    });

    const { result } = renderHook(() => useModels());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // Should not throw
    await act(async () => {
      await result.current.loadModel('tiny');
    });

    // activeModel should NOT be updated on failure
    expect(result.current.activeModel).toBeNull();
  });

  it('handles listModels failure gracefully', async () => {
    mockedInvoke.mockRejectedValue(new Error('Backend unavailable'));

    const { result } = renderHook(() => useModels());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // Should still finish loading without crash
    expect(result.current.models).toEqual([]);
  });
});
