# 🎙️ WhisperType (local-stt) — Architectural Proposal

> **Status:** APPROVED
> **Repo:** https://github.com/ponderrr/local-stt.git
> **Methodology:** Andrew Ponder Build System

---

## Discovery Summary

| Decision | Answer |
|----------|--------|
| **Primary Use** | Real-time dictation (talk → text appears live) |
| **Output Targets** | Direct into active text field + clipboard |
| **Interface** | Desktop GUI with controls |
| **Framework** | Tauri v2 (Rust backend, React frontend) |
| **Hotkey Behavior** | Toggle on/off with global hotkey (Ctrl+Shift+Space) |
| **Models Available** | All Whisper sizes via dropdown (tiny → large-v3) |
| **Download Strategy** | First-run setup wizard, fetch on demand |
| **OS** | CachyOS Linux (Arch-based) |
| **GPU** | RTX 5060 Ti (16GB VRAM), NVIDIA driver 590.48 |
| **CPU** | Ryzen 7 9700X (8-core/16-thread) |
| **RAM** | 32GB |
| **Storage** | 1TB NVMe Kingston |
| **Design System** | shadcn/ui dark, rich charcoal (#0f0f11), NOT pure black |

---

## Tech Stack

### Backend (Rust / Tauri)
| Crate | Purpose |
|-------|---------|
| `tauri` v2 | App framework, window management, IPC |
| `tauri-plugin-global-shortcut` | System-wide hotkey registration |
| `whisper-rs` (CUDA) | Rust bindings to whisper.cpp — GPU transcription |
| `cpal` | Cross-platform audio capture (mic input) |
| `enigo` (wayland+x11) | Simulate keyboard input → type into active field |
| `arboard` | Clipboard read/write |
| `reqwest` (stream) | Model downloads from HuggingFace |
| `futures-util` | Async stream processing |
| `serde` / `serde_json` | Config serialization |
| `tokio` (full) | Async runtime |
| `hound` | WAV audio buffer handling |
| `dirs` | XDG-compliant home directory resolution |

### Frontend (Tauri WebView)
| Tech | Purpose |
|------|---------|
| React 18 | UI framework |
| TypeScript | Type safety |
| Tailwind CSS | Utility-first styling |
| shadcn/ui | Component library (dark theme) |
| @tauri-apps/api | Rust ↔ Frontend IPC |

---

## Design System

> "Darkness is the design, color is the exception."

| Token | Value | Usage |
|-------|-------|-------|
| Background | `#0f0f11` | Page background (rich charcoal, NOT pure black) |
| Card | `#131316` | Widget/card surfaces (slightly elevated) |
| Popover | `#18181b` | Dropdowns, modals |
| Border | `rgba(255,255,255,0.08)` | Near-invisible borders |
| Text Primary | `#fafafa` | Main text |
| Text Muted | `#8b8b94` | Secondary text |
| Accent | ~5% of UI surface | Primary color (project-specific) |

**Rules:**
- ✅ Rich charcoal backgrounds with subtle depth layers
- ✅ Near-invisible borders (white at 8-10% opacity)
- ✅ Center all content — titles, values, labels
- ✅ Uppercase, small, muted section/widget titles
- ❌ NO pure black (#000000)
- ❌ NO glassmorphism / backdrop-blur
- ❌ NO glow effects / colored shadows

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    TAURI WINDOW                          │
│  ┌───────────────────────────────────────────────────┐  │
│  │              React Frontend (WebView)              │  │
│  │  Model Dropdown • Transcript • Status • Settings   │  │
│  └──────────────────────┬────────────────────────────┘  │
│                         │ Tauri IPC                      │
│  ┌──────────────────────┴────────────────────────────┐  │
│  │              Rust Backend (Tauri Core)              │  │
│  │                                                    │  │
│  │  Audio Engine ──► Transcription Engine              │  │
│  │  (cpal+VAD)       (whisper-rs + CUDA)              │  │
│  │                                                    │  │
│  │  Hotkey Manager    Output Manager                   │  │
│  │  (global-shortcut) (enigo + arboard)               │  │
│  │                                                    │  │
│  │  Model Manager     Config Manager                   │  │
│  │  (reqwest)         (serde + JSON)                  │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘

         ~/.whispertype/
         ├── config.json
         ├── models/
         │   ├── ggml-tiny.bin
         │   ├── ggml-base.bin
         │   ├── ggml-small.bin
         │   ├── ggml-medium.bin
         │   └── ggml-large-v3.bin
         └── logs/
```

---

## Audio Pipeline

```
Microphone → cpal (16kHz mono) → Ring Buffer → VAD Gate → whisper-rs (CUDA) → Output
```

- 3-second chunks with 0.5s overlap
- Energy-based VAD filters silence (saves GPU)
- Async: capture thread → processing thread → output thread

---

## Build Decomposition

| Section | Phases | Description |
|---------|--------|-------------|
| A | A.1 – A.6 | Project scaffolding (Tauri + React + deps) |
| B | B.1 – B.5 | Audio engine (capture, buffer, VAD) |
| C | C.1 – C.4 | Transcription + all Tauri commands |
| D | D.1 – D.2 | Global hotkey system |
| E | E.1 – E.8 | Frontend UI (main window, components, hooks) |
| F | F.1 – F.4 | First-run setup wizard |
| G | G.1 – G.5 | Integration, polish, README |

**Total: ~32 micro-phases**

---

## Cursor Rules Files

| File | Purpose |
|------|---------|
| `.cursorrules` | Project context, file structure, sacred laws |
| `.cursorrules-rust-tauri` | Rust/Tauri patterns, whisper-rs, cpal, async |
| `.cursorrules-react-dark` | React/shadcn dark design system tokens |
