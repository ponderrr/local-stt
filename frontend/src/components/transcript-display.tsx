import { useEffect, useRef } from "react";

interface TranscriptDisplayProps {
  transcript: string;
}

export function TranscriptDisplay({ transcript }: TranscriptDisplayProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [transcript]);

  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-1 h-full flex flex-col">
      <h3 className="text-xs font-medium uppercase tracking-wider text-muted-foreground text-center py-3">
        Transcript
      </h3>
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto px-4 pb-4 font-mono text-sm text-foreground/90 leading-relaxed"
      >
        {transcript ? (
          <p className="whitespace-pre-wrap">{transcript}</p>
        ) : (
          <p className="text-zinc-600 text-center italic mt-8">
            Press hotkey or click to start dictating...
          </p>
        )}
      </div>
    </div>
  );
}
