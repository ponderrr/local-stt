<h1 align="center">WhisperType</h1>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/platform-Linux-lightgrey.svg?logo=linux&logoColor=white" alt="Platform: Linux"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/CUDA-12%2B-76B900.svg?logo=nvidia" alt="CUDA 12+"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/Rust-1.77%2B-DEA584.svg?logo=rust" alt="Rust 1.77+"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/Tauri-v2-24C8D8.svg?logo=tauri" alt="Tauri v2"></a>
</p>

<p align="center">
  <strong>Local speech-to-text that runs entirely on your machine. No cloud. No API keys. No telemetry.</strong>
</p>

<p align="center">
  <em>Press Ctrl+Shift+Space. Speak. Text appears at your cursor. Press again to stop.</em>
</p>

---

## Why WhisperType?

Every dictation tool worth using sends your voice to someone else's server. WhisperType doesn't. It runs OpenAI's Whisper large-v3 directly on your NVIDIA GPU via whisper.cpp, transcribing speech in ~70ms per inference pass with zero network calls. Your audio never leaves your machine.

In v0.3.0, WhisperType introduces a dual-path transcription pipeline: Moonshine runs on the CPU to deliver instant streaming previews while you speak, and Whisper runs on the GPU to produce the final high-accuracy transcription when you pause. The result is text that appears as fast as you can say it, with accuracy that matches a dedicated GPU model.

---

## Features

WhisperType is a single-purpose tool: hear speech, produce text, stay out of the way.

| Feature | Details |
|---------|---------|
| **Local-only** | No cloud, no API keys, no telemetry, no subscriptions |
| **Dual-path inference** | Moonshine (CPU) for instant streaming previews + Whisper (GPU) for final accuracy |
| **CUDA-accelerated** | whisper.cpp with flash attention on NVIDIA GPUs |
| **9 models** | 7 Whisper GGML models (tiny through large-v3) + 2 Moonshine ONNX models, all downloadable from the app |
| **Silero VAD** | Neural network voice activity detection with 0.97 ROC-AUC at <1ms per frame |
| **Continuous inference** | LocalAgreement-2 deduplication confirms words across consecutive passes for real-time output |
| **Global hotkey** | `Ctrl+Shift+Space` toggles dictation from any application |
| **Output modes** | Type into the focused field, copy to clipboard, or both |
| **PipeWire-native** | Audio capture via `pipewire-pulse` at 48kHz, resampled to 16kHz mono |
| **Multi-language** | English, Spanish, French, German, Japanese, Chinese, or auto-detect |
| **Session-optimized** | CUDA state allocated once per session --- zero per-chunk initialization overhead |

---

## Requirements

