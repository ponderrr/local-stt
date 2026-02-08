# WhisperType (local-stt) — Phase 3: Atomic Micro-Phase Decomposition

> **Methodology:** Andrew Ponder Build System
> **Repo:** `https://github.com/ponderrr/local-stt.git`
> **Hardware:** RTX 5060 Ti (16GB VRAM), Ryzen 7 9700X, 32GB RAM, CachyOS Linux

---

# Section A: Project Scaffolding

**Estimated time: 30-45 min | Phases A.1 – A.6**

---

## Phase A.1: Initialize Tauri + React Project

### GOAL

Scaffold a Tauri v2 project with React + TypeScript frontend on CachyOS Linux.

### REQUIREMENTS

- Rust toolchain installed (`rustup`)
- Node.js 18+ and npm
- System dependencies: `webkit2gtk`, `base-devel`, `curl`, `wget`, `file`, `openssl`, `appmenu-gtk-module`, `libappindicator-gtk3`

### TASKS

**Step 1:** Install Tauri system dependencies (CachyOS/Arch)

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module libappindicator-gtk3 librsvg
```

**Step 2:** Create the project

```bash
cd ~/projects  # or wherever you keep repos
npm create tauri-app@latest local-stt -- --template react-ts
cd local-stt
```

**Step 3:** Initialize git and connect to remote

```bash
git init
git remote add origin https://github.com/ponderrr/local-stt.git
```

**Step 4:** Verify it runs

```bash
cargo tauri dev
```

A window should appear with the default Tauri + React template.

### COMPLETION CHECKLIST

- [x] `cargo tauri dev` opens a window with React content
- [x] `frontend/` contains React TypeScript files
- [x] `backend/` contains Rust source and `Cargo.toml`
- [x] Git remote points to `ponderrr/local-stt`

### VALIDATION

```bash
ls backend/src/main.rs && echo "✅ Tauri backend exists"
ls frontend/App.tsx && echo "✅ React frontend exists"
cd backend && cargo check && echo "✅ Rust compiles"
cd .. && npm run build && echo "✅ Frontend builds"
```

### COMMIT

```
chore(init): scaffold Tauri v2 + React TypeScript project
```

### NEXT

→ A.2: Install and configure Tailwind CSS + shadcn/ui

---

## Phase A.2: Install Tailwind CSS + shadcn/ui

### GOAL

Set up Tailwind CSS and shadcn/ui with the WhisperType dark design system tokens.

### TASKS

**Step 1:** Install Tailwind

```bash
npm install -D tailwindcss @tailwindcss/vite
```

**Step 2:** Configure `frontend/index.css` — replace all content with:

```css
@import "tailwindcss";

@layer base {
  :root {
    --background: 240 6% 6%;
    --foreground: 0 0% 98%;
    --card: 240 5% 8%;
    --card-foreground: 0 0% 98%;
    --popover: 240 5% 10%;
    --popover-foreground: 0 0% 98%;
    --primary: 217 91% 60%;
    --primary-foreground: 0 0% 98%;
    --secondary: 240 4% 16%;
    --secondary-foreground: 0 0% 98%;
    --muted: 240 4% 16%;
    --muted-foreground: 240 5% 55%;
    --accent: 240 4% 16%;
    --accent-foreground: 0 0% 98%;
    --destructive: 0 72% 51%;
    --destructive-foreground: 0 0% 98%;
    --border: 240 4% 14%;
    --input: 240 4% 14%;
    --ring: 217 91% 60%;
    --radius: 0.5rem;
  }
}

@layer base {
  * {
    @apply border-border;
  }
  body {
    @apply bg-background text-foreground;
    font-family: system-ui, -apple-system, sans-serif;
  }
}
```

**Step 3:** Initialize shadcn/ui

```bash
npx shadcn@latest init
```

When prompted:

- Style: Default
- Base color: Zinc
- CSS variables: Yes

**Step 4:** Install initial shadcn components we'll need

```bash
npx shadcn@latest add button card select dropdown-menu progress dialog scroll-area badge separator tooltip
```

**Step 5:** Update `vite.config.ts` to include the Tailwind plugin:

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import path from "path";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
});
```

### COMPLETION CHECKLIST

- [ ] Tailwind utility classes render correctly in browser
- [ ] Background is rich charcoal `#0f0f11`, not white or pure black
- [ ] shadcn/ui components importable (e.g., `import { Button } from "@/components/ui/button"`)

### VALIDATION

```bash
npm run build && echo "✅ Frontend builds with Tailwind"
ls src/components/ui/button.tsx && echo "✅ shadcn/ui components installed"
```

### COMMIT

```
feat(ui): configure Tailwind CSS + shadcn/ui dark design system
```

### NEXT

→ A.3: Configure Rust dependencies in Cargo.toml

---

## Phase A.3: Configure Rust Dependencies

### GOAL

Add all required Rust crates to `Cargo.toml` and verify they compile.

### TASKS

**Step 1:** Install CUDA toolkit (required for whisper-rs GPU acceleration)

```bash
sudo pacman -S cuda cudnn
```

**Step 2:** Update `backend/Cargo.toml` dependencies:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-shell = "2"
tauri-plugin-global-shortcut = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
whisper-rs = { version = "0.13", features = ["cuda"] }
cpal = "0.15"
enigo = { version = "0.2", features = ["wayland", "x11"] }
arboard = "3"
reqwest = { version = "0.12", features = ["stream"] }
futures-util = "0.3"
hound = "3"
dirs = "5"
```

**Step 3:** Verify compilation (this will take a few minutes on first build — whisper.cpp compiles from source with CUDA)

```bash
cd backend && cargo check
```

> **Note:** If `whisper-rs` fails to find CUDA, ensure `nvcc` is in PATH:
>
> ```bash
> export CUDA_PATH=/opt/cuda
> export PATH=$CUDA_PATH/bin:$PATH
> ```
>
> You may need to add this to `~/.bashrc` or `~/.zshrc`.

### COMPLETION CHECKLIST

- [ ] `cargo check` succeeds with all dependencies
- [ ] No CUDA-related build errors
- [ ] whisper-rs compiles with CUDA feature

### VALIDATION

```bash
cd backend && cargo check 2>&1 | tail -5
echo $? # Should be 0
```

### COMMIT

```
chore(deps): add whisper-rs, cpal, enigo, and all Rust dependencies
```

### NEXT

→ A.4: Create Rust module structure

---

## Phase A.4: Create Rust Module Structure

### GOAL

Set up the complete Rust module file tree with empty modules and proper `mod` declarations.

### TASKS

**Step 1:** Create directory structure

```bash
cd backend/src
mkdir -p audio transcription output hotkey model_manager config commands
```

**Step 2:** Create all module files with placeholder module declarations:

`backend/src/lib.rs`:

```rust
pub mod audio;
pub mod transcription;
pub mod output;
pub mod hotkey;
pub mod model_manager;
pub mod config;
pub mod commands;
```

`backend/src/audio/mod.rs`:

```rust
pub mod capture;
pub mod vad;
pub mod buffer;
```

`backend/src/transcription/mod.rs`:

```rust
pub mod engine;
pub mod models;
```

`backend/src/output/mod.rs`:

```rust
pub mod keyboard;
pub mod clipboard;
```

`backend/src/hotkey/mod.rs`:

```rust
pub mod manager;
```

`backend/src/model_manager/mod.rs`:

```rust
pub mod download;
pub mod storage;
```

`backend/src/config/mod.rs`:

```rust
pub mod settings;
```

`backend/src/commands/mod.rs`:

```rust
pub mod dictation;
pub mod models;
pub mod config;
pub mod system;
```

**Step 3:** Create empty source files for each module (each with a placeholder comment):

```bash
for f in audio/capture audio/vad audio/buffer \
         transcription/engine transcription/models \
         output/keyboard output/clipboard \
         hotkey/manager \
         model_manager/download model_manager/storage \
         config/settings \
         commands/dictation commands/models commands/config commands/system; do
    echo "// TODO: Implement" > backend/src/${f}.rs
done
```

**Step 4:** Verify

```bash
cd backend && cargo check
```

### COMPLETION CHECKLIST

- [ ] All directories created under `backend/src/`
- [ ] All `mod.rs` files declare their sub-modules
- [ ] `lib.rs` declares all top-level modules
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Module structure compiles"
find src -name "*.rs" | wc -l  # Should be ~18 files
```

### COMMIT

```
feat(structure): create complete Rust module file tree
```

### NEXT

→ A.5: Create React component structure

---

## Phase A.5: Create React Component Structure

### GOAL

Set up the complete frontend file tree with empty component shells.

### TASKS

**Step 1:** Create directories

```bash
mkdir -p src/hooks src/components/setup-wizard src/pages src/lib
```

**Step 2:** Create utility file `src/lib/utils.ts`:

```ts
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
```

**Step 3:** Create typed Tauri IPC wrapper `src/lib/tauri.ts`:

```ts
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
};
```

**Step 4:** Create empty component shells:

