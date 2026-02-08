import { useCallback, useEffect, useState } from "react";
import { events } from "@/lib/tauri";

export function useTranscription() {
  const [transcript, setTranscript] = useState("");

  useEffect(() => {
    const unlisten = events.onTranscription((data) => {
      setTranscript((prev) => {
        const separator = prev && !prev.endsWith(" ") ? " " : "";
        return prev + separator + data.text;
      });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const clear = useCallback(() => {
    setTranscript("");
  }, []);

  return { transcript, clear };
}
