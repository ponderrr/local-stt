import { useEffect, useState } from "react";
import { TranscriptDisplay } from "@/components/transcript-display";
import { ModelSelector } from "@/components/model-selector";
import { StatusIndicator } from "@/components/status-indicator";
import { SettingsPanel } from "@/components/settings-panel";
import { useDictation } from "@/hooks/use-dictation";
import { useTranscription } from "@/hooks/use-transcription";
import { useModels } from "@/hooks/use-models";
import { useConfig } from "@/hooks/use-config";

export function MainWindow() {
  const [showSettings, setShowSettings] = useState(false);
  const { status, error } = useDictation();
  const { transcript, clear } = useTranscription();
  const { models, activeModel, loadModel, loading } = useModels();
  const { config } = useConfig();

  useEffect(() => {
    if (loading || !config || activeModel) return;
    const defaultModel = config.default_model;
    const isDownloaded = models.find(
      (m) => m.id === defaultModel && m.downloaded
    );
    if (isDownloaded) {
      loadModel(defaultModel);
    }
  }, [loading, config, models, activeModel, loadModel]);

  return (
    <div className="h-screen flex flex-col bg-[#0f0f11] text-foreground p-6">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-sm font-medium uppercase tracking-widest text-muted-foreground">
          WhisperType
        </h1>
        <button
          onClick={() => setShowSettings(!showSettings)}
          className="hover:bg-white/[0.05] p-2 rounded-md transition-colors"
          title="Settings"
        >
          <svg
            className="w-4 h-4 text-muted-foreground"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
            />
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
        </button>
      </div>

      {/* Model Selector */}
      <div className="mb-4">
        <ModelSelector
          models={models}
          activeModel={activeModel}
          onSelect={loadModel}
        />
      </div>

      {/* Transcript Display */}
      <div className="flex-1 mb-4 min-h-0">
        <TranscriptDisplay transcript={transcript} />
      </div>

      {/* Status + Actions */}
      <div className="flex items-center justify-between">
        <StatusIndicator status={status} hotkey={config?.hotkey ?? "Ctrl+Shift+Space"} />
        {error && (
          <div className="mt-2 text-xs text-red-400 bg-red-400/10 border border-red-400/20 rounded px-3 py-2">
            {error}
          </div>
        )}

        <div className="flex items-center gap-2">
          <button
            onClick={() => {
              if (transcript) {
                navigator.clipboard.writeText(transcript);
              }
            }}
            className="hover:bg-white/[0.05] text-muted-foreground hover:text-foreground px-3 py-2 rounded-md text-xs uppercase tracking-wide transition-colors"
          >
            Copy
          </button>
          <button
            onClick={clear}
            className="hover:bg-white/[0.05] text-muted-foreground hover:text-foreground px-3 py-2 rounded-md text-xs uppercase tracking-wide transition-colors"
          >
            Clear
          </button>
        </div>
      </div>

      {/* Settings Panel (overlay) */}
      {showSettings && (
        <SettingsPanel onClose={() => setShowSettings(false)} />
      )}
    </div>
  );
}
