import { useCallback, useEffect, useState } from 'react';
import { commands, type Config } from '@/lib/tauri';

export function useConfig() {
  const [config, setConfig] = useState<Config | null>(null);

  useEffect(() => {
    commands.getConfig().then(setConfig).catch(console.error);
  }, []);

  const updateConfig = useCallback(async (newConfig: Config) => {
    try {
      await commands.updateConfig(newConfig);
      setConfig(newConfig);
    } catch (err) {
      console.error('Failed to update config:', err);
    }
  }, []);

  return { config, updateConfig };
}
