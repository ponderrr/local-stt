# Fix History

## Session 2026-02-08

### Fix 1: Invalid `--project` flag in npm script
- **File**: `/home/frosty/local-stt/package.json`
- **Error**: `error: unexpected argument '--project' found`
- **Root cause**: npm `tauri` script used `tauri --project backend` but Tauri v2 CLI has no `--project` flag
- **Fix**: Changed to `"tauri": "tauri"` -- Tauri auto-detects `backend/` directory

### Fix 2: Wayland protocol error crash
- **File**: `/home/frosty/local-stt/backend/src/main.rs`
- **Error**: `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display.`
- **Root cause**: WebKitGTK incompatibility with KDE Plasma Wayland compositor; env pre-sets `GDK_BACKEND=wayland`
- **Fix**: Unconditionally set `GDK_BACKEND=x11` in `main()` before `run()` using `unsafe { std::env::set_var(...) }`
- **Note**: Initially tried conditional `is_err()` check, but env already had `GDK_BACKEND=wayland` set by desktop environment

### Fix 3: Model download fails with "No such file or directory" on rename
- **File**: `/home/frosty/local-stt/backend/src/model_manager/download.rs`
- **Error**: `Failed to rename temp file: No such file or directory (os error 2)`
- **Root cause**: tokio::fs::File not closed (dropped) before tokio::fs::rename; also used sync fs::create_dir_all with exists() guard instead of async create_dir_all
- **Fix**: (1) Replaced sync `Config::ensure_dirs()` with `tokio::fs::create_dir_all(&models_dir).await` (2) Added `drop(file)` after flush() and before rename()

### Fix 4: Settings panel dropdowns have unreadable text
- **File**: `/home/frosty/local-stt/frontend/src/components/settings-panel.tsx`
- **Error**: Native `<option>` elements showed light text on light background in WebKitGTK
- **Root cause**: WebKitGTK ignores CSS classes on native `<option>` elements, defaults to OS/light theme colors
- **Fix**: Added inline `style={{ backgroundColor: "#18181b", color: "#fafafa" }}` to all `<option>` elements

### Fix 5: Recording activates but no transcription appears (VAD threshold too high)
- **File**: `/home/frosty/local-stt/backend/src/config/settings.rs`
- **Error**: Audio pipeline runs, status shows "listening", but no text output
- **Root cause**: Default `vad_threshold` was 0.3 (RMS energy), far too high for normal speech (typical RMS ~0.02-0.15)
- **Fix**: Changed default `vad_threshold` from 0.3 to 0.01

### Total errors fixed: 5
