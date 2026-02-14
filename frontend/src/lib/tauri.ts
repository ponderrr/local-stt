import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// Types
export interface Config {
  version: number;
  hotkey: string;
  default_model: string;
  output_mode: "type_into_field" | "clipboard" | "both";
  audio_device: string | null;
  language: string;
  vad_threshold: number;
  chunk_duration_ms: number;
  overlap_ms: number;
  downloaded_models: string[];
  first_run_complete: boolean;
}

export interface ModelInfo {
  id: string;
  display_name: string;
  filename: string;
  size_bytes: number;
  vram_mb: number;
  downloaded: boolean;
}

export interface GpuInfo {
  name: string;
  vram_total_mb: number;
  cuda_available: boolean;
}

export interface TranscriptionUpdate {
  text: string;
  is_partial: boolean;
}

export interface DownloadProgress {
  model_id: string;
  percent: number;
  downloaded_bytes: number;
  total_bytes: number;
}

// Commands
export const commands = {
  toggleDictation: () => invoke<boolean>("toggle_dictation"),
  startDictation: () => invoke<void>("start_dictation"),
  stopDictation: () => invoke<void>("stop_dictation"),
  listModels: () => invoke<ModelInfo[]>("list_models"),
  downloadModel: (modelId: string) => invoke<void>("download_model", { modelId }),
  deleteModel: (modelId: string) => invoke<void>("delete_model", { modelId }),
  loadModel: (modelId: string) => invoke<void>("load_model", { modelId }),
  getActiveModel: () => invoke<string | null>("get_active_model"),
  getConfig: () => invoke<Config>("get_config"),
  updateConfig: (config: Config) => invoke<void>("update_config", { config }),
  listAudioDevices: () => invoke<string[]>("list_audio_devices"),
  getGpuInfo: () => invoke<GpuInfo>("get_gpu_info"),
};

// Event Listeners
export const events = {
  onTranscription: (handler: (data: TranscriptionUpdate) => void): Promise<UnlistenFn> =>
    listen<TranscriptionUpdate>("transcription-update", (event) => handler(event.payload)),
  onDictationStatus: (handler: (status: string) => void): Promise<UnlistenFn> =>
    listen<string>("dictation-status", (event) => handler(event.payload)),
  onDownloadProgress: (handler: (data: DownloadProgress) => void): Promise<UnlistenFn> =>
    listen<DownloadProgress>("download-progress", (event) => handler(event.payload)),
  onOutputError: (handler: (message: string) => void): Promise<UnlistenFn> =>
    listen<string>("output-error", (event) => handler(event.payload)),
};
