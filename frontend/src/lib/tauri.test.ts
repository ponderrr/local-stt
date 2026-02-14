import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

// Mock @tauri-apps/api/event
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_event: string, _handler: unknown) => Promise.resolve(() => {})),
}));

import { commands, events } from "@/lib/tauri";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const mockedInvoke = vi.mocked(invoke);
const mockedListen = vi.mocked(listen);

describe("commands", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("toggleDictation calls invoke with correct command", async () => {
    mockedInvoke.mockResolvedValue(true);
    const result = await commands.toggleDictation();
    expect(mockedInvoke).toHaveBeenCalledWith("toggle_dictation");
    expect(result).toBe(true);
  });

  it("startDictation calls invoke with correct command", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    await commands.startDictation();
    expect(mockedInvoke).toHaveBeenCalledWith("start_dictation");
  });

  it("stopDictation calls invoke with correct command", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    await commands.stopDictation();
    expect(mockedInvoke).toHaveBeenCalledWith("stop_dictation");
  });

  it("listModels calls invoke with correct command", async () => {
    const mockModels = [{ id: "tiny", display_name: "Tiny", filename: "ggml-tiny.bin", size_bytes: 77691713, vram_mb: 1000, downloaded: true }];
    mockedInvoke.mockResolvedValue(mockModels);
    const result = await commands.listModels();
    expect(mockedInvoke).toHaveBeenCalledWith("list_models");
    expect(result).toEqual(mockModels);
  });

  it("downloadModel passes modelId parameter", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    await commands.downloadModel("tiny");
    expect(mockedInvoke).toHaveBeenCalledWith("download_model", { modelId: "tiny" });
  });

  it("deleteModel passes modelId parameter", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    await commands.deleteModel("base");
    expect(mockedInvoke).toHaveBeenCalledWith("delete_model", { modelId: "base" });
  });

  it("loadModel passes modelId parameter", async () => {
    mockedInvoke.mockResolvedValue(undefined);
    await commands.loadModel("small");
    expect(mockedInvoke).toHaveBeenCalledWith("load_model", { modelId: "small" });
  });

  it("getActiveModel calls invoke with correct command", async () => {
    mockedInvoke.mockResolvedValue("tiny");
    const result = await commands.getActiveModel();
    expect(mockedInvoke).toHaveBeenCalledWith("get_active_model");
    expect(result).toBe("tiny");
  });

  it("getActiveModel can return null", async () => {
    mockedInvoke.mockResolvedValue(null);
    const result = await commands.getActiveModel();
    expect(result).toBeNull();
  });

  it("getConfig calls invoke with correct command", async () => {
    const mockConfig = {
      version: 1,
      hotkey: "Ctrl+Shift+Space",
      default_model: "large-v3",
      output_mode: "both" as const,
      audio_device: null,
      language: "auto",
      vad_threshold: 0.01,
      chunk_duration_ms: 3000,
      overlap_ms: 500,
      downloaded_models: [],
      first_run_complete: false,
    };
    mockedInvoke.mockResolvedValue(mockConfig);
    const result = await commands.getConfig();
    expect(mockedInvoke).toHaveBeenCalledWith("get_config");
    expect(result).toEqual(mockConfig);
  });

  it("updateConfig passes config parameter", async () => {
    const config = {
      version: 1,
      hotkey: "Ctrl+Shift+Space",
      default_model: "tiny",
      output_mode: "clipboard" as const,
      audio_device: null,
      language: "en",
      vad_threshold: 0.01,
      chunk_duration_ms: 3000,
      overlap_ms: 500,
      downloaded_models: ["tiny"],
      first_run_complete: true,
    };
    mockedInvoke.mockResolvedValue(undefined);
    await commands.updateConfig(config);
    expect(mockedInvoke).toHaveBeenCalledWith("update_config", { config });
  });

  it("listAudioDevices calls invoke with correct command", async () => {
    const devices = ["Default", "USB Microphone"];
    mockedInvoke.mockResolvedValue(devices);
    const result = await commands.listAudioDevices();
    expect(mockedInvoke).toHaveBeenCalledWith("list_audio_devices");
    expect(result).toEqual(devices);
  });

  it("getGpuInfo calls invoke with correct command", async () => {
    const gpuInfo = { name: "NVIDIA RTX 3090", vram_total_mb: 24576, cuda_available: true };
    mockedInvoke.mockResolvedValue(gpuInfo);
    const result = await commands.getGpuInfo();
    expect(mockedInvoke).toHaveBeenCalledWith("get_gpu_info");
    expect(result).toEqual(gpuInfo);
  });
});

describe("events", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("onTranscription listens to transcription-update event", async () => {
    const handler = vi.fn();
    await events.onTranscription(handler);
    expect(mockedListen).toHaveBeenCalledWith("transcription-update", expect.any(Function));
  });

  it("onDictationStatus listens to dictation-status event", async () => {
    const handler = vi.fn();
    await events.onDictationStatus(handler);
    expect(mockedListen).toHaveBeenCalledWith("dictation-status", expect.any(Function));
  });

  it("onDownloadProgress listens to download-progress event", async () => {
    const handler = vi.fn();
    await events.onDownloadProgress(handler);
    expect(mockedListen).toHaveBeenCalledWith("download-progress", expect.any(Function));
  });

  it("onOutputError listens to output-error event", async () => {
    const handler = vi.fn();
    await events.onOutputError(handler);
    expect(mockedListen).toHaveBeenCalledWith("output-error", expect.any(Function));
  });

  it("onTranscriptionError listens to transcription-error event", async () => {
    const handler = vi.fn();
    await events.onTranscriptionError(handler);
    expect(mockedListen).toHaveBeenCalledWith("transcription-error", expect.any(Function));
  });

  it("event listeners return unlisten functions", async () => {
    const mockUnlisten = vi.fn();
    mockedListen.mockResolvedValue(mockUnlisten);
    const handler = vi.fn();
    const unlisten = await events.onDictationStatus(handler);
    expect(typeof unlisten).toBe("function");
  });
});
