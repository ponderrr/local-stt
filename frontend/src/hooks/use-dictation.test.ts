import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";

// Track listeners so we can simulate events
const listeners: Record<string, ((event: { payload: unknown }) => void)[]> = {};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
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

import { invoke } from "@tauri-apps/api/core";
import { useDictation } from "@/hooks/use-dictation";

const mockedInvoke = vi.mocked(invoke);

function emitEvent(event: string, payload: unknown) {
  if (listeners[event]) {
    for (const handler of listeners[event]) {
      handler({ payload });
    }
  }
}

describe("useDictation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
    for (const key of Object.keys(listeners)) {
      delete listeners[key];
    }
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("starts with idle status and no error", () => {
    const { result } = renderHook(() => useDictation());
    expect(result.current.status).toBe("idle");
    expect(result.current.error).toBeNull();
  });

  it("updates status on dictation-status event", async () => {
    const { result } = renderHook(() => useDictation());

    // Wait for effects to run
    await act(async () => {});

    act(() => {
      emitEvent("dictation-status", "listening");
    });
    expect(result.current.status).toBe("listening");

    act(() => {
      emitEvent("dictation-status", "idle");
    });
    expect(result.current.status).toBe("idle");
  });

  it("toggle calls toggleDictation command", async () => {
    mockedInvoke.mockResolvedValue(true);
    const { result } = renderHook(() => useDictation());

    await act(async () => {
      await result.current.toggle();
    });

    expect(mockedInvoke).toHaveBeenCalledWith("toggle_dictation");
  });

  it("toggle sets error status on failure", async () => {
    mockedInvoke.mockRejectedValue(new Error("failed"));
    const { result } = renderHook(() => useDictation());

    await act(async () => {
      await result.current.toggle();
    });

    expect(result.current.status).toBe("error");
  });

  it("sets error on output-error event", async () => {
    const { result } = renderHook(() => useDictation());
    await act(async () => {});

    act(() => {
      emitEvent("output-error", "Failed to type text");
    });
    expect(result.current.error).toBe("Failed to type text");
  });

  it("sets error on transcription-error event", async () => {
    const { result } = renderHook(() => useDictation());
    await act(async () => {});

    act(() => {
      emitEvent("transcription-error", "Transcription failed: GPU OOM");
    });
    expect(result.current.error).toBe("Transcription failed: GPU OOM");
  });

  it("auto-clears error after 5 seconds", async () => {
    const { result } = renderHook(() => useDictation());
    await act(async () => {});

    act(() => {
      emitEvent("output-error", "Test error");
    });
    expect(result.current.error).toBe("Test error");

    act(() => {
      vi.advanceTimersByTime(5000);
    });
    expect(result.current.error).toBeNull();
  });

  it("resets error timeout on new error", async () => {
    const { result } = renderHook(() => useDictation());
    await act(async () => {});

    act(() => {
      emitEvent("output-error", "First error");
    });

    // Advance 3 seconds (not enough to clear)
    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(result.current.error).toBe("First error");

    // New error resets the timer
    act(() => {
      emitEvent("output-error", "Second error");
    });
    expect(result.current.error).toBe("Second error");

    // 3 more seconds (6 total from first, 3 from second)
    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(result.current.error).toBe("Second error");

    // 2 more seconds (5 total from second)
    act(() => {
      vi.advanceTimersByTime(2000);
    });
    expect(result.current.error).toBeNull();
  });
});
