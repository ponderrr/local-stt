import { useEffect, useState } from 'react';
import { useConfig } from '@/hooks/use-config';
import { commands, type Config } from '@/lib/tauri';

interface SettingsPanelProps {
  onClose: () => void;
}

export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const { config, updateConfig } = useConfig();
  const [audioDevices, setAudioDevices] = useState<string[]>([]);
  const [localConfig, setLocalConfig] = useState<Config | null>(null);

  useEffect(() => {
    if (config) setLocalConfig({ ...config });
    commands.listAudioDevices().then(setAudioDevices).catch(console.error);
  }, [config]);

  const handleSave = async () => {
    if (localConfig) {
      await updateConfig(localConfig);
      onClose();
    }
  };

  if (!localConfig) return null;

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-[#131316] border border-white/[0.08] rounded-lg w-full max-w-md p-6 space-y-5">
        <h2 className="text-sm font-medium uppercase tracking-widest text-muted-foreground text-center">
          Settings
        </h2>

        {/* Output Mode */}
        <div>
          <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground block mb-2">
            Output Mode
          </label>
          <select
            value={localConfig.output_mode}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                output_mode: e.target.value as Config['output_mode'],
              })
            }
            className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2 text-sm text-foreground outline-none"
          >
            <option value="both" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              Type + Clipboard
            </option>
            <option
              value="type_into_field"
              style={{ backgroundColor: '#18181b', color: '#fafafa' }}
            >
              Type into Field
            </option>
            <option value="clipboard" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              Clipboard Only
            </option>
          </select>
        </div>

        {/* Audio Device */}
        <div>
          <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground block mb-2">
            Microphone
          </label>
          <select
            value={localConfig.audio_device ?? ''}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                audio_device: e.target.value || null,
              })
            }
            className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2 text-sm text-foreground outline-none"
          >
            <option value="" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              System Default
            </option>
            {audioDevices.map((device) => (
              <option
                key={device}
                value={device}
                style={{ backgroundColor: '#18181b', color: '#fafafa' }}
              >
                {device}
              </option>
            ))}
          </select>
        </div>

        {/* Language */}
        <div>
          <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground block mb-2">
            Language
          </label>
          <select
            value={localConfig.language}
            onChange={(e) => setLocalConfig({ ...localConfig, language: e.target.value })}
            className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2 text-sm text-foreground outline-none"
          >
            <option value="auto" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              Auto Detect
            </option>
            <option value="en" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              English
            </option>
            <option value="es" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              Spanish
            </option>
            <option value="fr" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              French
            </option>
            <option value="de" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              German
            </option>
            <option value="ja" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              Japanese
            </option>
            <option value="zh" style={{ backgroundColor: '#18181b', color: '#fafafa' }}>
              Chinese
            </option>
          </select>
        </div>

        {/* Hotkey Display */}
        <div>
          <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground block mb-2">
            Hotkey
          </label>
          <div className="bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2 text-sm text-foreground/60 font-mono">
            {localConfig.hotkey}
          </div>
        </div>

        {/* Actions */}
        <div className="flex items-center justify-end gap-3 pt-2">
          <button
            onClick={onClose}
            className="hover:bg-white/[0.05] text-muted-foreground px-4 py-2 rounded-md text-sm transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="bg-primary hover:bg-primary/90 text-primary-foreground px-4 py-2 rounded-md text-sm font-medium transition-colors"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
