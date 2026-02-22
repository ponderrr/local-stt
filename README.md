# WhisperType

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform: Linux](https://img.shields.io/badge/Platform-Linux-lightgrey.svg)](#requirements)
[![GPU: CUDA](https://img.shields.io/badge/GPU-CUDA-green.svg)](#requirements)

Local, private speech-to-text for Linux. Runs entirely on your hardware with OpenAI Whisper and CUDA acceleration.

> Screenshot coming soon

## Features

WhisperType turns your voice into text without ever leaving your machine. Audio is captured natively through PipeWire, processed by a multi-threaded Rust pipeline with energy-based voice activity detection, and transcribed by whisper.cpp running on your NVIDIA GPU. There are no cloud calls, no API keys, and no subscriptions. Your voice data never touches a network.

The global hotkey (Ctrl+Shift+Space) works from any application. Speak naturally, and transcribed text is typed directly into the active field, copied to the clipboard, or both. Transcription latency on an RTX-class GPU is typically under 300ms per chunk.

- **Models:** Supports all Whisper model sizes from tiny to large-v3, downloadable from the app
- **Languages:** English, Spanish, French, German, Japanese, Chinese, or auto-detect
- **Output modes:** Type into active field, clipboard, or both
- **Audio:** PipeWire-native capture via pipewire-pulse at 48kHz, resampled to 16kHz mono
- **Performance:** CUDA-accelerated inference with flash attention, session-level state reuse

## Requirements

- **OS:** Linux with PipeWire (X11 or Wayland)
- **GPU:** NVIDIA GPU with CUDA support
- **CUDA Toolkit:** 12.0+ with cuDNN
- **Rust:** 1.77+ (install via [rustup](https://rustup.rs))
- **Node.js:** 20+
- **System packages:**

Arch Linux / CachyOS:
```bash
sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3 librsvg patchelf \
  libpulse pkg-config cmake base-devel cuda cudnn
```

Debian / Ubuntu:
```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
  patchelf libpulse-dev pkg-config cmake build-essential
```

CUDA and cuDNN must be installed separately on Debian/Ubuntu. See the [NVIDIA CUDA installation guide](https://developer.nvidia.com/cuda-downloads).

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

For NVIDIA Blackwell GPUs (RTX 50-series, compute capability 12.0):

```bash
CMAKE_CUDA_ARCHITECTURES=120 npx tauri build
```

## Usage

1. Launch WhisperType
2. Select a model from the dropdown (large-v3 recommended for quality)
3. If the model isn't downloaded yet, click to download it
4. Press **Ctrl+Shift+Space** to start dictation
5. Speak naturally
6. Press **Ctrl+Shift+Space** again to stop

Transcribed text is output according to your configured output mode (type into active field, clipboard, or both). You can change this in Settings.

## Configuration

WhisperType stores its configuration at `~/.whispertype/config.json`. Key settings:

| Setting | Default | Description |
|---------|---------|-------------|
| `hotkey` | `Ctrl+Shift+Space` | Global toggle shortcut |
| `output_mode` | `both` | `type_into_field`, `clipboard`, or `both` |
| `audio_device` | `null` | PulseAudio source name, or `null` for system default |
| `language` | `auto` | Language code (`en`, `es`, `fr`, etc.) or `auto` |
| `vad_threshold` | `0.012` | Voice activity detection sensitivity (lower = more sensitive) |
| `chunk_duration_ms` | `2000` | Audio chunk length sent to Whisper |
| `overlap_ms` | `500` | Overlap between consecutive chunks |
| `default_model` | `large-v3` | Model loaded on startup |

Models are stored in `~/.whispertype/models/`.

## Architecture

WhisperType is a Tauri 2 desktop app with a Rust backend and React + TypeScript frontend. Audio capture runs through PipeWire via the PulseAudio Simple API (`pipewire-pulse`), feeding samples into a lock-free ring buffer. A dedicated DSP thread drains the buffer, converts to mono 16kHz, runs energy-based voice activity detection, and dispatches speech chunks over an MPSC channel to the transcription thread. The transcription thread runs whisper.cpp (via `whisper-rs`) with CUDA acceleration and flash attention, reusing a single inference state per session to avoid repeated GPU initialization.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the detailed threading model and data flow.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, build commands, and PR guidelines.

## License

[MIT](LICENSE)
