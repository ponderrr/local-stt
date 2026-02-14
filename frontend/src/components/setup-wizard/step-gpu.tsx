import { useEffect, useState } from 'react';
import { commands, type GpuInfo } from '@/lib/tauri';

interface StepGpuProps {
  onNext: () => void;
}

export function StepGpu({ onNext }: StepGpuProps) {
  const [gpu, setGpu] = useState<GpuInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    commands
      .getGpuInfo()
      .then((info) => setGpu(info))
      .catch(() => setGpu(null))
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 text-center space-y-6">
      <h2 className="text-lg font-semibold text-foreground">Welcome to WhisperType</h2>
      <p className="text-sm text-muted-foreground">
        Local AI-powered speech-to-text. Everything runs on your machine.
      </p>

      <div className="bg-[#0f0f11] border border-white/[0.06] rounded-lg p-4 space-y-2">
        <h3 className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
          Detected Hardware
        </h3>
        {loading ? (
          <p className="text-sm text-zinc-500">Detecting...</p>
        ) : gpu?.cuda_available ? (
          <>
            <p className="text-sm text-foreground font-medium">{gpu.name}</p>
            <p className="text-xs text-muted-foreground">
              {gpu.vram_total_mb.toLocaleString()} MB VRAM - CUDA Available
            </p>
            <div className="inline-block bg-emerald-500/10 text-emerald-400 text-xs px-2 py-1 rounded mt-1">
              GPU Acceleration Ready
            </div>
          </>
        ) : (
          <>
            <p className="text-sm text-foreground">No NVIDIA GPU detected</p>
            <div className="inline-block bg-amber-500/10 text-amber-400 text-xs px-2 py-1 rounded mt-1">
              CPU Mode (slower transcription)
            </div>
          </>
        )}
      </div>

      <button
        onClick={onNext}
        className="bg-primary hover:bg-primary/90 text-primary-foreground px-6 py-2.5 rounded-md text-sm font-medium transition-colors"
      >
        Continue
      </button>
    </div>
  );
}
