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

## Requirements

- **OS:** Linux (developed on CachyOS/Arch)
- **GPU:** NVIDIA GPU with CUDA support (recommended)
- **CUDA Toolkit:** Required for GPU acceleration
- **Node.js:** 18+
- **Rust:** Latest stable

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
cargo tauri dev
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

## Architecture

- **Backend:** Rust (Tauri) — audio capture, whisper-rs transcription, keyboard simulation
- **Frontend:** React + TypeScript + Tailwind + shadcn/ui
- **Transcription:** whisper.cpp via whisper-rs with CUDA acceleration
- **Audio:** cpal for microphone capture, energy-based VAD

## License

MIT
