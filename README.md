# WhisperType

Local AI-powered speech-to-text dictation for Linux. Everything runs on your machine — no cloud, no API keys, no data leaving your computer.

Built with **Tauri** (Rust) + **React** (TypeScript) + **whisper.cpp** (CUDA-accelerated).

## Features

- **Real-time dictation** — Speak and text appears in your active application
- **GPU-accelerated** — Uses NVIDIA CUDA for fast transcription
- **Multiple models** — Tiny to Large-v3, switch via dropdown
- **Global hotkey** — Ctrl+Shift+Space toggles from anywhere
- **Privacy-first** — 100% local, zero network after model download
- **Dual output** — Type into active field + copy to clipboard

## Prerequisites

- **OS:** Linux (developed on CachyOS/Arch). Note: The audio capture pipeline is optimized for PipeWire on Linux.
- **GPU:** NVIDIA GPU with CUDA support (recommended)
- **CUDA Toolkit:** Required for GPU acceleration
- **Node.js:** 18+
- **Rust Toolchain:** Latest stable

## Installation

### System Dependencies (Arch/CachyOS)

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget openssl \
  alsa-lib alsa-utils cuda cudnn libappindicator-gtk3 librsvg
```

### Build & Run

```bash
git clone https://github.com/ponderrr/local-stt.git
cd local-stt
npm install
npm run dev

# In a separate terminal:
cd backend
cargo run
```

### Production Build

```bash
cargo tauri build
```

## Usage

1. Launch WhisperType
2. Complete first-run setup (GPU detection + model download)
3. Select your preferred model from the dropdown
4. Press **Ctrl+Shift+Space** to start dictating
5. Speak naturally — text appears in real-time
6. Press **Ctrl+Shift+Space** again to stop

## Architecture Overview

- **Backend:** Rust (Tauri)
- **Frontend:** React + TypeScript + Tailwind + shadcn/ui
- **Transcription:** whisper.cpp via whisper-rs with CUDA acceleration
- **Audio Pipeline:**
  Microphone input is securely handled by an isolated, non-blocking **Audio Actor Thread** using `cpal`. Samples are pushed directly into a lock-free `ringbuf` heap. A dedicated **DSP Thread** pulls samples out, computes energy-based Voice Activity Detection (VAD), downsamples to mono 16kHz, and dispatches framed clean speech chunks over an asynchronous channel to the STT transcription engine. This prevents OS-level audio buffer underruns completely.

## License

MIT