`src/components/transcript-display.tsx`:

```tsx
export function TranscriptDisplay() {
  return <div>TODO: Transcript display</div>;
}
```

`src/components/model-selector.tsx`:

```tsx
export function ModelSelector() {
  return <div>TODO: Model selector</div>;
}
```

`src/components/status-indicator.tsx`:

```tsx
export function StatusIndicator() {
  return <div>TODO: Status indicator</div>;
}
```

`src/components/settings-panel.tsx`:

```tsx
export function SettingsPanel() {
  return <div>TODO: Settings</div>;
}
```

`src/components/setup-wizard/index.tsx`:

```tsx
export function SetupWizard() {
  return <div>TODO: Setup wizard</div>;
}
```

`src/pages/main-window.tsx`:

```tsx
export function MainWindow() {
  return <div>TODO: Main window</div>;
}
```

`src/pages/setup.tsx`:

```tsx
export function SetupPage() {
  return <div>TODO: Setup page</div>;
}
```

**Step 5:** Create hook shells:

`src/hooks/use-dictation.ts`:

```ts
export function useDictation() {
  // TODO
  return { isListening: false, toggle: async () => {} };
}
```

`src/hooks/use-models.ts`:

```ts
export function useModels() {
  // TODO
  return { models: [], activeModel: null, loadModel: async (_id: string) => {} };
}
```

`src/hooks/use-config.ts`:

```ts
export function useConfig() {
  // TODO
  return { config: null, updateConfig: async () => {} };
}
```

`src/hooks/use-transcription.ts`:

```ts
export function useTranscription() {
  // TODO
  return { transcript: "", clear: () => {} };
}
```

**Step 6:** Update `src/App.tsx`:

```tsx
import { MainWindow } from "./pages/main-window";

function App() {
  return (
    <div className="h-screen bg-background text-foreground">
      <MainWindow />
    </div>
  );
}

export default App;
```

### COMPLETION CHECKLIST

- [ ] All directories exist: `hooks/`, `components/`, `pages/`, `lib/`
- [ ] `src/lib/tauri.ts` has all typed IPC wrappers
- [ ] All component shells render without errors
- [ ] `npm run build` succeeds

### VALIDATION

```bash
npm run build && echo "✅ Frontend builds"
find src -name "*.tsx" -o -name "*.ts" | wc -l  # Should be ~15+ files
```

### COMMIT

```
feat(ui): create React component structure with typed Tauri IPC layer
```

### NEXT

→ A.6: Create app data directory and config defaults

---

## Phase A.6: App Data Directory + Config Defaults

### GOAL

Implement the config module: default config, load/save to `~/.whispertype/config.json`, and model registry.

### TASKS

**Step 1:** Implement `backend/src/config/settings.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    TypeIntoField,
    Clipboard,
    Both,
}

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

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            hotkey: "Ctrl+Shift+Space".to_string(),
            default_model: "large-v3".to_string(),
            output_mode: OutputMode::Both,
            audio_device: None,
            language: "auto".to_string(),
            vad_threshold: 0.3,
            chunk_duration_ms: 3000,
            overlap_ms: 500,
            downloaded_models: Vec::new(),
            first_run_complete: false,
        }
    }
}

impl Config {
    pub fn app_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".whispertype")
    }

    pub fn models_dir() -> PathBuf {
        Self::app_dir().join("models")
    }

    pub fn config_path() -> PathBuf {
        Self::app_dir().join("config.json")
    }

    pub fn ensure_dirs() -> Result<(), String> {
        let app_dir = Self::app_dir();
        fs::create_dir_all(&app_dir).map_err(|e| format!("Failed to create app dir: {}", e))?;
        fs::create_dir_all(Self::models_dir())
            .map_err(|e| format!("Failed to create models dir: {}", e))?;
        fs::create_dir_all(app_dir.join("logs"))
            .map_err(|e| format!("Failed to create logs dir: {}", e))?;
        Ok(())
    }

    pub fn load() -> Result<Self, String> {
        Self::ensure_dirs()?;
        let path = Self::config_path();
        if path.exists() {
            let content =
                fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), String> {
        Self::ensure_dirs()?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(Self::config_path(), content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }
}
```

**Step 2:** Implement `backend/src/transcription/models.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub id: String,
    pub display_name: String,
    pub filename: String,
    pub url: String,
    pub size_bytes: u64,
    pub vram_mb: u16,
}

pub fn get_model_registry() -> Vec<WhisperModel> {
    vec![
        WhisperModel {
            id: "tiny".to_string(),
            display_name: "Tiny (~75 MB)".to_string(),
            filename: "ggml-tiny.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
                .to_string(),
            size_bytes: 77_691_713,
            vram_mb: 1000,
        },
        WhisperModel {
            id: "base".to_string(),
            display_name: "Base (~150 MB)".to_string(),
            filename: "ggml-base.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
                .to_string(),
            size_bytes: 147_951_465,
            vram_mb: 1000,
        },
        WhisperModel {
            id: "small".to_string(),
            display_name: "Small (~500 MB)".to_string(),
            filename: "ggml-small.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
                .to_string(),
            size_bytes: 487_601_967,
            vram_mb: 1500,
        },
        WhisperModel {
            id: "medium".to_string(),
            display_name: "Medium (~1.5 GB)".to_string(),
            filename: "ggml-medium.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
                .to_string(),
            size_bytes: 1_533_774_781,
            vram_mb: 3000,
        },
        WhisperModel {
            id: "large-v3".to_string(),
            display_name: "Large V3 (~3 GB)".to_string(),
            filename: "ggml-large-v3.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .to_string(),
            size_bytes: 3_094_623_691,
            vram_mb: 5500,
        },
    ]
}
```

**Step 3:** Update `backend/src/config/mod.rs`:

```rust
pub mod settings;
pub use settings::{Config, OutputMode};
```

**Step 4:** Update `backend/src/transcription/mod.rs`:

```rust
pub mod engine;
pub mod models;
pub use models::{get_model_registry, WhisperModel};
```

### COMPLETION CHECKLIST

- [ ] `Config::default()` produces valid config
- [ ] `Config::load()` creates `~/.whispertype/config.json` on first run
- [ ] `Config::save()` writes pretty-printed JSON
- [ ] `get_model_registry()` returns all 5 models with valid HuggingFace URLs
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Config + model registry compiles"
cd backend && cargo test && echo "✅ Tests pass"
```

### COMMIT

```
feat(config): implement config persistence and Whisper model registry
```

### NEXT

→ B.1: Implement audio capture with cpal

---

# Section B: Audio Engine

**Estimated time: 45-60 min | Phases B.1 – B.5**

---

## Phase B.1: Microphone Capture with cpal

### GOAL

Capture audio from the default microphone at 16kHz mono (Whisper's required format) using cpal.

### TASKS

**Step 1:** Implement `backend/src/audio/capture.rs`:

```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleRate, Stream, StreamConfig};
use std::sync::mpsc;

pub struct AudioCapture {
    stream: Option<Stream>,
    pub sample_rate: u32,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            sample_rate: 16000,
        }
    }

    pub fn list_devices() -> Result<Vec<String>, String> {
        let host = cpal::default_host();
        let devices = host
            .input_devices()
            .map_err(|e| format!("Failed to enumerate devices: {}", e))?;

        let names: Vec<String> = devices
            .filter_map(|d| d.name().ok())
            .collect();

        Ok(names)
    }

    pub fn start(
        &mut self,
        device_name: Option<&str>,
        sender: mpsc::Sender<Vec<f32>>,
    ) -> Result<(), String> {
        let host = cpal::default_host();

        let device = match device_name {
            Some(name) => host
                .input_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .ok_or_else(|| format!("Audio device not found: {}", name))?,
            None => host
                .default_input_device()
                .ok_or("No default input device found")?,
        };

        let config = StreamConfig {
            channels: 1,
            sample_rate: SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    sender.send(data.to_vec()).ok();
                },
                |err| eprintln!("[audio] Stream error: {}", err),
                None,
            )
            .map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {}", e))?;

        self.stream = Some(stream);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.stream = None; // Drop the stream, which stops capture
    }

    pub fn is_active(&self) -> bool {
        self.stream.is_some()
    }
}
```

### COMPLETION CHECKLIST

- [ ] `AudioCapture::list_devices()` returns available input devices
- [ ] `AudioCapture::start()` begins capturing 16kHz mono audio
- [ ] `AudioCapture::stop()` cleanly stops the stream
- [ ] Audio samples are sent via `mpsc::Sender<Vec<f32>>`
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Audio capture compiles"
```

### COMMIT

```
feat(audio): implement microphone capture with cpal at 16kHz mono
```

### NEXT

→ B.2: Ring buffer for audio chunks

---

## Phase B.2: Audio Ring Buffer

### GOAL

Implement a fixed-size circular buffer that accumulates audio samples and produces overlapping chunks for transcription.

### TASKS

**Step 1:** Implement `backend/src/audio/buffer.rs`:

```rust
pub struct AudioRingBuffer {
    data: Vec<f32>,
    write_pos: usize,
    capacity: usize,          // Total samples the buffer holds
    chunk_size: usize,         // Samples per transcription chunk
    overlap_size: usize,       // Overlap between consecutive chunks
    samples_since_last: usize, // Samples written since last chunk extraction
}

impl AudioRingBuffer {
    /// Create a new ring buffer.
    /// - sample_rate: e.g., 16000
    /// - chunk_duration_ms: e.g., 3000 (3 seconds)
    /// - overlap_ms: e.g., 500
    /// - buffer_duration_s: total buffer capacity in seconds (e.g., 30)
    pub fn new(
        sample_rate: u32,
        chunk_duration_ms: u32,
        overlap_ms: u32,
        buffer_duration_s: u32,
    ) -> Self {
        let chunk_size = (sample_rate * chunk_duration_ms / 1000) as usize;
        let overlap_size = (sample_rate * overlap_ms / 1000) as usize;
        let capacity = (sample_rate * buffer_duration_s) as usize;

        Self {
            data: vec![0.0; capacity],
            write_pos: 0,
            capacity,
            chunk_size,
            overlap_size,
            samples_since_last: 0,
        }
    }

    /// Write audio samples into the buffer.
    pub fn write(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.data[self.write_pos % self.capacity] = sample;
            self.write_pos += 1;
            self.samples_since_last += 1;
        }
    }

    /// Check if enough samples have accumulated for a new chunk.
    pub fn has_chunk(&self) -> bool {
        self.samples_since_last >= (self.chunk_size - self.overlap_size)
            && self.write_pos >= self.chunk_size
    }

    /// Extract the latest chunk (with overlap from previous chunk).
    pub fn extract_chunk(&mut self) -> Option<Vec<f32>> {
        if !self.has_chunk() {
            return None;
        }

        let start = if self.write_pos >= self.chunk_size {
            self.write_pos - self.chunk_size
        } else {
            return None;
        };

        let mut chunk = Vec::with_capacity(self.chunk_size);
        for i in start..self.write_pos {
            chunk.push(self.data[i % self.capacity]);
        }

        self.samples_since_last = 0;
        Some(chunk)
    }

    /// Reset the buffer.
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.samples_since_last = 0;
        self.data.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_basic() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // Write 3 seconds of silence
        let samples = vec![0.0f32; 48000];
        buf.write(&samples);
        assert!(buf.has_chunk());
        let chunk = buf.extract_chunk().unwrap();
        assert_eq!(chunk.len(), 48000); // 3s * 16kHz
    }

    #[test]
    fn test_buffer_overlap() {
        let mut buf = AudioRingBuffer::new(16000, 3000, 500, 30);
        // Write 3 seconds
        buf.write(&vec![0.0f32; 48000]);
        buf.extract_chunk();
        // Write 2.5 more seconds (chunk_size - overlap)
        buf.write(&vec![0.0f32; 40000]);
        assert!(buf.has_chunk());
    }
}
```

### COMPLETION CHECKLIST

- [ ] Buffer accumulates audio samples
- [ ] `has_chunk()` returns true after enough samples
- [ ] `extract_chunk()` returns overlapping windows
- [ ] Tests pass
- [ ] `cargo check` and `cargo test` pass

### VALIDATION

```bash
cd backend && cargo test audio::buffer && echo "✅ Buffer tests pass"
```

### COMMIT

```
feat(audio): implement ring buffer with overlapping chunk extraction
```

### NEXT

→ B.3: Voice Activity Detection

---

## Phase B.3: Voice Activity Detection (VAD)

### GOAL

Implement energy-based VAD to detect when speech is present and avoid sending silence to Whisper.

### TASKS

**Step 1:** Implement `backend/src/audio/vad.rs`:

```rust
pub struct VoiceActivityDetector {
    threshold: f32,
    min_speech_frames: usize,
    min_silence_frames: usize,
    speech_frame_count: usize,
    silence_frame_count: usize,
    is_speech: bool,
}

impl VoiceActivityDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            min_speech_frames: 3,    // Require 3 consecutive voiced frames to trigger
            min_silence_frames: 10,  // Require 10 silent frames to end speech
            speech_frame_count: 0,
            silence_frame_count: 0,
            is_speech: false,
        }
    }

    /// Calculate RMS energy of an audio frame.
    fn rms_energy(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    /// Process a frame of audio and return whether speech is detected.
    /// Frame should be ~20-30ms of audio (320-480 samples at 16kHz).
    pub fn process_frame(&mut self, frame: &[f32]) -> bool {
        let energy = Self::rms_energy(frame);

        if energy > self.threshold {
            self.speech_frame_count += 1;
            self.silence_frame_count = 0;

            if self.speech_frame_count >= self.min_speech_frames {
                self.is_speech = true;
            }
        } else {
            self.silence_frame_count += 1;
            self.speech_frame_count = 0;

            if self.silence_frame_count >= self.min_silence_frames {
                self.is_speech = false;
            }
        }

        self.is_speech
    }

    /// Check if audio chunk contains speech (bulk check).
    pub fn contains_speech(&mut self, audio: &[f32]) -> bool {
        let frame_size = 480; // 30ms at 16kHz
        let mut any_speech = false;

        for frame in audio.chunks(frame_size) {
            if self.process_frame(frame) {
                any_speech = true;
            }
        }

        any_speech
    }

    pub fn reset(&mut self) {
        self.speech_frame_count = 0;
        self.silence_frame_count = 0;
        self.is_speech = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detected() {
        let mut vad = VoiceActivityDetector::new(0.01);
        let silence = vec![0.0f32; 480];
        assert!(!vad.process_frame(&silence));
    }

    #[test]
    fn test_speech_detected() {
        let mut vad = VoiceActivityDetector::new(0.01);
        // Simulate speech with higher energy
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        // Need min_speech_frames consecutive
        vad.process_frame(&speech);
        vad.process_frame(&speech);
        assert!(vad.process_frame(&speech));
    }
}
```

### COMPLETION CHECKLIST

- [ ] VAD detects speech based on RMS energy threshold
- [ ] Hysteresis prevents rapid toggling (min speech/silence frames)
- [ ] `contains_speech()` works on full audio chunks
- [ ] Tests pass

### VALIDATION

```bash
cd backend && cargo test audio::vad && echo "✅ VAD tests pass"
```

### COMMIT

```
feat(audio): implement energy-based voice activity detection
```

### NEXT

→ B.4: Audio pipeline integration

---

## Phase B.4: Audio Pipeline Integration

### GOAL

Wire together capture → buffer → VAD → chunk extraction into a single audio pipeline that runs on a dedicated thread and sends speech chunks via channel.

### TASKS

**Step 1:** Update `backend/src/audio/mod.rs` to expose a high-level pipeline:

```rust
pub mod buffer;
pub mod capture;
pub mod vad;

use buffer::AudioRingBuffer;
use capture::AudioCapture;
use vad::VoiceActivityDetector;
use std::sync::mpsc;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct AudioPipeline {
    is_running: Arc<AtomicBool>,
}

impl AudioPipeline {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the audio pipeline. Returns a receiver that yields speech audio chunks.
    pub fn start(
        &self,
        device_name: Option<String>,
        vad_threshold: f32,
        chunk_duration_ms: u32,
        overlap_ms: u32,
    ) -> Result<mpsc::Receiver<Vec<f32>>, String> {
        let is_running = self.is_running.clone();
        is_running.store(true, Ordering::SeqCst);

        let (chunk_tx, chunk_rx) = mpsc::channel::<Vec<f32>>();
        let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();

        // Start mic capture
        let mut capture = AudioCapture::new();
        capture.start(device_name.as_deref(), audio_tx)?;

        // Spawn processing thread
        let running = is_running.clone();
        std::thread::spawn(move || {
            let mut buffer = AudioRingBuffer::new(16000, chunk_duration_ms, overlap_ms, 30);
            let mut vad = VoiceActivityDetector::new(vad_threshold);

            // Keep capture alive by moving it into this thread
            let _capture = capture;

            while running.load(Ordering::SeqCst) {
                match audio_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                    Ok(samples) => {
                        buffer.write(&samples);

                        if buffer.has_chunk() {
                            if let Some(chunk) = buffer.extract_chunk() {
                                if vad.contains_speech(&chunk) {
                                    if chunk_tx.send(chunk).is_err() {
                                        break; // Receiver dropped
                                    }
                                }
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok(chunk_rx)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }
}
```

### COMPLETION CHECKLIST

- [ ] `AudioPipeline::start()` returns a receiver of speech chunks
- [ ] Pipeline runs on dedicated thread, doesn't block main
- [ ] VAD filters out silence chunks
- [ ] `AudioPipeline::stop()` cleanly shuts down
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Audio pipeline compiles"
```

### COMMIT

```
feat(audio): wire capture → buffer → VAD into unified audio pipeline
```

### NEXT

→ B.5: Audio device listing command

---

## Phase B.5: Audio Device Listing Command

### GOAL

Expose audio device enumeration to the frontend via Tauri command.

### TASKS

**Step 1:** Implement `backend/src/commands/system.rs`:

```rust
use crate::audio::capture::AudioCapture;

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>, String> {
    AudioCapture::list_devices()
}

