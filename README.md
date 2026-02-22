<p align="center">
  <h1 align="center">WhisperType</h1>
  <p align="center">
    Local, private speech-to-text for Linux.<br>
    Powered by OpenAI Whisper. Accelerated by CUDA. No cloud required.
  </p>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/platform-Linux-lightgrey.svg" alt="Platform: Linux"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/GPU-CUDA%2012%2B-76B900.svg?logo=nvidia" alt="GPU: CUDA"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/Rust-1.77%2B-DEA584.svg?logo=rust" alt="Rust: 1.77+"></a>
  <a href="#requirements"><img src="https://img.shields.io/badge/Tauri-v2-24C8D8.svg?logo=tauri" alt="Tauri: v2"></a>
</p>

<br>

> Screenshot coming soon

---

## Why WhisperType?

Every major dictation tool sends your voice to someone else's server. WhisperType doesn't. It runs OpenAI's Whisper large-v3 model directly on your NVIDIA GPU, transcribing speech in under 300ms per chunk with zero network calls. Your audio never leaves your machine.

Press **Ctrl+Shift+Space** from any application. Speak. Text appears. Press again to stop. That's it.

## Features

- **Completely local** --- no cloud, no API keys, no telemetry, no subscriptions
- **CUDA-accelerated** --- whisper.cpp with flash attention on your NVIDIA GPU
- **All Whisper models** --- tiny through large-v3, downloadable from the app
- **Global hotkey** --- Ctrl+Shift+Space toggles dictation from anywhere
- **Multiple output modes** --- type into the active field, copy to clipboard, or both
- **PipeWire-native** --- audio capture via `pipewire-pulse` at 48kHz, resampled to 16kHz mono
- **Voice activity detection** --- energy-based VAD filters silence so Whisper only processes speech
- **Multi-language** --- English, Spanish, French, German, Japanese, Chinese, or auto-detect
- **Session-optimized** --- Whisper inference state is created once per session, eliminating repeated CUDA initialization overhead

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | [Tauri v2](https://v2.tauri.app) |
| Backend | Rust |
| Frontend | React 19 + TypeScript + Tailwind CSS |
| Transcription | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) via [whisper-rs](https://github.com/tazz4843/whisper-rs) |
| Audio capture | PulseAudio Simple API (`libpulse`) through PipeWire |
| Text output | [enigo](https://github.com/enigo-rs/enigo) (X11/Wayland) + [arboard](https://github.com/1Password/arboard) (clipboard) |
| Audio pipeline | Lock-free ring buffer ([ringbuf](https://crates.io/crates/ringbuf)) + MPSC channels |

## Requirements

| Requirement | Details |
|-------------|---------|
| **OS** | Linux with PipeWire (X11 or Wayland) |
| **GPU** | NVIDIA with CUDA support (RTX 20-series or newer recommended) |
| **CUDA Toolkit** | 12.0+ with cuDNN |
| **Rust** | 1.77+ via [rustup](https://rustup.rs) |
| **Node.js** | 20+ |
| **CMake** | 3.18+ (for whisper.cpp CUDA build) |

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

CUDA and cuDNN must be installed separately. See the [NVIDIA CUDA installation guide](https://developer.nvidia.com/cuda-downloads).
</details>

## Installation

```bash
git clone https://github.com/ponderrr/local-stt.git
cd local-stt
npm install
```

### Development

```bash
npx tauri dev
```

### Production Build

```bash
npx tauri build
```

**NVIDIA Blackwell GPUs** (RTX 50-series, compute capability 12.0) require:

```bash
CMAKE_CUDA_ARCHITECTURES=120 npx tauri build
```

### CPU-Only Build (no CUDA)

```bash
cd backend
cargo build --no-default-features
```

## Usage

1. **Launch** WhisperType
2. **Select a model** from the dropdown --- large-v3 is recommended for accuracy
3. **Download** the model if it isn't already on disk
4. **Press Ctrl+Shift+Space** to start dictation
5. **Speak naturally** --- text is transcribed in real time
6. **Press Ctrl+Shift+Space** again to stop

Transcribed text is output according to your configured mode: typed directly into the active field, copied to the clipboard, or both. Change this anytime in Settings.

## Configuration

Configuration lives at `~/.whispertype/config.json`. Models are stored in `~/.whispertype/models/`.

| Setting | Default | Description |
|---------|---------|-------------|
| `hotkey` | `Ctrl+Shift+Space` | Global toggle shortcut |
| `output_mode` | `both` | `type_into_field`, `clipboard`, or `both` |
| `audio_device` | `null` | PulseAudio source name (`null` = system default) |
| `language` | `auto` | Language code (`en`, `es`, `fr`, `de`, `ja`, `zh`) or `auto` |
| `default_model` | `large-v3` | Model loaded on startup |
| `vad_threshold` | `0.012` | VAD sensitivity --- lower is more sensitive |
| `chunk_duration_ms` | `2000` | Audio chunk length sent to Whisper |
| `overlap_ms` | `500` | Overlap between consecutive chunks |

## Architecture

WhisperType uses a three-thread pipeline architecture for dictation:

```
Microphone
    |
    v
[pulse-actor]  --- PulseAudio Simple API, 48kHz mono capture
    |               Pushes f32 samples into lock-free ring buffer
    v
[dsp-pipeline] --- Drains ring buffer, resamples 48kHz -> 16kHz
    |               Runs energy-based VAD, extracts speech chunks
    v
[transcription] -- Receives chunks via MPSC channel
                    Runs whisper.cpp inference on GPU
                    Outputs text via enigo / clipboard
```

The Tauri frontend communicates with the Rust backend through IPC commands (`toggle_dictation`, `start_dictation`, `stop_dictation`, `load_model`, etc.) and receives real-time updates through event channels (`dictation-status`, `transcription-update`, `download-progress`).

All three audio threads are spawned on dictation start and cleanly terminated on stop. The pulse-actor receives an explicit `Quit` command, the DSP thread exits via an atomic flag, and the transcription thread exits when its channel closes.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full threading model and design rationale.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, build commands, testing requirements, and PR guidelines.

Quick version:

```bash
git clone https://github.com/ponderrr/local-stt.git
cd local-stt && npm install
npx tauri dev                          # run in dev mode
cd backend && cargo test               # all tests must pass
cd backend && cargo clippy -- -D warnings  # zero warnings required
```

## License

MIT --- see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built by <a href="https://github.com/ponderrr">Andrew Ponder</a>
</p>
