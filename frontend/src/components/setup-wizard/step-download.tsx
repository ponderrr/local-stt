import { useEffect, useState } from "react";
import { commands, events } from "@/lib/tauri";

interface StepDownloadProps {
  models: string[];
  onNext: () => void;
}

export function StepDownload({ models, onNext }: StepDownloadProps) {
  const [progress, setProgress] = useState<Record<string, number>>({});
  const [currentModel, setCurrentModel] = useState<string | null>(null);
  const [completed, setCompleted] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = events.onDownloadProgress((data) => {
      setProgress((prev) => ({ ...prev, [data.model_id]: data.percent }));
    });

    const downloadAll = async () => {
      for (const modelId of models) {
        setCurrentModel(modelId);
        try {
          await commands.downloadModel(modelId);
          setCompleted((prev) => [...prev, modelId]);
        } catch (err) {
          setError(`Failed to download ${modelId}: ${err}`);
          return;
        }
      }
      setCurrentModel(null);
    };

    downloadAll();

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [models]);

  const allDone = completed.length === models.length;

  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 space-y-6">
      <div className="text-center">
        <h2 className="text-lg font-semibold text-foreground">
          {allDone ? "Downloads Complete" : "Downloading Models"}
        </h2>
        <p className="text-sm text-muted-foreground mt-1">
          {allDone
            ? "All models are ready to use."
            : "This may take a few minutes depending on your connection."}
        </p>
      </div>

      <div className="space-y-4">
        {models.map((modelId) => {
          const pct = progress[modelId] ?? 0;
          const done = completed.includes(modelId);
          const active = currentModel === modelId;

          return (
            <div key={modelId} className="space-y-1.5">
              <div className="flex items-center justify-between text-sm">
                <span className={done ? "text-foreground" : "text-muted-foreground"}>
                  {modelId}
                </span>
                <span className="text-xs text-muted-foreground">
                  {done ? "Done" : active ? `${pct.toFixed(0)}%` : "Waiting..."}
                </span>
              </div>
              <div className="w-full h-1.5 bg-white/[0.05] rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full transition-all duration-300 ${
                    done ? "bg-emerald-500" : "bg-primary"
                  }`}
                  style={{ width: `${done ? 100 : pct}%` }}
                />
              </div>
            </div>
          );
        })}
      </div>

      {error && (
        <div className="bg-red-500/10 text-red-400 text-sm p-3 rounded-lg">
          {error}
        </div>
      )}

      <button
        onClick={onNext}
        disabled={!allDone}
        className="w-full bg-primary hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-primary-foreground px-6 py-2.5 rounded-md text-sm font-medium transition-colors"
      >
        {allDone ? "Continue" : "Downloading..."}
      </button>
    </div>
  );
}