#[tauri::command]
pub fn get_gpu_info() -> Result<serde_json::Value, String> {
    // Basic GPU detection — parse nvidia-smi output
    let output = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=name,memory.total", "--format=csv,noheader,nounits"])
        .output()
        .map_err(|_| "nvidia-smi not found — CUDA may not be available".to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(", ").collect();

    Ok(serde_json::json!({
        "name": parts.first().unwrap_or(&"Unknown"),
        "vram_total_mb": parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0),
        "cuda_available": output.status.success(),
    }))
}
```

**Step 2:** Update `backend/src/commands/mod.rs`:

```rust
pub mod config;
pub mod dictation;
pub mod models;
pub mod system;
```

### COMPLETION CHECKLIST

- [ ] `list_audio_devices` returns available input devices
- [ ] `get_gpu_info` detects RTX 5060 Ti and reports 16GB VRAM
- [ ] Both commands callable via Tauri IPC

### VALIDATION

```bash
cd backend && cargo check && echo "✅ System commands compile"
```

### COMMIT

```
feat(system): add audio device listing and GPU detection commands
```

### NEXT

→ C.1: Whisper-rs model loading

---

# Section C: Transcription Engine

**Estimated time: 45-60 min | Phases C.1 – C.4**

---

## Phase C.1: Whisper Model Loading

### GOAL

Implement model loading with whisper-rs, including CUDA GPU acceleration and model hot-swapping.

### TASKS

**Step 1:** Implement `backend/src/transcription/engine.rs`:

```rust
use std::path::Path;
use std::sync::Mutex;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct TranscriptionEngine {
    ctx: Mutex<Option<WhisperContext>>,
    active_model: Mutex<Option<String>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start: i64,
    pub end: i64,
}

impl TranscriptionEngine {
    pub fn new() -> Self {
        Self {
            ctx: Mutex::new(None),
            active_model: Mutex::new(None),
        }
    }

    pub fn load_model(&self, model_path: &Path, model_id: &str) -> Result<(), String> {
        // Drop existing context first to free VRAM
        {
            let mut ctx = self.ctx.lock().map_err(|e| e.to_string())?;
            *ctx = None;
        }

        let mut params = WhisperContextParameters::default();
        params.use_gpu(true);

        let new_ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or("Invalid model path")?,
            params,
        )
        .map_err(|e| format!("Failed to load whisper model '{}': {}", model_id, e))?;

        {
            let mut ctx = self.ctx.lock().map_err(|e| e.to_string())?;
            *ctx = Some(new_ctx);
        }
        {
            let mut active = self.active_model.lock().map_err(|e| e.to_string())?;
            *active = Some(model_id.to_string());
        }

        Ok(())
    }

    pub fn unload_model(&self) -> Result<(), String> {
        let mut ctx = self.ctx.lock().map_err(|e| e.to_string())?;
        *ctx = None;
        let mut active = self.active_model.lock().map_err(|e| e.to_string())?;
        *active = None;
        Ok(())
    }

    pub fn get_active_model(&self) -> Option<String> {
        self.active_model.lock().ok().and_then(|m| m.clone())
    }

    pub fn is_loaded(&self) -> bool {
        self.ctx.lock().map(|c| c.is_some()).unwrap_or(false)
    }

    pub fn transcribe(
        &self,
        audio_data: &[f32],
        language: &str,
    ) -> Result<Vec<TranscriptionSegment>, String> {
        let ctx_guard = self.ctx.lock().map_err(|e| e.to_string())?;
        let ctx = ctx_guard
            .as_ref()
            .ok_or("No model loaded. Load a model before transcribing.")?;

        let mut state = ctx.create_state().map_err(|e| format!("Failed to create state: {}", e))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        if language != "auto" {
            params.set_language(Some(language));
        }

        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_non_speech_tokens(true);
        params.set_no_context(true); // Each chunk is independent

        state
            .full(params, audio_data)
            .map_err(|e| format!("Transcription failed: {}", e))?;

        let num_segments = state
            .full_n_segments()
            .map_err(|e| format!("Failed to get segments: {}", e))?;

        let mut segments = Vec::new();
        for i in 0..num_segments {
            let text = state
                .full_get_segment_text(i)
                .map_err(|e| format!("Failed to get segment text: {}", e))?;

            // Skip empty or whitespace-only segments
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                continue;
            }

            let start = state.full_get_segment_t0(i).map_err(|e| e.to_string())?;
            let end = state.full_get_segment_t1(i).map_err(|e| e.to_string())?;

            segments.push(TranscriptionSegment {
                text: trimmed,
                start,
                end,
            });
        }

        Ok(segments)
    }
}
```

### COMPLETION CHECKLIST

- [ ] `load_model()` loads a GGML model with CUDA enabled
- [ ] `unload_model()` frees VRAM before loading a new model
- [ ] `transcribe()` processes f32 audio and returns text segments
- [ ] Empty/whitespace segments are filtered out
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Transcription engine compiles"
```

### COMMIT

```
feat(transcription): implement whisper-rs engine with CUDA and model hot-swap
```

### NEXT

→ C.2: Model download manager

---

## Phase C.2: Model Download Manager

### GOAL

Download Whisper GGML models from HuggingFace with progress events streamed to the frontend.

### TASKS

**Step 1:** Implement `backend/src/model_manager/download.rs`:

```rust
use crate::config::Config;
use crate::transcription::models::get_model_registry;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

pub async fn download_model(model_id: &str, app_handle: &AppHandle) -> Result<PathBuf, String> {
    let registry = get_model_registry();
    let model = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let dest = Config::models_dir().join(&model.filename);

    // Skip if already downloaded
    if dest.exists() {
        let metadata = std::fs::metadata(&dest).map_err(|e| e.to_string())?;
        if metadata.len() > 0 {
            return Ok(dest);
        }
    }

    let client = Client::new();
    let response = client
        .get(&model.url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total = response.content_length().unwrap_or(model.size_bytes);
    let mut downloaded: u64 = 0;

    // Write to temp file first, rename on completion
    let temp_path = dest.with_extension("bin.tmp");
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;

        app_handle
            .emit(
                "download-progress",
                serde_json::json!({
                    "model_id": model_id,
                    "percent": (downloaded as f64 / total as f64) * 100.0,
                    "downloaded_bytes": downloaded,
                    "total_bytes": total,
                }),
            )
            .ok();
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    // Rename temp to final
    tokio::fs::rename(&temp_path, &dest)
        .await
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    Ok(dest)
}

pub fn delete_model(model_id: &str) -> Result<(), String> {
    let registry = get_model_registry();
    let model = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let path = Config::models_dir().join(&model.filename);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete model: {}", e))?;
    }
    Ok(())
}

pub fn is_model_downloaded(model_id: &str) -> bool {
    let registry = get_model_registry();
    registry
        .iter()
        .find(|m| m.id == model_id)
        .map(|m| Config::models_dir().join(&m.filename).exists())
        .unwrap_or(false)
}
```

**Step 2:** Update `backend/src/model_manager/mod.rs`:

```rust
pub mod download;
pub mod storage;
pub use download::{delete_model, download_model, is_model_downloaded};
```

### COMPLETION CHECKLIST

- [ ] Downloads GGML model from HuggingFace
- [ ] Streams download-progress events to frontend
- [ ] Uses temp file + rename (atomic write)
- [ ] `delete_model()` removes model file
- [ ] `is_model_downloaded()` checks existence
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Download manager compiles"
```

### COMMIT

```
feat(models): implement model download with progress streaming and atomic writes
```

### NEXT

→ C.3: Output manager (keyboard + clipboard)

---

## Phase C.3: Output Manager (Keyboard + Clipboard)

### GOAL

Type transcribed text into the active application field and/or copy to clipboard.

### TASKS

**Step 1:** Implement `backend/src/output/keyboard.rs`:

```rust
use enigo::{Enigo, Keyboard, Settings};

pub fn type_text(text: &str) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to init keyboard simulator: {}", e))?;

    enigo
        .text(text)
        .map_err(|e| format!("Failed to type text: {}", e))?;

    Ok(())
}
```

**Step 2:** Implement `backend/src/output/clipboard.rs`:

```rust
use arboard::Clipboard;

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let mut clipboard =
        Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;

    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    Ok(())
}
```

**Step 3:** Update `backend/src/output/mod.rs`:

```rust
pub mod clipboard;
pub mod keyboard;

use crate::config::OutputMode;

