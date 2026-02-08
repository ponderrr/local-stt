import { useCallback, useEffect, useState } from "react";
import { commands, events, type ModelInfo } from "@/lib/tauri";

export function useModels() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [activeModel, setActiveModel] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<Record<string, number>>({});

  const refresh = useCallback(async () => {
    try {
      const modelList = await commands.listModels();
      setModels(modelList);
      const active = await commands.getActiveModel();
      setActiveModel(active);
    } catch (err) {
      console.error("Failed to fetch models:", err);
    }
  }, []);

  useEffect(() => {
    refresh();

    const unlisten = events.onDownloadProgress((data) => {
      setDownloadProgress((prev) => ({
        ...prev,
        [data.model_id]: data.percent,
      }));

      if (data.percent >= 100) {
        refresh();
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [refresh]);

  const loadModel = useCallback(async (modelId: string) => {
    try {
      await commands.loadModel(modelId);
      setActiveModel(modelId);
    } catch (err) {
      console.error("Failed to load model:", err);
    }
  }, []);

  const downloadModel = useCallback(async (modelId: string) => {
    try {
      await commands.downloadModel(modelId);
      await refresh();
    } catch (err) {
      console.error("Failed to download model:", err);
    }
  }, [refresh]);

  const deleteModel = useCallback(async (modelId: string) => {
    try {
      await commands.deleteModel(modelId);
      await refresh();
    } catch (err) {
      console.error("Failed to delete model:", err);
    }
  }, [refresh]);

  return { models, activeModel, loadModel, downloadModel, deleteModel, downloadProgress, refresh };
}
