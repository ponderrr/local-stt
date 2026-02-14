import { useCallback, useEffect, useRef, useState } from "react";
import { commands, events } from "@/lib/tauri";

type DictationStatus = "idle" | "listening" | "loading" | "error";

export function useDictation() {
  const [isListening, setIsListening] = useState(false);
  const [status, setStatus] = useState<DictationStatus>("idle");
  const [error, setError] = useState<string | null>(null);
  const errorTimeoutRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    const unlisten = events.onDictationStatus((newStatus) => {
      setStatus(newStatus as DictationStatus);
      setIsListening(newStatus === "listening");
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const unlisten = events.onOutputError((msg) => {
      if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
      setError(msg);
      errorTimeoutRef.current = setTimeout(() => setError(null), 5000);
    });
    return () => {
      unlisten.then((fn) => fn());
      if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
    };
  }, []);

  useEffect(() => {
    const unlisten = events.onTranscriptionError((msg) => {
      if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
      setError(msg);
      errorTimeoutRef.current = setTimeout(() => setError(null), 5000);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const toggle = useCallback(async () => {
    try {
      const result = await commands.toggleDictation();
      setIsListening(result);
    } catch (err) {
      console.error("Failed to toggle dictation:", err);
      setStatus("error");
    }
  }, []);

  return { isListening, status, toggle, error };
}