| Requirement | Version | Notes |
|-------------|---------|-------|
| **OS** | Linux | PipeWire with `pipewire-pulse`. X11 or Wayland. |
| **GPU** | NVIDIA RTX 20-series+ | Any GPU with CUDA compute capability 7.0+ |
| **CUDA Toolkit** | 12.0+ | With cuDNN |
| **Rust** | 1.77+ | Via [rustup](https://rustup.rs) |
| **Node.js** | 20+ | For the frontend build |
| **CMake** | 3.18+ | Required by whisper.cpp CUDA build |

### System Packages

<details>
<summary><strong>Arch Linux / CachyOS</strong></summary>

```bash
sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3 librsvg patchelf \
  libpulse pkg-config cmake base-devel cuda cudnn
```
</details>

<details>
<summary><strong>Debian / Ubuntu</strong></summary>

```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
  patchelf libpulse-dev pkg-config cmake build-essential
```

CUDA and cuDNN must be installed separately via the [NVIDIA CUDA Toolkit](https://developer.nvidia.com/cuda-downloads).
</details>

---

## Quick Start

```bash
git clone https://github.com/ponderrr/local-stt.git
cd local-stt
npm install
npx tauri dev
```

Four commands. First launch opens a setup wizard that walks you through model download and GPU detection.

---

## Installation

### Development

```bash
npx tauri dev
```

Starts both the Vite frontend dev server and the Rust backend with hot-reload.

### Production Build

```bash
npx tauri build
```

### NVIDIA Blackwell GPUs (RTX 50-series)

Blackwell cards (compute capability 12.0) need the CUDA architecture specified explicitly:

```bash
CMAKE_CUDA_ARCHITECTURES=120 npx tauri build
```

### CPU-Only Build (no CUDA)

```bash
cd backend
cargo build --no-default-features
```

---

## Usage

1. Launch WhisperType
2. Select a Whisper model from the dropdown --- `distil-large-v3` recommended for speed, `large-v3` for accuracy
3. Download the model if it isn't on disk yet
4. Optionally download a Moonshine model and enable dual-path mode in Settings
5. Press **Ctrl+Shift+Space** to start dictation
6. Speak naturally --- text is transcribed in real time
7. Press **Ctrl+Shift+Space** again to stop

Text output follows your configured mode: typed directly into the focused field via keyboard simulation, copied to the system clipboard, or both.

### Stream Engine Modes

| Mode | How it works |
|------|-------------|
| **Whisper Only** | Each inference pass runs on the GPU via whisper.cpp. Words are confirmed through LocalAgreement-2 and output incrementally. This is the default. |
| **Moonshine + Whisper** | During speech, Moonshine runs on the CPU (~28 MB model) for instant streaming previews. When speech ends, Whisper runs the full utterance on the GPU for final accuracy. Best of both worlds. |

Configure the stream engine in **Settings > Stream Engine**.

---

## Architecture

WhisperType uses a three-thread pipeline with an optional dual-path inference stage. Each thread has a single responsibility and communicates through lock-free or bounded channels.

```mermaid
graph LR
    subgraph Capture
        MIC[Microphone\n48kHz mono] --> PA[pulse-actor\nPulseAudio Simple API]
    end

    subgraph DSP
        PA -->|HeapRb\nlock-free ring buffer| DSP_T[dsp-pipeline\nresample 48→16kHz\nSilero VAD]
    end

    subgraph Inference
        DSP_T -->|mpsc channel\naudio segments| MS[Moonshine\nCPU streaming]
        DSP_T -->|mpsc channel\naudio segments| WH[Whisper\nGPU quality pass]
        MS -->|LocalAgreement-2| AGR[agreement\nword dedup]
        WH --> AGR
    end

    subgraph Output
        AGR --> ENIGO[enigo\nkeyboard sim]
        AGR --> CLIP[arboard\nclipboard]
    end

    style MIC fill:#0f0f11,stroke:#4CA9EF,color:#fff
    style PA fill:#0f0f11,stroke:#4CA9EF,color:#fff
    style DSP_T fill:#0f0f11,stroke:#4CA9EF,color:#fff
    style MS fill:#0f0f11,stroke:#9B59B6,color:#fff
    style WH fill:#0f0f11,stroke:#76B900,color:#fff
    style AGR fill:#0f0f11,stroke:#4CA9EF,color:#fff
    style ENIGO fill:#0f0f11,stroke:#E87722,color:#fff
    style CLIP fill:#0f0f11,stroke:#E87722,color:#fff
```

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Loading : load_model
    Loading --> Idle : model ready
    Idle --> Running : Ctrl+Shift+Space\n(300ms debounce)
    Running --> Idle : Ctrl+Shift+Space\nor stop_dictation
    Running --> Running : speech segment\n→ inference → output

    state Running {
        [*] --> PulseCapture
        PulseCapture --> DSP : ring buffer
        DSP --> DualPath : mpsc channel

        state DualPath {
            [*] --> Streaming : during speech
            Streaming --> QualityPass : EndOfSpeech
            QualityPass --> [*] : finalize + reset
        }
    }

    Idle --> [*] : app exit

    note right of Running
        Moonshine loaded before pipeline.start()
        to prevent ORT session race conditions.
        Silero VAD created inside DSP thread.
        All ORT sessions initialized sequentially.
    end note
```

**Dual-path inference.** During speech, Moonshine (~28 MB ONNX model) runs on the CPU to produce instant streaming text. Words are confirmed through LocalAgreement-2 and output incrementally. When the speaker pauses and Silero VAD fires an EndOfSpeech signal, Whisper runs the full accumulated audio buffer on the GPU for a final accuracy pass. Any remaining tentative words are confirmed and output. This gives sub-200ms perceived latency during speech with Whisper-grade accuracy on the final transcript.

**Why libpulse over cpal?** The cpal crate has a known issue ([cpal#554](https://github.com/RustAudio/cpal/issues/554)) where ALSA's `POLLIN` flag conflicts with PipeWire's graph processing model. Blocking reads via PulseAudio's Simple API sidestep this entirely --- PipeWire's `pipewire-pulse` compatibility layer handles the scheduling.

**Thread-per-stage, not async.** Both whisper-rs and PulseAudio's Simple API are blocking. An async runtime would just park a thread anyway. Three `std::thread` instances with explicit ownership is simpler to reason about and has zero runtime overhead.

**ORT session ordering.** ONNX Runtime's `load-dynamic` strategy shares a single `libonnxruntime.so` across Silero VAD and Moonshine. Creating sessions concurrently from different threads causes `GetElementType` crashes. WhisperType solves this by loading Moonshine before `pipeline.start()` (which creates the Silero VAD session), ensuring sequential initialization.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full component breakdown, concurrency model, and design decision records.

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri v2](https://v2.tauri.app) |
| Backend | Rust (Edition 2021) |
| Frontend | React 19 + TypeScript + Tailwind CSS |
| GPU transcription | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) via [whisper-rs 0.15](https://github.com/tazz4843/whisper-rs) |
| CPU transcription | [Moonshine](https://github.com/usefulsensors/moonshine) via [transcribe-rs 0.2.9](https://crates.io/crates/transcribe-rs) |
| ONNX Runtime | [ort 2.0.0-rc.10](https://crates.io/crates/ort) with `load-dynamic` (shared by Silero VAD + Moonshine) |
| Voice activity detection | [Silero VAD](https://github.com/snakers4/silero-vad) via [silero-vad-rust](https://github.com/sheldonix/silero-vad-rust) |
| Audio capture | PulseAudio Simple API (`libpulse-simple`) through PipeWire |
| Text output | [enigo 0.6](https://github.com/enigo-rs/enigo) (X11 `x11rb` / Wayland) + [arboard 3](https://github.com/1Password/arboard) (clipboard) |
| Audio pipeline | Lock-free ring buffer ([ringbuf 0.4](https://crates.io/crates/ringbuf)) + `std::sync::mpsc` |

---

## Configuration

Configuration lives at `~/.whispertype/config.json`. Models are stored in `~/.whispertype/models/`.

| Setting | Default | Description |
|---------|---------|-------------|
| `hotkey` | `Ctrl+Shift+Space` | Global toggle shortcut |
| `output_mode` | `both` | `type_into_field`, `clipboard`, or `both` |
| `stream_engine` | `whisper_only` | `whisper_only` or `moonshine` (dual-path) |
| `audio_device` | `null` | PulseAudio source name (`null` = system default) |
| `language` | `auto` | `en`, `es`, `fr`, `de`, `ja`, `zh`, or `auto` |
| `default_model` | `distil-large-v3` | Whisper model loaded on startup |
| `vad_backend` | `silero` | `silero` (neural network) or `energy` (volume threshold) |
| `vad_threshold` | `0.012` | Energy VAD RMS threshold --- lower = more sensitive |

---

## Contributing

Contributions welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for full details. Questions or ideas? Open a thread in [GitHub Discussions](https://github.com/ponderrr/local-stt/discussions).

```bash
git clone https://github.com/ponderrr/local-stt.git
cd local-stt && npm install
npx tauri dev                              # run in dev mode
cd backend && cargo test                   # all tests must pass
cd backend && cargo clippy -- -D warnings  # zero warnings
```

---

## License

MIT --- see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built by <a href="https://github.com/ponderrr">Andrew Ponder</a>
</p>
