import type { ModelInfo } from '@/lib/tauri';

interface ModelSelectorProps {
  models: ModelInfo[];
  activeModel: string | null;
  onSelect: (modelId: string) => void;
}

export function ModelSelector({ models, activeModel, onSelect }: ModelSelectorProps) {
  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-4">
      <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground block text-center mb-3">
        Model
      </label>
      <select
        value={activeModel ?? ''}
        onChange={(e) => onSelect(e.target.value)}
        className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2.5 text-sm text-foreground focus:ring-1 focus:ring-primary focus:border-primary outline-none appearance-none cursor-pointer text-center"
      >
        <option value="" disabled>
          Select a model...
        </option>
        {models.map((model) => (
          <option key={model.id} value={model.id} disabled={!model.downloaded}>
            {model.display_name}
            {!model.downloaded ? ' (not downloaded)' : ''}
          </option>
        ))}
      </select>
    </div>
  );
}
