# Contributing to WhisperType

Thanks for your interest in contributing to WhisperType! This guide will help you get set up and submit your first PR.

## Development Environment

### Prerequisites

- **OS:** Linux (X11 or Wayland)
- **Rust:** 1.77+ (`rustup` recommended)
- **Node.js:** 20+
- **NVIDIA GPU** with CUDA 12+ and cuDNN installed
- **System packages:**

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev \
  patchelf libpulse-dev pkg-config cmake build-essential

# Arch Linux
sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3 librsvg patchelf \
  libpulse pkg-config cmake base-devel
```

### Setup

```bash
git clone https://github.com/ponderrr/local-stt.git
cd local-stt
npm install
```

### Running in Development

```bash
npx tauri dev
```

This starts both the Vite frontend dev server and the Rust backend with hot-reload.

## Build Commands

### With CUDA (default)

```bash
npx tauri build
```

For NVIDIA Blackwell GPUs (RTX 50-series), set the compute capability:

```bash
CMAKE_CUDA_ARCHITECTURES=120 npx tauri build
```

### Without CUDA (CPU-only)

```bash
cd backend
cargo build --no-default-features
```

## Testing

All tests must pass before submitting a PR.

```bash
# Rust tests
cd backend
cargo test

# Frontend tests
npm test

# Clippy (must be clean — zero warnings)
cd backend
cargo clippy -- -D warnings
```

## Submitting a Pull Request

1. Fork the repository
2. Create a feature branch from `main`: `git checkout -b my-feature`
3. Make your changes
4. Ensure `cargo test`, `npm test`, and `cargo clippy -- -D warnings` all pass
5. Commit with a clear message describing what and why
6. Push your branch and open a PR against `main`

## Code Style

- **Rust:** Edition 2021, formatted with `rustfmt` defaults
- **TypeScript:** Formatted with project Prettier/ESLint config
- Keep changes focused — one concern per PR
- Add tests for new functionality where practical
