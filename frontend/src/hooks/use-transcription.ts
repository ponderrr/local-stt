import { useCallback, useEffect, useState } from 'react';
import { events } from '@/lib/tauri';

export function useTranscription() {
  const [committed, setCommitted] = useState('');
  const [partial, setPartial] = useState('');

  useEffect(() => {
    const unlisten = events.onTranscription((data) => {
      if (data.is_partial) {
        setPartial(data.text);
      } else {
        setCommitted((prev) => {
          const separator = prev && !prev.endsWith(' ') ? ' ' : '';
          return prev + separator + data.text;
        });
        setPartial('');
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const transcript = committed + (partial ? (committed ? ' ' : '') + partial : '');

  const clear = useCallback(() => {
    setCommitted('');
    setPartial('');
  }, []);

  return { transcript, committed, partial, clear };
}
