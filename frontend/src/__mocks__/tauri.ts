import { vi } from 'vitest';

// Mock Tauri invoke
export const invoke = vi.fn();

// Mock event listeners - each returns a Promise<UnlistenFn>
const eventListeners: Record<string, Array<(payload: unknown) => void>> = {};

export const listen = vi.fn((event: string, handler: (event: { payload: unknown }) => void) => {
  if (!eventListeners[event]) {
    eventListeners[event] = [];
  }
  const wrappedHandler = (payload: unknown) => handler({ payload });
  eventListeners[event].push(wrappedHandler);

  const unlisten = () => {
    const idx = eventListeners[event].indexOf(wrappedHandler);
    if (idx >= 0) eventListeners[event].splice(idx, 1);
  };
  return Promise.resolve(unlisten);
});

// Helper: emit a mock event to all registered listeners
export function emitMockEvent(event: string, payload: unknown) {
  if (eventListeners[event]) {
    for (const handler of eventListeners[event]) {
      handler(payload);
    }
  }
}

// Helper: clear all listeners between tests
export function clearMockListeners() {
  for (const key of Object.keys(eventListeners)) {
    delete eventListeners[key];
  }
}