pub fn output_text(text: &str, mode: &OutputMode) -> Result<(), String> {
    match mode {
        OutputMode::TypeIntoField => keyboard::type_text(text),
        OutputMode::Clipboard => clipboard::copy_to_clipboard(text),
        OutputMode::Both => {
            keyboard::type_text(text)?;
            clipboard::copy_to_clipboard(text)?;
            Ok(())
        }
    }
}
```

### COMPLETION CHECKLIST

- [ ] `type_text()` simulates keystrokes via enigo
- [ ] `copy_to_clipboard()` copies text via arboard
- [ ] `output_text()` dispatches based on OutputMode
- [ ] Works on Wayland (CachyOS default) — enigo Wayland feature enabled
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Output manager compiles"
```

### COMMIT

```
feat(output): implement keyboard simulation and clipboard output
```

### NEXT

→ C.4: Tauri commands for dictation, models, and config

---

## Phase C.4: Wire All Tauri Commands

### GOAL

Implement all remaining Tauri commands and register them in `main.rs`, connecting the full backend pipeline.

### TASKS

**Step 1:** Implement `backend/src/commands/dictation.rs`:

```rust
use crate::audio::AudioPipeline;
use crate::config::Config;
use crate::output;
use crate::transcription::engine::TranscriptionEngine;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex as TokioMutex;

pub struct AppState {
    pub config: std::sync::Mutex<Config>,
    pub transcription: Arc<TranscriptionEngine>,
    pub audio_pipeline: Arc<std::sync::Mutex<AudioPipeline>>,
    pub dictation_handle: TokioMutex<Option<tokio::task::JoinHandle<()>>>,
}

#[tauri::command]
pub async fn toggle_dictation(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    let pipeline = state.audio_pipeline.lock().map_err(|e| e.to_string())?;

    if pipeline.is_running() {
        pipeline.stop();

        // Cancel the processing task
        let mut handle = state.dictation_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }

        app_handle.emit("dictation-status", "idle").ok();
        Ok(false)
    } else {
        let config = state.config.lock().map_err(|e| e.to_string())?.clone();

        let chunk_rx = pipeline.start(
            config.audio_device.clone(),
            config.vad_threshold,
            config.chunk_duration_ms,
            config.overlap_ms,
        )?;

        let engine = state.transcription.clone();
        let output_mode = config.output_mode.clone();
        let language = config.language.clone();
        let app = app_handle.clone();

        // Spawn async task to process chunks
        let handle = tokio::spawn(async move {
            loop {
                match tokio::task::spawn_blocking({
                    let rx = unsafe {
                        // We need to move the receiver into the blocking task
                        // This is safe because we only access it from one thread
                        &*(&chunk_rx as *const _)
                    };
                    move || rx.recv_timeout(std::time::Duration::from_millis(200))
                })
                .await
                {
                    Ok(Ok(audio_chunk)) => {
                        let engine = engine.clone();
                        let language = language.clone();
                        let output_mode = output_mode.clone();
                        let app = app.clone();

                        // Transcribe on blocking thread
                        if let Ok(segments) = tokio::task::spawn_blocking(move || {
                            engine.transcribe(&audio_chunk, &language)
                        })
                        .await
                        .unwrap_or(Err("Task panicked".to_string()))
                        {
                            for segment in &segments {
                                // Output the text
                                output::output_text(&segment.text, &output_mode).ok();

                                // Send to frontend
                                app.emit(
                                    "transcription-update",
                                    serde_json::json!({
                                        "text": segment.text,
                                        "is_partial": false,
                                    }),
                                )
                                .ok();
                            }
                        }
                    }
                    Ok(Err(_)) => continue, // Timeout, check if still running
                    Err(_) => break,        // Channel disconnected
                }
            }
        });

        let mut dictation_handle = state.dictation_handle.lock().await;
        *dictation_handle = Some(handle);

        app_handle.emit("dictation-status", "listening").ok();
        Ok(true)
    }
}

#[tauri::command]
pub async fn start_dictation(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    toggle_dictation(state, app_handle).await.map(|_| ())
}

#[tauri::command]
pub async fn stop_dictation(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let pipeline = state.audio_pipeline.lock().map_err(|e| e.to_string())?;
    if pipeline.is_running() {
        pipeline.stop();
        let mut handle = state.dictation_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
        }
        app_handle.emit("dictation-status", "idle").ok();
    }
    Ok(())
}
```

**Step 2:** Implement `backend/src/commands/models.rs`:

```rust
use crate::commands::dictation::AppState;
use crate::config::Config;
use crate::model_manager;
use crate::transcription::{get_model_registry, WhisperModel};
use tauri::{AppHandle, State};

#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub vram_mb: u16,
    pub downloaded: bool,
}

impl From<&WhisperModel> for ModelInfo {
    fn from(m: &WhisperModel) -> Self {
        Self {
            id: m.id.clone(),
            display_name: m.display_name.clone(),
            filename: m.filename.clone(),
            size_bytes: m.size_bytes,
            vram_mb: m.vram_mb,
            downloaded: model_manager::is_model_downloaded(&m.id),
        }
    }
}

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    get_model_registry().iter().map(ModelInfo::from).collect()
}

#[tauri::command]
pub async fn download_model_cmd(
    model_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    model_manager::download_model(&model_id, &app_handle).await?;

    // Update config
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if !config.downloaded_models.contains(&model_id) {
        config.downloaded_models.push(model_id);
        config.save()?;
    }

    Ok(())
}

#[tauri::command]
pub fn delete_model_cmd(model_id: String, state: State<'_, AppState>) -> Result<(), String> {
    model_manager::delete_model(&model_id)?;

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.downloaded_models.retain(|m| m != &model_id);
    config.save()?;

    Ok(())
}

#[tauri::command]
pub async fn load_model_cmd(
    model_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    app_handle.emit("dictation-status", "loading").ok();

    let registry = get_model_registry();
    let model = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let model_path = Config::models_dir().join(&model.filename);
    if !model_path.exists() {
        return Err(format!("Model not downloaded: {}", model_id));
    }

    let engine = state.transcription.clone();
    let mid = model_id.clone();

    tokio::task::spawn_blocking(move || engine.load_model(&model_path, &mid))
        .await
        .map_err(|e| e.to_string())??;

    // Update default model in config
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.default_model = model_id;
    config.save()?;

    app_handle.emit("dictation-status", "idle").ok();
    Ok(())
}

#[tauri::command]
pub fn get_active_model(state: State<'_, AppState>) -> Option<String> {
    state.transcription.get_active_model()
}
```

**Step 3:** Implement `backend/src/commands/config.rs`:

```rust
use crate::commands::dictation::AppState;
use crate::config::Config;
use tauri::State;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn update_config(config: Config, state: State<'_, AppState>) -> Result<(), String> {
    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config;
    current.save()?;
    Ok(())
}
```

**Step 4:** Update `backend/src/main.rs`:

```rust
mod audio;
mod commands;
mod config;
mod model_manager;
mod output;
mod transcription;

use commands::dictation::AppState;
use config::Config;
use std::sync::{Arc, Mutex};
use transcription::engine::TranscriptionEngine;

fn main() {
    let config = Config::load().expect("Failed to load config");

    let app_state = AppState {
        config: Mutex::new(config),
        transcription: Arc::new(TranscriptionEngine::new()),
        audio_pipeline: Arc::new(Mutex::new(audio::AudioPipeline::new())),
        dictation_handle: tokio::sync::Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::dictation::toggle_dictation,
            commands::dictation::start_dictation,
            commands::dictation::stop_dictation,
            commands::models::list_models,
            commands::models::download_model_cmd,
            commands::models::delete_model_cmd,
            commands::models::load_model_cmd,
            commands::models::get_active_model,
            commands::config::get_config,
            commands::config::update_config,
            commands::system::list_audio_devices,
            commands::system::get_gpu_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### COMPLETION CHECKLIST

- [ ] All Tauri commands registered in `main.rs`
- [ ] `AppState` is managed and accessible from all commands
- [ ] Dictation toggle starts/stops the full pipeline
- [ ] Model commands handle download, load, delete, list
- [ ] Config commands handle get/update with persistence
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ All Tauri commands compile"
```

### COMMIT

```
feat(commands): wire all Tauri IPC commands — dictation, models, config, system
```

### NEXT

→ D.1: Global hotkey registration

---

# Section D: Hotkey System

**Estimated time: 15-20 min | Phases D.1 – D.2**

---

## Phase D.1: Global Hotkey Registration

### GOAL

Register a system-wide hotkey (default: Ctrl+Shift+Space) that toggles dictation from anywhere.

### TASKS

**Step 1:** Implement `backend/src/hotkey/manager.rs`:

```rust
// Hotkey is handled via tauri-plugin-global-shortcut
// Configuration happens in main.rs setup hook
```

**Step 2:** Update `backend/src/main.rs` to register the global shortcut in the setup hook:

