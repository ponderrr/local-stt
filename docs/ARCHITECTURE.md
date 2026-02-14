# WhisperType Architecture

This document describes the system architecture of WhisperType, a local AI-powered
speech-to-text desktop application built with Tauri v2, Rust, and React.

## Table of Contents

- [System Overview](#system-overview)
- [Data Flow](#data-flow)
- [Backend Architecture](#backend-architecture)
- [Frontend Architecture](#frontend-architecture)
- [Integration Boundary: IPC Commands and Events](#integration-boundary-ipc-commands-and-events)

---

## System Overview

WhisperType is a Tauri v2 desktop application that captures audio from the system
microphone, detects speech using energy-based voice activity detection, transcribes
speech locally using Whisper (via `whisper-rs`/`whisper.cpp`), and outputs the
resulting text either by simulating keyboard input, copying to the clipboard, or both.

All inference happens on the user's machine. No audio or text data leaves the device.

```
+-------------------------------------------------------------------+
|                        WhisperType (Tauri v2)                     |
|                                                                   |
|  +---------------------+            +-------------------------+   |
|  |   Frontend (React)  |   IPC      |    Backend (Rust)       |   |
|  |                     | <--------> |                         |   |
|  |  main.tsx           | commands   |  lib.rs (entry)         |   |
|  |  App.tsx            | + events   |  commands/              |   |
|  |  pages/             |            |  audio/                 |   |
|  |  components/        |            |  transcription/         |   |
|  |  hooks/             |            |  output/                |   |
|  |  lib/tauri.ts       |            |  config/                |   |
|  +---------------------+            |  model_manager/         |   |
|                                     +-------------------------+   |
+-------------------------------------------------------------------+
```

### Technology Stack

| Layer      | Technology                                       |
|------------|--------------------------------------------------|
| Framework  | Tauri v2                                         |
| Backend    | Rust (edition 2021)                              |
| Frontend   | React 19, TypeScript, Tailwind CSS v4, Vite 7   |
| STT Engine | whisper-rs 0.15 (whisper.cpp bindings, CUDA)     |
| Audio      | cpal 0.15, ringbuf 0.4                           |
| Output     | enigo 0.6 (keyboard simulation), arboard 3       |
| Networking | reqwest 0.12 (model downloads)                   |

---

## Data Flow

The following diagram shows the complete data path from microphone input to text
output during active dictation.

```
  Microphone
      |
      v
+------------------+
|  AudioCapture    |   cpal input stream -> ring buffer producer
|  (cpal 0.15)    |   native sample rate (e.g. 48kHz stereo)
+------------------+
      |
      v  HeapRb<f32> (lock-free ring buffer, 3s capacity)
      |
+------------------+
|  Pipeline Thread |
|                  |
|  1. Read from    |   consumer.pop_slice() -> raw samples
|     ring buffer  |
|                  |
|  2. to_mono()    |   stereo/multi-ch -> mono (channel averaging)
|                  |
|  3. resample()   |   native rate -> 16kHz (linear interpolation)
|                  |
|  4. AudioRing-   |   write resampled samples; extract overlapping
|     Buffer       |   chunks (default: 3s chunks, 500ms overlap)
|                  |
|  5. VAD check    |   RMS energy > threshold for 3+ consecutive
|                  |   30ms frames = speech detected
+------------------+
      |
      v  mpsc::channel<Vec<f32>> (speech chunks only)
      |
+------------------+
|  Transcription   |   std::thread (NOT tokio)
|  Thread          |
|                  |
|  1. Receive      |   receiver.recv() blocks until chunk arrives
|     audio chunk  |
|                  |
|  2. Whisper      |   engine.transcribe(chunk, language)
|     inference    |   - Greedy sampling (best_of=1)
|                  |   - GPU-accelerated (CUDA) when available
|                  |
|  3. Output text  |   output::output_text(text, mode)
|                  |   - TypeIntoField: enigo keyboard simulation
|                  |   - Clipboard: arboard clipboard copy
|                  |   - Both: keyboard + clipboard
|                  |
|  4. Emit events  |   app.emit("transcription-update", ...)
|                  |   app.emit("dictation-status", ...)
+------------------+
      |
      v
+------------------+
|  Frontend        |   Receives events via Tauri event system
|  (React hooks)   |   Updates UI: transcript display, status
+------------------+
```

---

## Backend Architecture

The backend lives in `backend/src/`. The crate is named `whispertype` and exposes
the library name `tauri_app_lib`.

### Module Map

```
backend/src/
  lib.rs                  -- Entry point: run() builds Tauri app, registers
  |                          commands, plugins, global shortcut, manages AppState
  main.rs                 -- Binary entry: sets GDK_BACKEND=x11 on Linux,
  |                          disables DMA-BUF renderer, calls run()
  |
  commands/
  |  mod.rs               -- Re-exports: config, dictation, models, system
  |  dictation.rs         -- AppState struct, toggle/start/stop dictation
  |  models.rs            -- list/download/delete/load/get_active model commands
  |  config.rs            -- get_config, update_config commands
  |  system.rs            -- list_audio_devices, get_gpu_info commands
  |
  audio/
  |  mod.rs               -- AudioPipeline, to_mono(), resample(), unit tests
  |  capture.rs           -- AudioCapture: cpal stream management
  |  buffer.rs            -- AudioRingBuffer: circular buffer with overlap
  |  vad.rs               -- VoiceActivityDetector: RMS energy-based VAD
  |
  transcription/
  |  mod.rs               -- Re-exports: engine, models
  |  engine.rs            -- TranscriptionEngine: Whisper context management
  |  models.rs            -- WhisperModel registry (5 models, HuggingFace URLs)
  |
  output/
  |  mod.rs               -- output_text() dispatcher by OutputMode
  |  clipboard.rs         -- copy_to_clipboard() via arboard
  |  keyboard.rs          -- type_text() via enigo
  |
  config/
  |  mod.rs               -- Re-exports: Config, OutputMode
  |  settings.rs          -- Config struct, load/save, path management
  |
  model_manager/
     mod.rs               -- Re-exports: download, delete, is_model_downloaded
     download.rs          -- Async HTTP download with progress events, atomic writes
```

### Audio Pipeline

The `AudioPipeline` struct (`audio/mod.rs`) orchestrates the full capture-to-chunk
pipeline. It uses an `Arc<AtomicBool>` for thread-safe start/stop signaling, which
means all its methods take `&self` (no external Mutex needed).

**Capture** (`audio/capture.rs`):
- Uses `cpal` to open the default (or named) input device
- Reads the device's native sample rate and channel count
- Pushes raw `f32` samples into a lock-free `HeapRb<f32>` ring buffer (capacity:
  48000 * 2 * 3 = 288,000 samples, approximately 3 seconds of 48kHz stereo audio)

**Format Conversion** (`audio/mod.rs`):
- `to_mono()`: averages interleaved multi-channel frames into mono
- `resample()`: linear interpolation from device rate to 16kHz (Whisper's expected
  rate)

**Buffering** (`audio/buffer.rs`):
- `AudioRingBuffer`: circular buffer with configurable chunk size (default 48,000
  samples = 3s at 16kHz) and overlap (default 8,000 samples = 500ms)
- Tracks `samples_since_last` to determine when a new chunk is ready
- Chunk readiness condition: `samples_since_last >= (chunk_size - overlap_size)
  AND write_pos >= chunk_size`

**Voice Activity Detection** (`audio/vad.rs`):
- `VoiceActivityDetector`: energy-based RMS detection
- Frame size: 480 samples (30ms at 16kHz)
- Requires 3 consecutive voiced frames to trigger speech
- Requires 10 consecutive silent frames to end speech
- Only chunks that pass VAD are sent to the transcription thread

### Transcription Engine

`TranscriptionEngine` (`transcription/engine.rs`) wraps `whisper-rs`:

- Holds a `Mutex<Option<WhisperContext>>` and `Mutex<Option<String>>` for the active
  model ID
- `load_model()`: drops any existing context first (frees VRAM), then loads new
  context with GPU enabled
- `transcribe()`: creates a fresh `WhisperState` per call, uses greedy sampling
  (best_of=1), suppresses blanks and non-speech tokens, returns
  `Vec<TranscriptionSegment>` with text, start, and end timestamps

The engine is wrapped in `Arc` for thread sharing. Internal `Mutex`es protect the
context and active model state.

### Output Pipeline

`output::output_text()` (`output/mod.rs`) dispatches based on `OutputMode`:

| Mode            | Behavior                                            |
|-----------------|-----------------------------------------------------|
| `TypeIntoField` | `enigo::Enigo::text()` -- simulates keyboard input  |
| `Clipboard`     | `arboard::Clipboard::set_text()` -- copies to clipboard |
| `Both`          | Keyboard simulation first, then clipboard copy      |

### State Management

All shared state is held in `AppState` (defined in `commands/dictation.rs`):

```rust
pub struct AppState {
    pub engine: Arc<TranscriptionEngine>,
    pub pipeline: AudioPipeline,
    pub config: Mutex<Config>,
    pub transcription_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}
```

- `engine`: shared across dictation toggle and model management commands via `Arc`
- `pipeline`: uses internal `Arc<AtomicBool>` so methods take `&self`
- `config`: protected by `Mutex` for read/write access
- `transcription_thread`: tracks the spawned processing thread for clean shutdown

`AppState` is constructed in `lib.rs::run()` and registered via
`tauri::Builder::manage(app_state)`.

### Command Registration

All IPC commands are registered in `lib.rs` via `tauri::generate_handler![]`:

```rust
.invoke_handler(tauri::generate_handler![
    commands::dictation::toggle_dictation,
    commands::dictation::start_dictation,
    commands::dictation::stop_dictation,
    commands::models::list_models,
    commands::models::download_model,
    commands::models::delete_model,
    commands::models::load_model,
    commands::models::get_active_model,
    commands::config::get_config,
    commands::config::update_config,
    commands::system::list_audio_devices,
    commands::system::get_gpu_info,
])
```

### Global Shortcut

A global `Ctrl+Shift+Space` shortcut is registered in the Tauri `setup` hook using
`tauri-plugin-global-shortcut`. The handler calls `toggle_dictation_inner()` directly
(the same function used by the IPC command), ensuring consistent behavior.

### Platform Workarounds (main.rs)

On Linux, `main.rs` sets two environment variables before Tauri initialization:
- `GDK_BACKEND=x11`: forces XWayland for WebKitGTK to avoid Wayland protocol errors
  on some compositors (e.g. KDE Plasma)
- `WEBKIT_DISABLE_DMABUF_RENDERER=1`: disables GBM buffer creation that fails on
  newer NVIDIA GPUs, causing black screen

---

## Frontend Architecture

The frontend lives in `frontend/src/` and uses React 19 with TypeScript, styled via
Tailwind CSS v4. The Vite dev server runs on port 1420.

### Component Hierarchy

```
main.tsx
  React.StrictMode
    App.tsx (AppWithErrorBoundary)
      ErrorBoundary (class component)
        App (function component)
          |
          +-- [first_run_complete = false] --> SetupWizard
          |     |-- StepGpu          (GPU detection)
          |     |-- StepModels       (model selection)
          |     |-- StepDownload     (download with progress)
          |     +-- StepComplete     (finish setup)
          |
          +-- [first_run_complete = true]  --> MainWindow
                |-- ModelSelector    (model dropdown)
                |-- TranscriptDisplay (scrollable transcript area)
                |-- StatusIndicator  (idle/listening/loading/error dot)
                |-- SettingsPanel    (modal overlay)
```

### Hook Responsibilities

| Hook               | File                          | Purpose                                                   |
|--------------------|-------------------------------|-----------------------------------------------------------|
| `useDictation`     | `hooks/use-dictation.ts`      | Listens to `dictation-status`, `output-error`, `transcription-error` events; provides `status`, `toggle()`, `error` |
| `useTranscription` | `hooks/use-transcription.ts`  | Listens to `transcription-update` events; accumulates transcript text; provides `transcript`, `clear()` |
| `useModels`        | `hooks/use-models.ts`         | Calls `list_models`/`get_active_model` on mount; provides `models`, `activeModel`, `loadModel()`, `downloadModel()`, `deleteModel()`, `refresh()`, `loading` |
| `useConfig`        | `hooks/use-config.ts`         | Calls `get_config` on mount; provides `config`, `updateConfig()` |

### Tauri IPC Interface

All IPC communication is centralized in `lib/tauri.ts`. This module exports:
- `commands`: an object mapping command names to `invoke()` calls with proper types
- `events`: an object mapping event names to `listen()` subscriptions with proper
  payload types

See [API-REFERENCE.md](./API-REFERENCE.md) for the full typed interface.

---

## Integration Boundary: IPC Commands and Events

### Commands (Frontend -> Backend)

| Command              | Parameters                   | Return Type      | Module         |
|----------------------|------------------------------|------------------|----------------|
| `toggle_dictation`   | none                         | `boolean`        | dictation      |
| `start_dictation`    | none                         | `void`           | dictation      |
| `stop_dictation`     | none                         | `void`           | dictation      |
| `list_models`        | none                         | `ModelInfo[]`    | models         |
| `download_model`     | `modelId: string`            | `void`           | models         |
| `delete_model`       | `modelId: string`            | `void`           | models         |
| `load_model`         | `modelId: string`            | `void`           | models         |
| `get_active_model`   | none                         | `string \| null` | models         |
| `get_config`         | none                         | `Config`         | config         |
| `update_config`      | `config: Config`             | `void`           | config         |
| `list_audio_devices` | none                         | `string[]`       | system         |
| `get_gpu_info`       | none                         | `GpuInfo`        | system         |

### Events (Backend -> Frontend)

| Event                  | Payload Type           | When Emitted                              |
|------------------------|------------------------|-------------------------------------------|
| `dictation-status`     | `string`               | On dictation start (`"listening"`), stop (`"idle"`), model loading (`"loading"`) |
| `transcription-update` | `TranscriptionUpdate`  | After each Whisper segment is transcribed  |
| `download-progress`    | `DownloadProgress`     | During model download (per HTTP chunk)     |
| `output-error`         | `string`               | When text output (keyboard/clipboard) fails |
| `transcription-error`  | `string`               | When Whisper inference fails               |

### Payload Types

```typescript
interface TranscriptionUpdate {
  text: string;
  is_partial: boolean;
}

interface DownloadProgress {
  model_id: string;
  percent: number;
  downloaded_bytes: number;
  total_bytes: number;
}

interface Config {
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

interface ModelInfo {
  id: string;
  display_name: string;
  filename: string;
  size_bytes: number;
  vram_mb: number;
  downloaded: boolean;
}

interface GpuInfo {
  name: string;
  vram_total_mb: number;
  cuda_available: boolean;
}
```
