import { useEffect, useState } from 'react';
import { commands, type ModelInfo } from '@/lib/tauri';

interface StepModelsProps {
  selected: string[];
  onSelect: (ids: string[]) => void;
  onNext: () => void;
}

export function StepModels({ selected, onSelect, onNext }: StepModelsProps) {
  const [models, setModels] = useState<ModelInfo[]>([]);

  useEffect(() => {
    commands.listModels().then(setModels).catch(console.error);
  }, []);

  const toggleModel = (id: string) => {
    if (selected.includes(id)) {
      onSelect(selected.filter((m) => m !== id));
    } else {
      onSelect([...selected, id]);
    }
  };

  const totalSize = models
    .filter((m) => selected.includes(m.id))
    .reduce((acc, m) => acc + m.size_bytes, 0);

  const formatSize = (bytes: number) => {
    if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
    return `${(bytes / 1e6).toFixed(0)} MB`;
  };

  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 space-y-6">
      <div className="text-center">
        <h2 className="text-lg font-semibold text-foreground">Choose Models</h2>
        <p className="text-sm text-muted-foreground mt-1">
          Select which Whisper models to download. You can add more later.
        </p>
      </div>

      <div className="space-y-2">
        {models.map((model) => (
          <button
            key={model.id}
            onClick={() => toggleModel(model.id)}
            className={`w-full flex items-center justify-between p-3 rounded-lg border transition-colors ${
              selected.includes(model.id)
                ? 'border-primary/50 bg-primary/5'
                : 'border-white/[0.08] hover:bg-white/[0.03]'
            }`}
          >
            <div className="text-left">
              <p className="text-sm font-medium text-foreground">{model.display_name}</p>
              <p className="text-xs text-muted-foreground">
                ~{model.vram_mb.toLocaleString()} MB VRAM
              </p>
            </div>
            <div
              className={`w-4 h-4 rounded border-2 flex items-center justify-center transition-colors ${
                selected.includes(model.id) ? 'border-primary bg-primary' : 'border-zinc-600'
              }`}
            >
              {selected.includes(model.id) && (
                <svg className="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </div>
          </button>
        ))}
      </div>

      <div className="text-center text-xs text-muted-foreground">
        Total download: {formatSize(totalSize)}
      </div>

      <button
        onClick={onNext}
        disabled={selected.length === 0}
        className="w-full bg-primary hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-primary-foreground px-6 py-2.5 rounded-md text-sm font-medium transition-colors"
      >
        Download {selected.length} Model{selected.length !== 1 ? 's' : ''}
      </button>
    </div>
  );
}
