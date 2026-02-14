# Debugger Agent Memory

## Environment
- Arch Linux (CachyOS) with KDE Plasma on Wayland
- Rust 1.93.0, Node 25.4.0, npm 11.8.0
- Tauri CLI 2.10.0, Tauri 2.10.2
- rustup NOT installed (system rust package)

## Fixed Issues Log
See [fixes.md](fixes.md) for detailed issue history.

## Key Learnings

### Tauri CLI
- `--project` flag does NOT exist in Tauri v2 CLI
- Tauri auto-detects `backend/` directory containing `tauri.conf.json`
- npm script should be just `"tauri": "tauri"`, not `"tauri": "tauri --project backend"`

### Wayland/GDK
- KDE Plasma Wayland + WebKitGTK causes `Error 71 (Protocol error)` crash
- Fix: Force `GDK_BACKEND=x11` in `main.rs` before `run()`
- Environment pre-sets `GDK_BACKEND=wayland`, so must unconditionally override (not check `is_err()`)
- `std::env::set_var` requires `unsafe` block in Rust 1.83+ (edition 2021 compiles but warns)
- `Failed to create GBM buffer` warning under X11 is harmless

### Build Pipeline
- `cargo check` from `backend/` dir: checks Rust
- `npx tsc --noEmit` from root: checks TypeScript
- Both pass cleanly as of last session

### Download/File Operations
- tokio::fs::File must be dropped (closed) before tokio::fs::rename to avoid ENOENT
- Use tokio::fs::create_dir_all instead of sync fs with exists() checks for async contexts
- PathBuf::with_extension("bin.tmp") replaces extension correctly: `foo.bin` -> `foo.bin.tmp`

### VAD (Voice Activity Detection)
- RMS energy threshold of 0.3 is FAR too high for normal speech detection
- Normal speech RMS is ~0.02-0.15 on normalized [-1,1] audio
- Default threshold set to 0.01 to ensure speech passes through
- VAD also requires min_speech_frames (3) consecutive frames above threshold

### WebKit/Select Styling
- Native <option> elements in WebKitGTK ignore CSS classes for background/text
- Must use inline `style` attributes with explicit backgroundColor and color
- This is a WebKitGTK-specific issue; Chromium handles CSS classes on options
