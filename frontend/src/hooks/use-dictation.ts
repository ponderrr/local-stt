import { useCallback, useEffect, useState } from "react";
import { commands, events } from "@/lib/tauri";

type DictationStatus = "idle" | "listening" | "loading" | "error";

export function useDictation() {
  const [isListening, setIsListening] = useState(false);
  const [status, setStatus] = useState<DictationStatus>("idle");

  useEffect(() => {
    const unlisten = events.onDictationStatus((newStatus) => {
      setStatus(newStatus as DictationStatus);
      setIsListening(newStatus === "listening");
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

  return { isListening, status, toggle };
}
