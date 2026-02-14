# WhisperType Developer Guide

This guide covers everything a new contributor needs to set up the development
environment, run the application, execute tests, and extend the codebase.

See also: [ARCHITECTURE.md](./ARCHITECTURE.md) for system design,
[ALGORITHMS.md](./ALGORITHMS.md) for algorithm details,
[API-REFERENCE.md](./API-REFERENCE.md) for the complete typed interface.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Development Setup](#development-setup)
- [Running in Dev Mode](#running-in-dev-mode)
- [Running Tests](#running-tests)
- [Adding a New Tauri Command](#adding-a-new-tauri-command)
- [Adding a New Settings Option](#adding-a-new-settings-option)
- [Debugging Guide](#debugging-guide)
- [Code Conventions](#code-conventions)

---

## Prerequisites

### System Requirements

| Requirement          | Version / Notes                                      |
|----------------------|------------------------------------------------------|
| Rust                 | Edition 2021 (stable toolchain)                      |
| Node.js              | v18+ (for npm, Vite, TypeScript)                     |
| npm                  | v9+                                                  |
| CUDA Toolkit         | 11.x or 12.x (for GPU-accelerated Whisper)           |
| NVIDIA Driver        | Compatible with your CUDA version                    |

### Linux-Specific Dependencies

Tauri v2 on Linux requires WebKitGTK and several system libraries:

```
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget \
    libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \
    libasound2-dev libxdo-dev

# Arch Linux
sudo pacman -S webkit2gtk-4.1 base-devel openssl gtk3 \
    libayatana-appindicator librsvg alsa-lib xdotool
```

For audio capture (`cpal`), you need ALSA development headers (`libasound2-dev` on
Debian or `alsa-lib` on Arch).

For keyboard simulation (`enigo`), you need `libxdo-dev` on Debian or `xdotool` on
Arch (for X11 support).

---

## Development Setup

### 1. Clone the Repository

```bash
git clone <repository-url>
cd local-stt
```

### 2. Install Node.js Dependencies

```bash
npm install
```

This installs React, Tauri CLI, Vite, Tailwind CSS, Vitest, and all other frontend
dependencies listed in `package.json`.

### 3. Verify Rust Toolchain

```bash
rustc --version
cargo --version
```

No additional Rust setup is needed -- the `backend/Cargo.toml` defines all
dependencies and they will be downloaded on first build.

### 4. Verify CUDA (Optional)

GPU acceleration is enabled by default via the `cuda` feature in `Cargo.toml`:

```toml
[features]
default = ["cuda"]
cuda = ["whisper-rs/cuda"]
```

To verify CUDA is available:

```bash
nvidia-smi
```

To build without CUDA (CPU-only mode):

```bash
cd backend && cargo build --no-default-features
```

### 5. Create the Application Directory

On first run, the app creates `~/.whispertype/` automatically. If you want to
pre-create it:

```bash
mkdir -p ~/.whispertype/models
```

---

## Running in Dev Mode

### Full Tauri Dev Server

From the project root:

```bash
npx tauri dev
```

This starts:
1. Vite dev server on `http://localhost:1420` (with HMR)
2. Rust backend compilation and launch
3. The Tauri window pointing at the Vite dev URL

The first build will take several minutes as Rust dependencies and `whisper.cpp`
compile from source.

### Frontend Only (No Backend)

To iterate on UI without the Rust backend:

```bash
npm run dev
```

This starts only the Vite dev server. Tauri IPC calls will fail, but you can
develop UI components and styles.

### Backend Only (Cargo Check)

To check the Rust code for compilation errors without building:

```bash
cd backend && cargo check
```

Note: `cargo check` must be run from the `backend/` directory, not the project root.

---

## Running Tests

### Backend Tests (Rust)

```bash
cd backend && cargo test
```

This runs all unit tests across the backend modules. Tests are defined alongside
the source code in `#[cfg(test)]` blocks.

The backend has comprehensive unit tests for:
- Audio format conversion (`to_mono`, `resample`)
- Ring buffer behavior (write, chunk extraction, overlap)
- Voice activity detection (RMS energy, state transitions, thresholds)
- Configuration serialization/deserialization
- Model registry integrity
- Model file management (`delete_model`, `is_model_downloaded`)
- Output mode routing

Some tests (output, clipboard, keyboard) are environment-dependent and may produce
`Err` results in headless/CI environments. This is expected -- the tests verify
that the code does not panic rather than asserting success.

### Frontend Tests (TypeScript/Vitest)

```bash
npm test
```

Or to run in watch mode:

```bash
npm run test:watch
```

Frontend tests use Vitest with jsdom and `@testing-library/react`. They mock the
Tauri `invoke` and `listen` APIs to test hooks and IPC wrappers in isolation.

Test files:
- `frontend/src/lib/tauri.test.ts` -- tests all command wrappers and event listeners
- `frontend/src/hooks/use-dictation.test.ts` -- tests dictation status, error
  handling, auto-clear timeout
- `frontend/src/hooks/use-models.test.ts` -- tests model loading, downloading,
  deletion, error recovery

The test setup file (`frontend/src/test-setup.ts`) imports
`@testing-library/jest-dom/vitest` for DOM matchers.

---

## Adding a New Tauri Command

Follow these steps to add a new IPC command that the frontend can invoke.

### Step 1: Define the Rust Command

Create or extend a file in `backend/src/commands/`. For example, to add a
`get_version` command:

```rust
// backend/src/commands/system.rs

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

If the command needs access to application state, add a `State` parameter:

```rust
#[tauri::command]
pub fn my_command(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.language.clone())
}
```

For async commands (e.g., network calls), use `async`:

```rust
#[tauri::command]
pub async fn my_async_command(app: AppHandle) -> Result<(), String> {
    // async work here
    Ok(())
}
```

### Step 2: Register the Command

Add it to the `invoke_handler` in `backend/src/lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::system::get_version,  // <-- add here
])
```

### Step 3: Add the Frontend Wrapper

Add a typed wrapper in `frontend/src/lib/tauri.ts`:

```typescript
export const commands = {
  // ... existing commands ...
  getVersion: () => invoke<string>("get_version"),
};
```

Note: Tauri v2 auto-converts snake_case Rust parameters to camelCase for JavaScript.
So a Rust parameter `model_id: String` becomes `modelId` in the `invoke` call.

### Step 4: Use in a Component or Hook

```typescript
import { commands } from "@/lib/tauri";

// In a component or hook:
const version = await commands.getVersion();
```

### Step 5: Add Tests

**Backend**: add a `#[test]` function in the relevant module's `#[cfg(test)]` block.

**Frontend**: add test cases in a `.test.ts` file:

```typescript
it("getVersion calls invoke with correct command", async () => {
  mockedInvoke.mockResolvedValue("0.1.0");
  const result = await commands.getVersion();
  expect(mockedInvoke).toHaveBeenCalledWith("get_version");
  expect(result).toBe("0.1.0");
});
```

---

## Adding a New Settings Option

### Step 1: Add the Field to Config

In `backend/src/config/settings.rs`, add the new field to the `Config` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... existing fields ...
    pub my_new_option: bool,
}
```

Update the `Default` implementation:

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            my_new_option: false,
        }
    }
}
```

### Step 2: Update the TypeScript Type

In `frontend/src/lib/tauri.ts`, add the field to the `Config` interface:

```typescript
export interface Config {
  // ... existing fields ...
  my_new_option: boolean;
}
```

### Step 3: Add the UI Control

In `frontend/src/components/settings-panel.tsx`, add a control inside the settings
form. For a boolean toggle:

```tsx
<div>
  <label className="text-xs font-medium uppercase tracking-wider text-muted-foreground block mb-2">
    My New Option
  </label>
  <input
    type="checkbox"
    checked={localConfig.my_new_option}
    onChange={(e) =>
      setLocalConfig({ ...localConfig, my_new_option: e.target.checked })
    }
  />
</div>
```

The existing save/cancel flow in `SettingsPanel` will handle persisting the change
through `updateConfig()`.

### Step 4: Use the Setting

On the backend, access the setting from `AppState`:

```rust
let config = state.config.lock().map_err(|e| e.to_string())?;
if config.my_new_option {
    // ...
}
```

On the frontend, access it via the `useConfig` hook:

```typescript
const { config } = useConfig();
if (config?.my_new_option) {
  // ...
}
```

### Step 5: Handle Migration

If existing users have a saved `config.json` without the new field, deserialization
will fail. Consider using `#[serde(default)]` on the new field:

```rust
pub struct Config {
    // ... existing fields ...
    #[serde(default)]
    pub my_new_option: bool,
}
```

This makes the field optional during deserialization, defaulting to `false` (or
whatever `Default` produces for the type).

---

## Debugging Guide

### Backend Debugging

**Logging**: The backend uses `eprintln!()` for diagnostic output. When running
`npx tauri dev`, Rust stderr is printed to the terminal.

Look for these log prefixes:
- `[audio]`: audio device configuration and stream errors
- `Transcription error:`: Whisper inference failures
- `Output error:`: keyboard or clipboard failures

**Checking audio device**: To verify audio capture is working:

```bash
# List available audio devices from the app
# (invoke list_audio_devices from the frontend, or check stderr for [audio] logs)
```

The audio capture module logs device configuration on start:
```
[audio] Device config: rate=48000Hz, channels=2, format=F32
```

**Model issues**: If model loading fails, check:
1. The model file exists in `~/.whispertype/models/`
2. The file is not empty (incomplete download)
3. CUDA drivers are loaded (for GPU models)
4. Sufficient VRAM is available

**Common errors**:
- "No model loaded": ensure a model is downloaded and loaded before starting dictation
- "Failed to init keyboard simulator": enigo cannot connect to X11/Wayland display
  server
- "Failed to access clipboard": arboard cannot access system clipboard (may happen
  in Wayland without proper permissions)

### Frontend Debugging

**Browser DevTools**: Right-click in the Tauri window and select "Inspect" to open
WebKitGTK DevTools. This gives you the console, network tab, and DOM inspector.

**Event debugging**: Add temporary event listeners in the browser console:

```javascript
// Listen to all Tauri events
window.__TAURI_INTERNALS__.invoke("plugin:event|listen", {
  event: "dictation-status",
  handler: (e) => console.log("Status:", e.payload)
});
```

**React DevTools**: Install the React DevTools browser extension. Since Tauri uses
WebKitGTK, you may need the standalone React DevTools connected via the debug port.

### Linux-Specific Issues

**Black screen on NVIDIA**: The `WEBKIT_DISABLE_DMABUF_RENDERER=1` environment
variable in `main.rs` should fix this. If the window is still black, try setting
`GDK_BACKEND=x11` manually in your shell.

**Wayland keyboard simulation**: If `enigo` fails on Wayland, try running the app
under XWayland. The `GDK_BACKEND=x11` setting in `main.rs` already forces the
WebKitGTK renderer to use X11, but keyboard simulation may still need an X11
session for the target application.

---

## Code Conventions

### Rust (Backend)

- **Crate name**: `whispertype` (in Cargo.toml), library name `tauri_app_lib`
- **Entry point**: `lib.rs` contains the `run()` function; `main.rs` is a thin
  wrapper that sets environment variables and calls `run()`
- **Error handling**: Commands return `Result<T, String>` -- Tauri serializes the
  `Err` variant as a string error for the frontend
- **Thread model**: Audio processing uses `std::thread`, not `tokio::spawn`. The
  Tokio runtime is used only for `async` Tauri commands (model download, model load)
- **State access**: `AppState` is defined in `commands/dictation.rs`. Other command
  modules reference it via `use crate::commands::dictation::AppState`
- **Mutex discipline**: Lock mutexes briefly. Drop guards (or the `Mutex` lock)
  before spawning threads or performing blocking operations
- **Tests**: placed in `#[cfg(test)] mod tests` blocks at the bottom of each file

### TypeScript (Frontend)

- **Path aliases**: `@/` maps to `frontend/src/` (configured in `tsconfig.json`
  and `vite.config.ts`)
- **Component files**: PascalCase export names, kebab-case filenames
  (e.g., `ModelSelector` in `model-selector.tsx`)
- **Hook files**: `use-<name>.ts` naming convention, one hook per file
- **IPC centralization**: all Tauri `invoke` and `listen` calls go through
  `lib/tauri.ts` -- components never call `invoke()` directly
- **Styling**: Tailwind CSS utility classes, dark theme only. The color scheme is
  defined in `frontend/src/index.css` using CSS custom properties
- **State management**: React hooks + Tauri events, no external state library
  (no Redux, Zustand, etc.)
- **Test framework**: Vitest with `@testing-library/react` for hook testing,
  jsdom environment

### Project Structure

```
local-stt/
  package.json           -- Root npm config (scripts, frontend deps)
  vite.config.ts         -- Vite config (React, Tailwind, path aliases)
  tsconfig.json          -- TypeScript config (strict mode, path aliases)
  frontend/
    src/                 -- All frontend source code
  backend/
    Cargo.toml           -- Rust dependencies and features
    tauri.conf.json      -- Tauri v2 configuration (window, security, build)
    src/                 -- All backend source code
  docs/                  -- Documentation (this directory)
```

The root `package.json` handles all frontend tooling. The Tauri CLI (`npx tauri`)
knows to look in `backend/` for the Rust crate via the Tauri configuration.

### Git Conventions

- Commit messages follow conventional commits: `feat:`, `fix:`, `docs:`, `test:`,
  `refactor:`, etc.
- Branch naming: `feat/<description>`, `fix/<description>`
- All changes should include tests where applicable