```rust
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn main() {
    let config = Config::load().expect("Failed to load config");
    let default_hotkey = config.hotkey.clone();

    let app_state = AppState {
        config: Mutex::new(config),
        transcription: Arc::new(TranscriptionEngine::new()),
        audio_pipeline: Arc::new(Mutex::new(audio::AudioPipeline::new())),
        dictation_handle: tokio::sync::Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let state = app.state::<AppState>();
                            let _ =
                                commands::dictation::toggle_dictation(state, app.clone()).await;
                        });
                    }
                })
                .build(),
        )
        .manage(app_state)
        .setup(|app| {
            // Register default hotkey (Ctrl+Shift+Space)
            let shortcut = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::SHIFT),
                Code::Space,
            );
            app.global_shortcut().register(shortcut)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::dictation::toggle_dictation,
            commands::dictation::start_dictation,
            commands::dictation::stop_dictation,
            commands::models::list_models,
            commands::models::download_model_cmd,
            commands::models::delete_model_cmd,
            commands::models::load_model_cmd,
            commands::models::get_active_model,
            commands::config::get_config,
            commands::config::update_config,
            commands::system::list_audio_devices,
            commands::system::get_gpu_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### COMPLETION CHECKLIST

- [ ] Ctrl+Shift+Space toggles dictation from any application
- [ ] Hotkey registered on app startup
- [ ] `cargo check` passes

### VALIDATION

```bash
cd backend && cargo check && echo "✅ Global hotkey compiles"
```

### COMMIT

```
feat(hotkey): register global Ctrl+Shift+Space shortcut for dictation toggle
```

### NEXT

→ E.1: Main window frontend UI

---

# Section E: Frontend UI

**Estimated time: 60-90 min | Phases E.1 – E.8**

---

## Phase E.1: Main Window Layout

### GOAL

Build the main app window with the WhisperType dark design system — status indicator, model dropdown, transcript display, and action buttons.

### TASKS

**Step 1:** Implement `src/pages/main-window.tsx`:

```tsx
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
  const { isListening, status, toggle } = useDictation();
  const { transcript, clear } = useTranscription();
  const { models, activeModel, loadModel } = useModels();
  const { config } = useConfig();

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
```

### COMPLETION CHECKLIST

- [ ] Main window renders with rich charcoal background (#0f0f11)
- [ ] Layout: header → model selector → transcript → status/actions
- [ ] All content centered where appropriate
- [ ] Settings gear icon in header
- [ ] `npm run build` succeeds

### VALIDATION

```bash
npm run build && echo "✅ Main window builds"
```

### COMMIT

```
feat(ui): implement main window layout with dark design system
```

### NEXT

→ E.2: Status indicator component

---

## Phase E.2: Status Indicator Component

### GOAL

Animated status dot with label showing current dictation state.

### TASKS

Implement `src/components/status-indicator.tsx`:

```tsx
interface StatusIndicatorProps {
  status: "idle" | "listening" | "loading" | "error";
  hotkey: string;
}

const statusConfig = {
  idle: { color: "bg-zinc-600", label: "Idle", pulse: false },
  listening: { color: "bg-emerald-500", label: "Listening", pulse: true },
  loading: { color: "bg-amber-500", label: "Loading Model...", pulse: true },
  error: { color: "bg-red-500", label: "Error", pulse: false },
} as const;

export function StatusIndicator({ status, hotkey }: StatusIndicatorProps) {
  const config = statusConfig[status];

  return (
    <div className="flex items-center gap-3">
      <div className="flex items-center gap-2">
        <div
          className={`w-2 h-2 rounded-full ${config.color} ${config.pulse ? "animate-pulse" : ""}`}
        />
        <span className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
          {config.label}
        </span>
      </div>
      <span className="text-xs text-zinc-600 font-mono">[{hotkey}]</span>
    </div>
  );
}
```

### COMMIT

```
feat(ui): add animated status indicator component
```

---

## Phase E.3: Model Selector Dropdown

### GOAL

Dropdown showing all Whisper models with download status and VRAM info.

### TASKS

Implement `src/components/model-selector.tsx`:

```tsx
import type { ModelInfo } from "@/lib/tauri";

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
        value={activeModel ?? ""}
        onChange={(e) => onSelect(e.target.value)}
        className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2.5 text-sm text-foreground focus:ring-1 focus:ring-primary focus:border-primary outline-none appearance-none cursor-pointer text-center"
      >
        <option value="" disabled>
          Select a model...
        </option>
        {models.map((model) => (
          <option
            key={model.id}
            value={model.id}
            disabled={!model.downloaded}
          >
            {model.display_name}
            {!model.downloaded ? " (not downloaded)" : ""}
            {model.id === activeModel ? " ●" : ""}
          </option>
        ))}
      </select>
    </div>
  );
}
```

### COMMIT

```
feat(ui): add model selector dropdown with download status
```

---

## Phase E.4: Transcript Display

### GOAL

Scrollable monospace text area that shows live transcription, auto-scrolls to bottom.

### TASKS

Implement `src/components/transcript-display.tsx`:

```tsx
import { useEffect, useRef } from "react";

interface TranscriptDisplayProps {
  transcript: string;
}

export function TranscriptDisplay({ transcript }: TranscriptDisplayProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [transcript]);

  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-1 h-full flex flex-col">
      <h3 className="text-xs font-medium uppercase tracking-wider text-muted-foreground text-center py-3">
        Transcript
      </h3>
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto px-4 pb-4 font-mono text-sm text-foreground/90 leading-relaxed"
      >
        {transcript ? (
          <p className="whitespace-pre-wrap">{transcript}</p>
        ) : (
          <p className="text-zinc-600 text-center italic mt-8">
            Press {"{hotkey}"} or click to start dictating...
          </p>
        )}
      </div>
    </div>
  );
}
```

### COMMIT

```
feat(ui): add auto-scrolling transcript display
```

---

## Phase E.5: React Hooks — useDictation

### GOAL

Hook that manages dictation state, listens to status events, and provides toggle control.

### TASKS

Implement `src/hooks/use-dictation.ts`:

```ts
import { useCallback, useEffect, useState } from "react";
import { commands, events } from "@/lib/tauri";

type DictationStatus = "idle" | "listening" | "loading" | "error";

