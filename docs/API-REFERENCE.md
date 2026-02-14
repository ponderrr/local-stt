# WhisperType API Reference

Complete reference for all Tauri IPC commands, events, Rust public types, and
frontend hook APIs.

See also: [ARCHITECTURE.md](./ARCHITECTURE.md) for system design,
[DEVELOPER-GUIDE.md](./DEVELOPER-GUIDE.md) for how to extend these APIs.

## Table of Contents

- [Tauri IPC Commands](#tauri-ipc-commands)
- [Tauri Events](#tauri-events)
- [Rust Public Types](#rust-public-types)
- [Frontend TypeScript Types](#frontend-typescript-types)
- [Frontend Hook API](#frontend-hook-api)

---

## Tauri IPC Commands

All commands are invoked from the frontend via `@tauri-apps/api/core::invoke()`.
The typed wrappers in `frontend/src/lib/tauri.ts` provide type-safe access.

Commands that accept parameters use Tauri's automatic camelCase conversion: a Rust
parameter named `model_id: String` is invoked as `{ modelId: "..." }` from
JavaScript.

### Dictation Commands

Source: `backend/src/commands/dictation.rs`

---

#### `toggle_dictation`

Toggle dictation on or off. If off, starts the audio pipeline and transcription
thread. If on, stops them.

| Property   | Value                                                      |
|------------|------------------------------------------------------------|
| Rust fn    | `toggle_dictation(state: State<AppState>, app: AppHandle)` |
| JS call    | `commands.toggleDictation()`                               |
| Parameters | none                                                       |
| Returns    | `boolean` -- `true` if dictation started, `false` if stopped |
| Errors     | `"No model loaded..."` if no model is currently loaded     |
| Events     | Emits `dictation-status` with `"listening"` or `"idle"`    |

---

#### `start_dictation`

Start dictation if not already running. No-op if already listening.

| Property   | Value                                                      |
|------------|------------------------------------------------------------|
| Rust fn    | `start_dictation(state: State<AppState>, app: AppHandle)`  |
| JS call    | `commands.startDictation()`                                |
| Parameters | none                                                       |
| Returns    | `void`                                                     |
| Errors     | Same as `toggle_dictation` (if no model loaded)            |

---

#### `stop_dictation`

Stop dictation if currently running. No-op if already idle.

| Property   | Value                                                      |
|------------|------------------------------------------------------------|
| Rust fn    | `stop_dictation(state: State<AppState>, app: AppHandle)`   |
| JS call    | `commands.stopDictation()`                                 |
| Parameters | none                                                       |
| Returns    | `void`                                                     |
| Events     | Emits `dictation-status` with `"idle"`                     |

---

### Model Management Commands

Source: `backend/src/commands/models.rs`

---

#### `list_models`

Return all models in the registry with their download status.

| Property   | Value                                       |
|------------|---------------------------------------------|
| Rust fn    | `list_models()`                             |
| JS call    | `commands.listModels()`                     |
| Parameters | none                                        |
| Returns    | `ModelInfo[]`                               |

The `downloaded` field on each `ModelInfo` is computed by checking whether the
model file exists in `~/.whispertype/models/`.

---

#### `download_model`

Download a model from HuggingFace. Emits progress events during download.

| Property   | Value                                                   |
|------------|---------------------------------------------------------|
| Rust fn    | `download_model(model_id: String, app: AppHandle)`      |
| JS call    | `commands.downloadModel(modelId)`                       |
| Parameters | `modelId: string` -- one of: `"tiny"`, `"base"`, `"small"`, `"medium"`, `"large-v3"` |
| Returns    | `void`                                                  |
| Errors     | `"Unknown model: ..."`, network errors, file I/O errors |
| Events     | Emits `download-progress` periodically during download   |
| Async      | yes                                                      |

The download uses atomic writes: data is written to a `.bin.tmp` file first, then
renamed to the final `.bin` filename on completion. If the model is already
downloaded (file exists and is non-empty), the download is skipped.

The model ID is added to `Config.downloaded_models` and saved after successful
download.

---

#### `delete_model`

Delete a downloaded model file from disk.

| Property   | Value                                                 |
|------------|-------------------------------------------------------|
| Rust fn    | `delete_model(model_id: String, state: State<AppState>)` |
| JS call    | `commands.deleteModel(modelId)`                       |
| Parameters | `modelId: string`                                     |
| Returns    | `void`                                                |
| Errors     | `"Unknown model: ..."`, file deletion errors          |

If the model being deleted is currently loaded, it is unloaded first (VRAM freed).
The model ID is removed from `Config.downloaded_models`.

---

#### `load_model`

Load a downloaded model into the Whisper engine (GPU/CPU memory).

| Property   | Value                                                       |
|------------|-------------------------------------------------------------|
| Rust fn    | `load_model(model_id: String, state: State<AppState>, app: AppHandle)` |
| JS call    | `commands.loadModel(modelId)`                               |
| Parameters | `modelId: string`                                           |
| Returns    | `void`                                                      |
| Errors     | `"Unknown model: ..."`, `"Model not downloaded: ..."`, Whisper load errors |
| Events     | Emits `dictation-status` with `"loading"` then `"idle"`     |
| Async      | yes (uses `tokio::task::spawn_blocking` for the heavy load) |

Any previously loaded model is unloaded first to free VRAM. After loading,
`Config.default_model` is updated to the new model ID.

---

#### `get_active_model`

Return the ID of the currently loaded model, or null if none.

| Property   | Value                                        |
|------------|----------------------------------------------|
| Rust fn    | `get_active_model(state: State<AppState>)`   |
| JS call    | `commands.getActiveModel()`                  |
| Parameters | none                                         |
| Returns    | `string \| null`                             |

---

### Configuration Commands

Source: `backend/src/commands/config.rs`

---

#### `get_config`

Return the current application configuration.

| Property   | Value                                        |
|------------|----------------------------------------------|
| Rust fn    | `get_config(state: State<AppState>)`         |
| JS call    | `commands.getConfig()`                       |
| Parameters | none                                         |
| Returns    | `Config`                                     |

---

#### `update_config`

Replace the entire configuration and save to disk.

| Property   | Value                                                   |
|------------|---------------------------------------------------------|
| Rust fn    | `update_config(config: Config, state: State<AppState>)` |
| JS call    | `commands.updateConfig(config)`                         |
| Parameters | `config: Config` -- the full config object              |
| Returns    | `void`                                                  |
| Errors     | Mutex lock errors, file I/O errors                      |

The entire config is replaced (not merged). The frontend must send the complete
`Config` object.

---

### System Commands

Source: `backend/src/commands/system.rs`

---

#### `list_audio_devices`

Return names of all available audio input devices.

| Property   | Value                               |
|------------|-------------------------------------|
| Rust fn    | `list_audio_devices()`              |
| JS call    | `commands.listAudioDevices()`       |
| Parameters | none                                |
| Returns    | `string[]`                          |
| Errors     | Device enumeration errors           |

---

#### `get_gpu_info`

Return information about the NVIDIA GPU (if available).

| Property   | Value                               |
|------------|-------------------------------------|
| Rust fn    | `get_gpu_info()`                    |
| JS call    | `commands.getGpuInfo()`             |
| Parameters | none                                |
| Returns    | `GpuInfo`                           |
| Errors     | `"nvidia-smi not found..."` if CUDA is not installed |

Internally runs `nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits`
and parses the output.

---

## Tauri Events

Events are emitted from the Rust backend using `AppHandle::emit()` and received
on the frontend via `@tauri-apps/api/event::listen()`. All event subscriptions are
centralized in `frontend/src/lib/tauri.ts`.

---

### `dictation-status`

Emitted when the dictation state changes.

| Property   | Value                                                  |
|------------|--------------------------------------------------------|
| Event name | `"dictation-status"`                                   |
| Payload    | `string` -- one of `"idle"`, `"listening"`, `"loading"` |
| JS handler | `events.onDictationStatus(handler)`                    |
| Emitted by | `toggle_dictation_inner()`, `load_model()`, `stop_dictation()`, transcription thread completion |

---

### `transcription-update`

Emitted when a transcription segment is produced.

| Property   | Value                                              |
|------------|----------------------------------------------------|
| Event name | `"transcription-update"`                           |
| Payload    | `TranscriptionUpdate` (`{ text: string, is_partial: boolean }`) |
| JS handler | `events.onTranscription(handler)`                  |
| Emitted by | Transcription thread, after each Whisper segment   |

Currently, `is_partial` is always `false` (all segments are final). The field
exists for future streaming/partial transcription support.

---

### `download-progress`

Emitted periodically during model download.

| Property   | Value                                              |
|------------|----------------------------------------------------|
| Event name | `"download-progress"`                              |
| Payload    | `DownloadProgress` (see type below)                |
| JS handler | `events.onDownloadProgress(handler)`               |
| Emitted by | `model_manager::download_model()`, per HTTP chunk  |

```typescript
interface DownloadProgress {
  model_id: string;         // e.g. "large-v3"
  percent: number;          // 0.0 to 100.0
  downloaded_bytes: number; // bytes downloaded so far
  total_bytes: number;      // total file size in bytes
}
```

---

### `output-error`

Emitted when text output (keyboard simulation or clipboard) fails.

| Property   | Value                                              |
|------------|----------------------------------------------------|
| Event name | `"output-error"`                                   |
| Payload    | `string` -- error message                          |
| JS handler | `events.onOutputError(handler)`                    |
| Emitted by | Transcription thread, when `output::output_text()` fails |

---

### `transcription-error`

Emitted when Whisper inference fails on an audio chunk.

| Property   | Value                                              |
|------------|----------------------------------------------------|
| Event name | `"transcription-error"`                            |
| Payload    | `string` -- error message                          |
| JS handler | `events.onTranscriptionError(handler)`             |
| Emitted by | Transcription thread, when `engine.transcribe()` fails |

---

## Rust Public Types

### `AppState`

Source: `backend/src/commands/dictation.rs`

Central application state managed by Tauri.

```rust
pub struct AppState {
    pub engine: Arc<TranscriptionEngine>,
    pub pipeline: AudioPipeline,
    pub config: Mutex<Config>,
    pub transcription_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}
```

| Field                  | Type                                     | Description                          |
|------------------------|------------------------------------------|--------------------------------------|
| `engine`               | `Arc<TranscriptionEngine>`               | Whisper engine, shared across threads |
| `pipeline`             | `AudioPipeline`                          | Audio capture/processing pipeline    |
| `config`               | `Mutex<Config>`                          | User configuration                   |
| `transcription_thread` | `Mutex<Option<JoinHandle<()>>>`          | Handle to spawned transcription thread |

---

### `Config`

Source: `backend/src/config/settings.rs`

User-facing configuration, persisted to `~/.whispertype/config.json`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub hotkey: String,
    pub default_model: String,
    pub output_mode: OutputMode,
    pub audio_device: Option<String>,
    pub language: String,
    pub vad_threshold: f32,
    pub chunk_duration_ms: u32,
    pub overlap_ms: u32,
    pub downloaded_models: Vec<String>,
    pub first_run_complete: bool,
}
```

| Field               | Type             | Default               | Description                                   |
|---------------------|------------------|-----------------------|-----------------------------------------------|
| `version`           | `u32`            | `1`                   | Config schema version                         |
| `hotkey`            | `String`         | `"Ctrl+Shift+Space"`  | Global shortcut display string                |
| `default_model`     | `String`         | `"large-v3"`          | Model ID to auto-load on startup              |
| `output_mode`       | `OutputMode`     | `Both`                | How transcribed text is output                |
| `audio_device`      | `Option<String>` | `None`                | Audio input device name (`None` = system default) |
| `language`          | `String`         | `"auto"`              | Whisper language code or `"auto"` for detection |
| `vad_threshold`     | `f32`            | `0.01`                | RMS energy threshold for speech detection     |
| `chunk_duration_ms` | `u32`            | `3000`                | Audio chunk duration in milliseconds          |
| `overlap_ms`        | `u32`            | `500`                 | Overlap between consecutive chunks            |
| `downloaded_models` | `Vec<String>`    | `[]`                  | List of downloaded model IDs                  |
| `first_run_complete`| `bool`           | `false`               | Whether setup wizard has been completed       |

Static methods:

| Method          | Returns    | Description                               |
|-----------------|------------|-------------------------------------------|
| `app_dir()`     | `PathBuf`  | `~/.whispertype`                          |
| `models_dir()`  | `PathBuf`  | `~/.whispertype/models`                   |
| `config_path()` | `PathBuf`  | `~/.whispertype/config.json`              |
| `ensure_dirs()` | `Result`   | Creates app, models, and logs directories |
| `load()`        | `Result<Config>` | Loads from disk or creates default  |
| `save()`        | `Result`   | Serializes and writes to disk             |

---

### `OutputMode`

Source: `backend/src/config/settings.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    TypeIntoField,
    Clipboard,
    Both,
}
```

Serialized as `"type_into_field"`, `"clipboard"`, or `"both"` in JSON.

---

### `TranscriptionEngine`

Source: `backend/src/transcription/engine.rs`

Thread-safe Whisper context wrapper.

```rust
pub struct TranscriptionEngine {
    ctx: Mutex<Option<WhisperContext>>,
    active_model: Mutex<Option<String>>,
}
```

| Method                    | Signature                                                    | Description                     |
|---------------------------|--------------------------------------------------------------|---------------------------------|
| `new()`                   | `() -> Self`                                                 | Creates engine with no model    |
| `load_model()`            | `(&self, path: &Path, id: &str) -> Result<(), String>`       | Loads a GGML model file         |
| `unload_model()`          | `(&self) -> Result<(), String>`                              | Unloads current model, frees VRAM |
| `get_active_model()`      | `(&self) -> Option<String>`                                  | Returns active model ID         |
| `is_loaded()`             | `(&self) -> bool`                                            | Whether a model is loaded       |
| `transcribe()`            | `(&self, audio: &[f32], lang: &str) -> Result<Vec<TranscriptionSegment>, String>` | Run inference |

---

### `TranscriptionSegment`

Source: `backend/src/transcription/engine.rs`

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: i64,
    pub end: i64,
}
```

| Field   | Type     | Description                              |
|---------|----------|------------------------------------------|
| `text`  | `String` | Transcribed text (trimmed whitespace)    |
| `start` | `i64`    | Start timestamp (Whisper time units)     |
| `end`   | `i64`    | End timestamp (Whisper time units)       |

---

### `WhisperModel`

Source: `backend/src/transcription/models.rs`

Registry entry for an available Whisper model.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub id: String,
    pub display_name: String,
    pub filename: String,
    pub url: String,
    pub size_bytes: u64,
    pub vram_mb: u16,
}
```

| Field          | Type     | Example                            |
|----------------|----------|------------------------------------|
| `id`           | `String` | `"large-v3"`                       |
| `display_name` | `String` | `"Large V3 (~3 GB)"`              |
| `filename`     | `String` | `"ggml-large-v3.bin"`              |
| `url`          | `String` | `"https://huggingface.co/..."`     |
| `size_bytes`   | `u64`    | `3093846125`                       |
| `vram_mb`      | `u16`    | `6000`                             |

Access the registry: `get_model_registry() -> &'static [WhisperModel]`

---

### `ModelInfo`

Source: `backend/src/commands/models.rs`

Serializable model information sent to the frontend (includes `downloaded` status).

```rust
#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub vram_mb: u16,
    pub downloaded: bool,
}
```

Created from `WhisperModel` via `From` trait, with `downloaded` computed by checking
disk.

---

### `AudioPipeline`

Source: `backend/src/audio/mod.rs`

Orchestrates audio capture, format conversion, buffering, and VAD.

```rust
pub struct AudioPipeline {
    is_running: Arc<AtomicBool>,
}
```

| Method         | Signature                                                                     | Description                          |
|----------------|-------------------------------------------------------------------------------|--------------------------------------|
| `new()`        | `() -> Self`                                                                  | Creates a stopped pipeline           |
| `start()`      | `(&self, device: Option<String>, threshold: f32, chunk_ms: u32, overlap_ms: u32) -> Result<Receiver<Vec<f32>>, String>` | Starts capture and returns chunk receiver |
| `stop()`       | `(&self)`                                                                     | Signals the pipeline thread to stop  |
| `is_running()` | `(&self) -> bool`                                                             | Whether the pipeline is active       |

---

### `AudioCapture`

Source: `backend/src/audio/capture.rs`

Manages the cpal input stream.

```rust
pub struct AudioCapture {
    stream: Option<Stream>,
    pub device_sample_rate: u32,
    pub device_channels: u16,
}
```

| Method          | Signature                                                            | Description                        |
|-----------------|----------------------------------------------------------------------|------------------------------------|
| `new()`         | `() -> Self`                                                         | Creates capture (default 48kHz/1ch)|
| `list_devices()`| `() -> Result<Vec<String>, String>`                                  | Lists input device names           |
| `start()`       | `(&mut self, name: Option<&str>, producer: HeapProd<f32>) -> Result` | Opens device and starts streaming  |

---

### `AudioRingBuffer`

Source: `backend/src/audio/buffer.rs`

Circular buffer for accumulating and extracting overlapping audio chunks.

```rust
pub struct AudioRingBuffer {
    data: Vec<f32>,
    write_pos: usize,
    capacity: usize,
    chunk_size: usize,
    overlap_size: usize,
    samples_since_last: usize,
}
```

| Method            | Signature                                                  | Description                            |
|-------------------|------------------------------------------------------------|----------------------------------------|
| `new()`           | `(rate: u32, chunk_ms: u32, overlap_ms: u32, buf_s: u32) -> Self` | Creates buffer with computed sizes |
| `write()`         | `(&mut self, samples: &[f32])`                             | Writes samples into circular buffer    |
| `has_chunk()`     | `(&self) -> bool`                                          | Whether a new chunk is ready           |
| `extract_chunk()` | `(&mut self) -> Option<Vec<f32>>`                          | Extracts latest chunk, resets counter  |

---

### `VoiceActivityDetector`

Source: `backend/src/audio/vad.rs`

Energy-based voice activity detector.

```rust
pub struct VoiceActivityDetector {
    threshold: f32,
    min_speech_frames: usize,    // 3
    min_silence_frames: usize,   // 10
    speech_frame_count: usize,
    silence_frame_count: usize,
    is_speech: bool,
}
```

| Method            | Signature                            | Description                               |
|-------------------|--------------------------------------|-------------------------------------------|
| `new()`           | `(threshold: f32) -> Self`           | Creates VAD with given energy threshold   |
| `rms_energy()`    | `(samples: &[f32]) -> f32` (private) | Computes RMS energy of a frame            |
| `process_frame()` | `(&mut self, frame: &[f32]) -> bool` | Process one frame, returns speech state   |
| `contains_speech()`| `(&mut self, audio: &[f32]) -> bool`| Bulk check: any speech in entire chunk    |

---

## Frontend TypeScript Types

Source: `frontend/src/lib/tauri.ts`

### `Config`

```typescript
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
```

### `ModelInfo`

```typescript
interface ModelInfo {
  id: string;
  display_name: string;
  filename: string;
  size_bytes: number;
  vram_mb: number;
  downloaded: boolean;
}
```

### `GpuInfo`

```typescript
interface GpuInfo {
  name: string;
  vram_total_mb: number;
  cuda_available: boolean;
}
```

### `TranscriptionUpdate`

```typescript
interface TranscriptionUpdate {
  text: string;
  is_partial: boolean;
}
```

### `DownloadProgress`

```typescript
interface DownloadProgress {
  model_id: string;
  percent: number;
  downloaded_bytes: number;
  total_bytes: number;
}
```

---

## Frontend Hook API

### `useDictation()`

Source: `frontend/src/hooks/use-dictation.ts`

Manages dictation lifecycle and error state.

```typescript
function useDictation(): {
  status: "idle" | "listening" | "loading" | "error";
  toggle: () => Promise<void>;
  error: string | null;
}
```

| Return Field | Type                                      | Description                              |
|-------------|-------------------------------------------|------------------------------------------|
| `status`    | `"idle" \| "listening" \| "loading" \| "error"` | Current dictation status            |
| `toggle`    | `() => Promise<void>`                     | Calls `toggle_dictation` command         |
| `error`     | `string \| null`                          | Current error message (auto-clears after 5s) |

**Event subscriptions**:
- `dictation-status`: updates `status`
- `output-error`: sets `error`, starts 5-second auto-clear timer
- `transcription-error`: sets `error`, starts 5-second auto-clear timer

---

### `useTranscription()`

Source: `frontend/src/hooks/use-transcription.ts`

Accumulates transcribed text from Whisper events.

```typescript
function useTranscription(): {
  transcript: string;
  clear: () => void;
}
```

| Return Field | Type         | Description                                  |
|-------------|--------------|----------------------------------------------|
| `transcript`| `string`     | Accumulated transcription text               |
| `clear`     | `() => void` | Clears the transcript to empty string        |

New segments are appended with a space separator (unless the current transcript
already ends with a space).

**Event subscriptions**:
- `transcription-update`: appends `data.text` to transcript

---

### `useModels()`

Source: `frontend/src/hooks/use-models.ts`

Manages the model registry, active model, and model lifecycle operations.

```typescript
function useModels(): {
  models: ModelInfo[];
  activeModel: string | null;
  loadModel: (modelId: string) => Promise<void>;
  downloadModel: (modelId: string) => Promise<void>;
  deleteModel: (modelId: string) => Promise<void>;
  refresh: () => Promise<void>;
  loading: boolean;
}
```

| Return Field    | Type                                   | Description                         |
|----------------|----------------------------------------|-------------------------------------|
| `models`       | `ModelInfo[]`                          | All models from registry            |
| `activeModel`  | `string \| null`                       | Currently loaded model ID           |
| `loadModel`    | `(id: string) => Promise<void>`        | Loads a model into the engine       |
| `downloadModel`| `(id: string) => Promise<void>`        | Downloads a model, then refreshes   |
| `deleteModel`  | `(id: string) => Promise<void>`        | Deletes a model, then refreshes     |
| `refresh`      | `() => Promise<void>`                  | Re-fetches model list and active model |
| `loading`      | `boolean`                              | `true` until initial fetch completes |

**Event subscriptions**:
- `download-progress`: auto-refreshes model list when `percent >= 100`

---

### `useConfig()`

Source: `frontend/src/hooks/use-config.ts`

Manages application configuration.

```typescript
function useConfig(): {
  config: Config | null;
  updateConfig: (newConfig: Config) => Promise<void>;
}
```

| Return Field   | Type                                  | Description                         |
|---------------|---------------------------------------|-------------------------------------|
| `config`      | `Config \| null`                      | Current config (`null` while loading) |
| `updateConfig`| `(config: Config) => Promise<void>`   | Saves new config to backend         |