export function useDictation() {
  const [isListening, setIsListening] = useState(false);
  const [status, setStatus] = useState<DictationStatus>("idle");

  useEffect(() => {
    const unlisten = events.onDictationStatus((newStatus) => {
      setStatus(newStatus as DictationStatus);
      setIsListening(newStatus === "listening");
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const toggle = useCallback(async () => {
    try {
      const result = await commands.toggleDictation();
      setIsListening(result);
    } catch (err) {
      console.error("Failed to toggle dictation:", err);
      setStatus("error");
    }
  }, []);

  return { isListening, status, toggle };
}
```

### COMMIT

```
feat(hooks): implement useDictation hook with event listening
```

---

## Phase E.6: React Hooks — useTranscription

### GOAL

Hook that accumulates transcription text from Tauri events.

### TASKS

Implement `src/hooks/use-transcription.ts`:

```ts
import { useCallback, useEffect, useState } from "react";
import { events } from "@/lib/tauri";

export function useTranscription() {
  const [transcript, setTranscript] = useState("");

  useEffect(() => {
    const unlisten = events.onTranscription((data) => {
      setTranscript((prev) => {
        const separator = prev && !prev.endsWith(" ") ? " " : "";
        return prev + separator + data.text;
      });
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const clear = useCallback(() => {
    setTranscript("");
  }, []);

  return { transcript, clear };
}
```

### COMMIT

```
feat(hooks): implement useTranscription hook with live text accumulation
```

---

## Phase E.7: React Hooks — useModels + useConfig

### GOAL

Hooks for model management and app configuration.

### TASKS

Implement `src/hooks/use-models.ts`:

```ts
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
        refresh(); // Refresh model list when download completes
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
```

Implement `src/hooks/use-config.ts`:

```ts
import { useCallback, useEffect, useState } from "react";
import { commands, type Config } from "@/lib/tauri";

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
      console.error("Failed to update config:", err);
    }
  }, []);

  return { config, updateConfig };
}
```

### COMMIT

```
feat(hooks): implement useModels and useConfig hooks
```

---

## Phase E.8: Settings Panel

### GOAL

Overlay panel for configuring hotkey, output mode, audio device, and language.

### TASKS

Implement `src/components/settings-panel.tsx`:

```tsx
import { useEffect, useState } from "react";
import { useConfig } from "@/hooks/use-config";
import { commands, type Config } from "@/lib/tauri";

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
              setLocalConfig({ ...localConfig, output_mode: e.target.value as Config["output_mode"] })
            }
            className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2 text-sm text-foreground outline-none"
          >
            <option value="both">Type + Clipboard</option>
            <option value="type_into_field">Type into Field</option>
            <option value="clipboard">Clipboard Only</option>
          </select>
        </div>

        {/* Audio Device */}
        <div>
          <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground block mb-2">
            Microphone
          </label>
          <select
            value={localConfig.audio_device ?? ""}
            onChange={(e) =>
              setLocalConfig({
                ...localConfig,
                audio_device: e.target.value || null,
              })
            }
            className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2 text-sm text-foreground outline-none"
          >
            <option value="">System Default</option>
            {audioDevices.map((device) => (
              <option key={device} value={device}>
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
            onChange={(e) =>
              setLocalConfig({ ...localConfig, language: e.target.value })
            }
            className="w-full bg-[#18181b] border border-white/[0.08] rounded-md px-3 py-2 text-sm text-foreground outline-none"
          >
            <option value="auto">Auto Detect</option>
            <option value="en">English</option>
            <option value="es">Spanish</option>
            <option value="fr">French</option>
            <option value="de">German</option>
            <option value="ja">Japanese</option>
            <option value="zh">Chinese</option>
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
```

### COMMIT

```
feat(ui): implement settings panel with output mode, mic, language config
```

### NEXT

→ F.1: First-run setup wizard

---

# Section F: First-Run Setup Wizard

**Estimated time: 30-45 min | Phases F.1 – F.4**

---

## Phase F.1: Wizard Shell + GPU Detection Step

### GOAL

Multi-step setup wizard that appears on first launch. Step 1 detects GPU hardware.

### TASKS

Implement `src/components/setup-wizard/index.tsx`:

```tsx
import { useState } from "react";
import { StepGpu } from "./step-gpu";
import { StepModels } from "./step-models";
import { StepDownload } from "./step-download";
import { StepComplete } from "./step-complete";

interface SetupWizardProps {
  onComplete: () => void;
}

const STEPS = ["gpu", "models", "download", "complete"] as const;
type Step = (typeof STEPS)[number];

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const [step, setStep] = useState<Step>("gpu");
  const [selectedModels, setSelectedModels] = useState<string[]>(["large-v3"]);

  const next = () => {
    const idx = STEPS.indexOf(step);
    if (idx < STEPS.length - 1) {
      setStep(STEPS[idx + 1]);
    }
  };

  return (
    <div className="h-screen flex items-center justify-center bg-[#0f0f11] p-6">
      <div className="w-full max-w-lg">
        {/* Progress */}
        <div className="flex items-center justify-center gap-2 mb-8">
          {STEPS.map((s, i) => (
            <div
              key={s}
              className={`h-1 w-12 rounded-full transition-colors ${
                STEPS.indexOf(step) >= i ? "bg-primary" : "bg-white/[0.08]"
              }`}
            />
          ))}
        </div>

        {step === "gpu" && <StepGpu onNext={next} />}
        {step === "models" && (
          <StepModels
            selected={selectedModels}
            onSelect={setSelectedModels}
            onNext={next}
          />
        )}
        {step === "download" && (
          <StepDownload models={selectedModels} onNext={next} />
        )}
        {step === "complete" && <StepComplete onFinish={onComplete} />}
      </div>
    </div>
  );
}
```

Implement `src/components/setup-wizard/step-gpu.tsx`:

```tsx
import { useEffect, useState } from "react";
import { commands, type GpuInfo } from "@/lib/tauri";

interface StepGpuProps {
  onNext: () => void;
}

export function StepGpu({ onNext }: StepGpuProps) {
  const [gpu, setGpu] = useState<GpuInfo | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    commands
      .getGpuInfo()
      .then((info) => setGpu(info as unknown as GpuInfo))
      .catch(() => setGpu(null))
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 text-center space-y-6">
      <h2 className="text-lg font-semibold text-foreground">
        Welcome to WhisperType
      </h2>
      <p className="text-sm text-muted-foreground">
        Local AI-powered speech-to-text. Everything runs on your machine.
      </p>

      <div className="bg-[#0f0f11] border border-white/[0.06] rounded-lg p-4 space-y-2">
        <h3 className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
          Detected Hardware
        </h3>
        {loading ? (
          <p className="text-sm text-zinc-500">Detecting...</p>
        ) : gpu?.cuda_available ? (
          <>
            <p className="text-sm text-foreground font-medium">{gpu.name}</p>
            <p className="text-xs text-muted-foreground">
              {gpu.vram_total_mb.toLocaleString()} MB VRAM • CUDA Available
            </p>
            <div className="inline-block bg-emerald-500/10 text-emerald-400 text-xs px-2 py-1 rounded mt-1">
              GPU Acceleration Ready
            </div>
          </>
        ) : (
          <>
            <p className="text-sm text-foreground">No NVIDIA GPU detected</p>
            <div className="inline-block bg-amber-500/10 text-amber-400 text-xs px-2 py-1 rounded mt-1">
              CPU Mode (slower transcription)
            </div>
          </>
        )}
      </div>

      <button
        onClick={onNext}
        className="bg-primary hover:bg-primary/90 text-primary-foreground px-6 py-2.5 rounded-md text-sm font-medium transition-colors"
      >
        Continue
      </button>
    </div>
  );
}
```

### COMMIT

```
feat(wizard): implement setup wizard shell and GPU detection step
```

---

## Phase F.2: Model Selection Step

### GOAL

Let user choose which Whisper models to download with size and VRAM info.

### TASKS

Implement `src/components/setup-wizard/step-models.tsx`:

```tsx
import { useEffect, useState } from "react";
import { commands, type ModelInfo } from "@/lib/tauri";

interface StepModelsProps {
  selected: string[];
  onSelect: (ids: string[]) => void;
  onNext: () => void;
}

export function StepModels({ selected, onSelect, onNext }: StepModelsProps) {
  const [models, setModels] = useState<ModelInfo[]>([]);

  useEffect(() => {
    commands.listModels().then(setModels).catch(console.error);
  }, []);

  const toggleModel = (id: string) => {
    if (selected.includes(id)) {
      onSelect(selected.filter((m) => m !== id));
    } else {
      onSelect([...selected, id]);
    }
  };

  const totalSize = models
    .filter((m) => selected.includes(m.id))
    .reduce((acc, m) => acc + m.size_bytes, 0);

  const formatSize = (bytes: number) => {
    if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
    return `${(bytes / 1e6).toFixed(0)} MB`;
  };

  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 space-y-6">
      <div className="text-center">
        <h2 className="text-lg font-semibold text-foreground">Choose Models</h2>
        <p className="text-sm text-muted-foreground mt-1">
          Select which Whisper models to download. You can add more later.
        </p>
      </div>

      <div className="space-y-2">
        {models.map((model) => (
          <button
            key={model.id}
            onClick={() => toggleModel(model.id)}
            className={`w-full flex items-center justify-between p-3 rounded-lg border transition-colors ${
              selected.includes(model.id)
                ? "border-primary/50 bg-primary/5"
                : "border-white/[0.08] hover:bg-white/[0.03]"
            }`}
          >
            <div className="text-left">
              <p className="text-sm font-medium text-foreground">
                {model.display_name}
              </p>
              <p className="text-xs text-muted-foreground">
                ~{model.vram_mb.toLocaleString()} MB VRAM
              </p>
            </div>
            <div
              className={`w-4 h-4 rounded border-2 flex items-center justify-center transition-colors ${
                selected.includes(model.id)
                  ? "border-primary bg-primary"
                  : "border-zinc-600"
              }`}
            >
              {selected.includes(model.id) && (
                <svg className="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                    clipRule="evenodd"
                  />
                </svg>
              )}
            </div>
          </button>
        ))}
      </div>

      <div className="text-center text-xs text-muted-foreground">
        Total download: {formatSize(totalSize)}
      </div>

      <button
        onClick={onNext}
        disabled={selected.length === 0}
        className="w-full bg-primary hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed text-primary-foreground px-6 py-2.5 rounded-md text-sm font-medium transition-colors"
      >
        Download {selected.length} Model{selected.length !== 1 ? "s" : ""}
      </button>
    </div>
  );
}
```

### COMMIT

```
feat(wizard): implement model selection step with size info
```

---

## Phase F.3: Model Download Step

### GOAL

Download selected models with live progress bars.

### TASKS

Implement `src/components/setup-wizard/step-download.tsx`:

```tsx
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
                  {done ? "✓" : active ? `${pct.toFixed(0)}%` : "Waiting..."}
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
```

### COMMIT

```
feat(wizard): implement model download step with progress bars
```

---

## Phase F.4: Completion Step + App Router

### GOAL

Final wizard step and App.tsx router that shows wizard on first launch, main window after.

### TASKS

Implement `src/components/setup-wizard/step-complete.tsx`:

```tsx
interface StepCompleteProps {
  onFinish: () => void;
}

export function StepComplete({ onFinish }: StepCompleteProps) {
  return (
    <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 text-center space-y-6">
      <div className="text-4xl">🎙️</div>
      <h2 className="text-lg font-semibold text-foreground">You're All Set</h2>
      <p className="text-sm text-muted-foreground">
        Press <span className="font-mono text-foreground">Ctrl+Shift+Space</span> anywhere
        to start dictating. WhisperType will transcribe your speech locally — nothing leaves
        your machine.
      </p>
      <button
        onClick={onFinish}
        className="bg-primary hover:bg-primary/90 text-primary-foreground px-6 py-2.5 rounded-md text-sm font-medium transition-colors"
      >
        Start Using WhisperType
      </button>
    </div>
  );
}
```

Update `src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { MainWindow } from "./pages/main-window";
import { SetupWizard } from "./components/setup-wizard";
import { commands, type Config } from "./lib/tauri";

function App() {
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    commands
      .getConfig()
      .then(setConfig)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const handleSetupComplete = async () => {
    if (config) {
      const updated = { ...config, first_run_complete: true };
      await commands.updateConfig(updated);
      setConfig(updated);
    }
  };

  if (loading) {
    return (
      <div className="h-screen bg-[#0f0f11] flex items-center justify-center">
        <div className="text-muted-foreground text-sm">Loading...</div>
      </div>
    );
  }

  if (!config?.first_run_complete) {
    return (
      <div className="dark">
        <SetupWizard onComplete={handleSetupComplete} />
      </div>
    );
  }

  return (
    <div className="dark">
      <MainWindow />
    </div>
  );
}

export default App;
```

### COMPLETION CHECKLIST

- [ ] First launch shows setup wizard
- [ ] Wizard: GPU detect → model select → download → complete
- [ ] After setup, app shows main window
- [ ] Subsequent launches skip wizard
- [ ] `npm run build` succeeds

### VALIDATION

```bash
npm run build && echo "✅ Full frontend builds"
cargo tauri dev  # Visual verification
```

### COMMIT

```
feat(wizard): complete setup wizard with app routing
```

### NEXT

→ G.1: End-to-end integration testing

---

# Section G: Integration & Polish

**Estimated time: 30-45 min | Phases G.1 – G.5**

---

## Phase G.1: End-to-End Pipeline Test

### GOAL

Verify the complete pipeline works: hotkey → mic capture → VAD → whisper → text output.

### TASKS

1. Run `cargo tauri dev`
2. Complete the setup wizard (download at least `tiny` model for fast testing)
3. Load the `tiny` model from the dropdown
4. Press Ctrl+Shift+Space
5. Speak into your microphone
6. Verify text appears in the transcript display
7. Verify text is typed into the previously active field (if output mode includes typing)
8. Press Ctrl+Shift+Space again to stop

**Debug checklist if something fails:**

- Check terminal for Rust errors (`cargo tauri dev` shows backend logs)
- Check browser devtools console for frontend errors
- Verify `nvidia-smi` shows GPU utilization during transcription
- Verify `~/.whispertype/models/` contains the downloaded model file
- Test audio: `arecord -d 3 test.wav && aplay test.wav`

### COMPLETION CHECKLIST

- [ ] Hotkey toggles dictation on/off
- [ ] Status indicator updates (idle ↔ listening)
- [ ] Speech is detected and transcribed
- [ ] Text appears in transcript display
- [ ] Text is output via keyboard/clipboard as configured
- [ ] No crashes or error states

### VALIDATION

```bash
ls ~/.whispertype/config.json && echo "✅ Config exists"
ls ~/.whispertype/models/*.bin && echo "✅ Models downloaded"
nvidia-smi | rg -i "whisper\|tauri" || echo "Check GPU usage manually during transcription"
```

### COMMIT

```
test(e2e): verify complete dictation pipeline end-to-end
```

---

## Phase G.2: Error Handling & Edge Cases

### GOAL

Add graceful error handling for common failure modes.

### TASKS

1. **No microphone**: Show user-friendly error in status indicator
2. **No model loaded**: Prompt user to select a model before dictating
3. **Model download interrupted**: Clean up temp files, allow retry
4. **CUDA unavailable**: Fall back to CPU mode with warning
5. **Hotkey conflict**: Log warning, suggest alternative

Add error boundary to `src/App.tsx`:

```tsx
import { Component, type ReactNode } from "react";

class ErrorBoundary extends Component<
  { children: ReactNode },
  { hasError: boolean; error: string }
> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: "" };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error: error.message };
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-screen bg-[#0f0f11] flex items-center justify-center p-6">
          <div className="bg-[#131316] border border-white/[0.08] rounded-lg p-8 text-center max-w-md">
            <h2 className="text-lg font-semibold text-foreground mb-2">Something went wrong</h2>
            <p className="text-sm text-muted-foreground mb-4">{this.state.error}</p>
            <button
              onClick={() => window.location.reload()}
              className="bg-primary hover:bg-primary/90 text-primary-foreground px-4 py-2 rounded-md text-sm"
            >
              Reload
            </button>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
```

### COMMIT

```
fix(error): add error boundaries and graceful failure handling
```

---

## Phase G.3: Window Configuration (Tauri)

### GOAL

Configure the Tauri window properties — size, title, decorations, always-on-top option.

### TASKS

Update `backend/tauri.conf.json` window configuration:

```json
{
  "app": {
    "windows": [
      {
        "title": "WhisperType",
        "width": 420,
        "height": 600,
        "minWidth": 360,
        "minHeight": 480,
        "resizable": true,
        "decorations": true,
        "transparent": false,
        "center": true
      }
    ]
  }
}
```

### COMMIT

```
feat(window): configure Tauri window size and properties
```

---

## Phase G.4: Auto-Load Default Model on Startup

### GOAL

After first-run setup, automatically load the default model when the app starts.

### TASKS

Add a `setup` hook in `main.rs` that loads the default model after the window is created. Alternatively, handle it on the frontend in `MainWindow` with a `useEffect` that calls `loadModel(config.default_model)` on mount if no model is active.

Update `src/pages/main-window.tsx` to add auto-load:

```tsx
// Add to MainWindow component, after hooks
useEffect(() => {
  if (config && !activeModel) {
    const defaultModel = config.default_model;
    const isDownloaded = models.find(
      (m) => m.id === defaultModel && m.downloaded
    );
    if (isDownloaded) {
      loadModel(defaultModel);
    }
  }
}, [config, models, activeModel, loadModel]);
```

### COMMIT

```
feat(startup): auto-load default Whisper model on app launch
```

---

## Phase G.5: README + Final Polish

### GOAL

Write a comprehensive README and do a final review pass.

### TASKS

Create `README.md` in project root:

```markdown
# 🎙️ WhisperType

Local AI-powered speech-to-text dictation for Linux. Everything runs on your machine — no cloud, no API keys, no data leaving your computer.

Built with **Tauri** (Rust) + **React** (TypeScript) + **whisper.cpp** (CUDA-accelerated).

## Features

- **Real-time dictation** — Speak and text appears in your active application
- **GPU-accelerated** — Uses NVIDIA CUDA for fast transcription
- **Multiple models** — Tiny to Large-v3, switch via dropdown
- **Global hotkey** — Ctrl+Shift+Space toggles from anywhere
- **Privacy-first** — 100% local, zero network after model download
- **Dual output** — Type into active field + copy to clipboard

## Requirements

- **OS:** Linux (developed on CachyOS/Arch)
- **GPU:** NVIDIA GPU with CUDA support (recommended)
- **CUDA Toolkit:** Required for GPU acceleration
- **Node.js:** 18+
- **Rust:** Latest stable

## Installation

### System Dependencies (Arch/CachyOS)

\`\`\`bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget openssl \
  alsa-lib alsa-utils cuda cudnn libappindicator-gtk3 librsvg
\`\`\`

### Build & Run

\`\`\`bash
git clone https://github.com/ponderrr/local-stt.git
cd local-stt
npm install
cargo tauri dev
\`\`\`

### Production Build

\`\`\`bash
cargo tauri build
\`\`\`

## Usage

1. Launch WhisperType
2. Complete first-run setup (GPU detection + model download)
3. Select your preferred model from the dropdown
4. Press **Ctrl+Shift+Space** to start dictating
5. Speak naturally — text appears in real-time
6. Press **Ctrl+Shift+Space** again to stop

## Architecture

- **Backend:** Rust (Tauri) — audio capture, whisper-rs transcription, keyboard simulation
- **Frontend:** React + TypeScript + Tailwind + shadcn/ui
- **Transcription:** whisper.cpp via whisper-rs with CUDA acceleration
- **Audio:** cpal for microphone capture, energy-based VAD

## License

MIT
```

### COMMIT

```
docs(readme): add comprehensive README with setup and usage instructions
```

---

# Phase Summary

| Section | Phases | Description | Est. Time |
|---------|--------|-------------|-----------|
| **A** | A.1 – A.6 | Project scaffolding | 30-45 min |
| **B** | B.1 – B.5 | Audio engine | 45-60 min |
| **C** | C.1 – C.4 | Transcription + commands | 45-60 min |
| **D** | D.1 – D.2 | Hotkey system | 15-20 min |
| **E** | E.1 – E.8 | Frontend UI | 60-90 min |
| **F** | F.1 – F.4 | Setup wizard | 30-45 min |
| **G** | G.1 – G.5 | Integration + polish | 30-45 min |

**Total: ~32 micro-phases | ~4.5-6 hours estimated build time**

---

> *"Big things are built from small perfect pieces."*
> — The Andrew Ponder Methodology
