# WhisperType — Audit Fix Plan
## Generated from comprehensive codebase audit

---

## Summary Table

| Phase | Title | Priority | Category | Files |
|-------|-------|----------|----------|-------|
| 1 | Fix audio capture to use device-native format with resampling | CRITICAL | Bug Fix | `backend/src/audio/capture.rs` |
| 2 | Surface output_text errors to frontend | HIGH | Bug Fix | `backend/src/commands/dictation.rs`, `frontend/src/hooks/use-dictation.ts`, `frontend/src/pages/main-window.tsx` |
| 3 | Surface transcription errors to frontend | HIGH | Bug Fix | `backend/src/commands/dictation.rs`, `frontend/src/hooks/use-dictation.ts` |
| 4 | Track and join transcription thread on toggle | HIGH | Bug Fix | `backend/src/commands/dictation.rs` |
| 5 | Fix model auto-load race condition after setup | MEDIUM | Bug Fix | `frontend/src/pages/main-window.tsx`, `frontend/src/hooks/use-models.ts` |
| 6 | Set Content Security Policy | MEDIUM | Bug Fix | `backend/tauri.conf.json` |
| 7 | Fix index.html title | LOW | Code Smell | `index.html` |
| 8 | Fix StepComplete placeholder text | LOW | Code Smell | `frontend/src/components/setup-wizard/step-complete.tsx` |
| 9 | Remove unnecessary unsafe block in main.rs | LOW | Code Smell | `backend/src/main.rs` |
| 10 | Fix unnecessary type cast in StepGpu | LOW | Code Smell | `frontend/src/components/setup-wizard/step-gpu.tsx` |
| 11 | Normalize Config import paths | LOW | Code Smell | `backend/src/commands/dictation.rs` |
| 12 | Remove unused `_toggle` and `isListening` | LOW | Code Smell | `frontend/src/pages/main-window.tsx`, `frontend/src/hooks/use-dictation.ts` |
| 13 | Remove dead backend stubs: storage.rs, hotkey/manager.rs | LOW | Dead Code | `backend/src/model_manager/storage.rs`, `backend/src/model_manager/mod.rs`, `backend/src/hotkey/manager.rs`, `backend/src/hotkey/mod.rs` |
| 14 | Remove unused backend methods and dependency | LOW | Dead Code | `backend/src/audio/capture.rs`, `backend/src/audio/vad.rs`, `backend/src/audio/buffer.rs`, `backend/Cargo.toml` |
| 15 | Remove dead frontend page and unused state | LOW | Dead Code | `frontend/src/pages/setup.tsx`, `frontend/src/hooks/use-models.ts` |
| 16 | Remove unused shadcn UI components | LOW | Dead Code | `frontend/src/components/ui/button.tsx`, `card.tsx`, `select.tsx`, `dropdown-menu.tsx`, `dialog.tsx`, `scroll-area.tsx`, `badge.tsx`, `separator.tsx`, `tooltip.tsx`, `progress.tsx` |
| 17 | Eliminate heap allocation in audio callback | MEDIUM | Optimization | `backend/src/audio/capture.rs`, `backend/Cargo.toml` |
| 18 | Make model registry static | LOW | Optimization | `backend/src/transcription/models.rs` |

---

## Section A: Critical Pipeline Fixes (Phases 1-4)

---

### Phase 1: Fix audio capture to use device-native format with resampling
**Priority:** CRITICAL
**Category:** Bug Fix
**Files:** `backend/src/audio/capture.rs`
**Estimated Complexity:** Complex

**Context:**
In `backend/src/audio/capture.rs`, the `start()` method (line 29) hardcodes a `StreamConfig` at line 47-51 with `sample_rate: SampleRate(16000)` and `channels: 1`. Most microphones (USB, built-in, Bluetooth) only support 44100Hz or 48000Hz stereo. When cpal calls `build_input_stream` with this unsupported config, it returns an error and dictation cannot start at all. The app is non-functional on most real hardware because of this.

**Instructions:**
1. In `backend/src/audio/capture.rs`, in the `start()` method at line ~38, replace the hardcoded `StreamConfig` block (lines 47-51) with a query to the device's default supported config using `device.default_input_config()`.
2. Store the device's native sample rate and channel count from the returned `SupportedStreamConfig`.
3. In the `build_input_stream` callback (line 53-59), keep capturing audio in the device's native format.
4. After `sender.send(data.to_vec())` at line 57, add a resampling step in the transcription thread (in `backend/src/commands/dictation.rs`, inside the `while let Ok(chunk) = receiver.recv()` loop at line 46). Before passing audio to `engine.transcribe()`, resample from the device's native rate to 16000Hz and convert stereo to mono if needed. A simple linear interpolation resampler is sufficient for speech.
5. Alternatively, implement the resampling inside `AudioCapture::start()` before sending over the channel — resample in-place so the receiver always gets 16kHz mono f32 samples. This keeps the resampling logic contained in the audio module.
6. Update the `AudioCapture::new()` constructor (line 11-16) to no longer require a `sample_rate` parameter, since the rate will be determined by the device. Or keep it as the *target* sample rate and document it as such.

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors
- [ ] Run `cargo run` or `npm run tauri dev`, click the dictation toggle (or press Ctrl+Shift+Space), and confirm in the terminal that audio capture starts without a "stream configuration not supported" error
- [ ] Speak for 5 seconds and confirm the terminal shows transcription output (even if garbled — the point is that audio reaches whisper-rs)
- [ ] Test with at least one real microphone (not just default device)

**Approval Gate:** Do not proceed to Phase 2 until audio capture successfully starts and audio data reaches the transcription engine on real hardware.

---

### Phase 2: Surface output_text errors to frontend
**Priority:** HIGH
**Category:** Bug Fix
**Files:** `backend/src/commands/dictation.rs`, `frontend/src/hooks/use-dictation.ts`, `frontend/src/pages/main-window.tsx`
**Estimated Complexity:** Moderate

**Context:**
In `backend/src/commands/dictation.rs` at line 49, `output::output_text(&segment.text, &output_mode).ok()` silently discards the Result. If `enigo::text()` or `arboard::set_text()` fails (common on Wayland, or when no X11 display is set), the user sees transcription text appear in the UI but nothing is typed or copied into their target application, with no error indication whatsoever.

**Instructions:**
1. In `backend/src/commands/dictation.rs`, at line 49, replace `.ok()` with proper error handling. When `output_text` returns `Err(e)`, emit an error event to the frontend:
   ```rust
   if let Err(e) = output::output_text(&segment.text, &output_mode) {
       app_clone.emit("output-error", format!("Failed to output text: {}", e)).ok();
   }
   ```
2. In `frontend/src/lib/tauri.ts`, add a new event listener in the `events` object for `"output-error"` that takes a `string` payload.
3. In `frontend/src/hooks/use-dictation.ts`, add a state variable `outputError: string | null` and subscribe to the `"output-error"` event. Clear the error after a timeout (e.g., 5 seconds) or on next successful transcription.
4. In `frontend/src/pages/main-window.tsx`, display the `outputError` when non-null — a simple red text banner or toast below the status indicator is sufficient.

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors
- [ ] Run `npm run build` from root — no TypeScript errors
- [ ] To test: temporarily make `output_text` always fail (e.g., return `Err("test error".into())`) and confirm the error appears in the frontend UI
- [ ] Revert the test failure and confirm normal operation still works

**Approval Gate:** Do not proceed to Phase 3 until output errors are visibly surfaced in the frontend.

---

### Phase 3: Surface transcription errors to frontend
**Priority:** HIGH
**Category:** Bug Fix
**Files:** `backend/src/commands/dictation.rs`, `frontend/src/hooks/use-dictation.ts`
**Estimated Complexity:** Simple

**Context:**
In `backend/src/commands/dictation.rs` at line 62, when `engine.transcribe()` fails, the error is only printed with `eprintln!("Transcription error: {}", e)`. The frontend has no way to know transcription failed. This can happen with malformed audio, GPU OOM, or model corruption.

**Instructions:**
1. In `backend/src/commands/dictation.rs`, at line 62 inside the `Err(e)` branch, add an event emission:
   ```rust
   Err(e) => {
       eprintln!("Transcription error: {}", e);
       app_clone.emit("transcription-error", format!("Transcription failed: {}", e)).ok();
   }
   ```
2. In `frontend/src/lib/tauri.ts`, add a new event listener for `"transcription-error"` with a `string` payload.
3. In `frontend/src/hooks/use-dictation.ts`, subscribe to the `"transcription-error"` event and expose a `transcriptionError` state variable. Clear after 5 seconds or on next successful transcription.
4. Display this error in the UI (can reuse the same error display mechanism from Phase 2).

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors
- [ ] Run `npm run build` from root — no TypeScript errors
- [ ] To test: temporarily make `engine.transcribe()` return an error, confirm the error message appears in the frontend UI
- [ ] Revert and confirm normal transcription still works

**Approval Gate:** Do not proceed to Phase 4 until transcription errors are visibly surfaced in the frontend.

---

### Phase 4: Track and join transcription thread on toggle
**Priority:** HIGH
**Category:** Bug Fix
**Files:** `backend/src/commands/dictation.rs`
**Estimated Complexity:** Moderate

**Context:**
In `backend/src/commands/dictation.rs`, the `std::thread::spawn` at line 44 creates a transcription thread but never stores the `JoinHandle`. If a user rapidly toggles dictation on/off/on/off, multiple transcription threads can run simultaneously, all fighting over the `ctx` Mutex in `TranscriptionEngine`. This causes thread leaks and Mutex contention.

**Instructions:**
1. In `backend/src/commands/dictation.rs`, add a field to the `AppState` struct (line 9-13):
   ```rust
   pub transcription_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
   ```
2. Initialize it as `Mutex::new(None)` wherever `AppState` is constructed (in `backend/src/lib.rs`).
3. In `toggle_dictation_inner()`, before spawning a new transcription thread, check if there's an existing JoinHandle. If so, join it (it should already be finishing since `pipeline.stop()` was called, which drops the sender and causes `receiver.recv()` to return `Err`):
   ```rust
   // Join previous transcription thread if any
   if let Some(handle) = state.transcription_thread.lock().unwrap().take() {
       handle.join().ok();
   }
   ```
4. After `std::thread::spawn(...)` at line 44, store the returned `JoinHandle` in `state.transcription_thread`.
5. In the stop-dictation branch of `toggle_dictation_inner()`, after calling `pipeline.stop()`, also join the transcription thread to ensure clean shutdown before returning.

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors
- [ ] Run `npm run tauri dev`, rapidly toggle dictation on/off 10 times in quick succession (Ctrl+Shift+Space), confirm no panics or thread-related errors in the terminal
- [ ] Confirm dictation still starts and stops cleanly after rapid toggling
- [ ] Confirm only one transcription thread is active at a time (add a temporary `println!("transcription thread started/ended")` to verify)

**Approval Gate:** Do not proceed to Phase 5 until rapid toggle cycles produce no thread leaks or panics.

---

## Section B: Frontend & Config Fixes (Phases 5-6)

---

### Phase 5: Fix model auto-load race condition after setup
**Priority:** MEDIUM
**Category:** Bug Fix
**Files:** `frontend/src/pages/main-window.tsx`, `frontend/src/hooks/use-models.ts`
**Estimated Complexity:** Simple

**Context:**
In `frontend/src/pages/main-window.tsx` at lines 18-28, the `useEffect` that auto-loads the default model depends on `models` being populated. But `useModels` triggers an async `refresh()` call, and during the first render cycle, `models` may still be empty. The effect might not trigger correctly on the first mount if `models` hasn't loaded yet.

**Instructions:**
1. In `frontend/src/hooks/use-models.ts`, ensure the `refresh()` function is called on mount and that the `models` state is guaranteed to be populated before the auto-load effect runs. Add a `loading` boolean state that starts as `true` and is set to `false` after the first `refresh()` completes.
2. In `frontend/src/pages/main-window.tsx`, in the `useEffect` at line 18, add `loading` as a guard condition: only attempt auto-load when `!loading` (i.e., models have been fetched at least once):
   ```typescript
   useEffect(() => {
     if (loading || !config || activeModel) return;
     const defaultModel = config.default_model;
     const isDownloaded = models.find((m) => m.id === defaultModel && m.downloaded);
     if (isDownloaded) {
       loadModel(defaultModel);
     }
   }, [loading, config, models, activeModel, loadModel]);
   ```
3. Return `loading` from the `useModels` hook so `MainWindow` can consume it.

**Verification:**
- [ ] Run `npm run build` from root — no TypeScript errors
- [ ] Run `npm run tauri dev`, complete the setup wizard, and confirm the default model is auto-loaded after transitioning to MainWindow (status indicator should show "Model loaded" or similar, not "No model loaded")
- [ ] Refresh the app (close and reopen) and confirm the model auto-loads on startup

**Approval Gate:** Do not proceed to Phase 6 until model auto-load works reliably after both first-time setup and subsequent app launches.

---

### Phase 6: Set Content Security Policy
**Priority:** MEDIUM
**Category:** Bug Fix
**Files:** `backend/tauri.conf.json`
**Estimated Complexity:** Trivial

**Context:**
In `backend/tauri.conf.json` at line 27, `"csp": null` explicitly disables the Content Security Policy. This means the webview has unrestricted access to load scripts, styles, and make network requests from any origin. While unlikely to be exploited in a local desktop app, it's a security best practice violation.

**Instructions:**
1. In `backend/tauri.conf.json`, at line 27, replace `"csp": null` with a restrictive CSP:
   ```json
   "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost; connect-src ipc: http://ipc.localhost https://huggingface.co https://*.huggingface.co"
   ```
   Note: `connect-src` includes `huggingface.co` because model downloads fetch from HuggingFace. The `'unsafe-inline'` for styles is needed for Tailwind/CSS-in-JS.

**Verification:**
- [ ] Run `npm run tauri dev` and confirm the app loads without CSP errors in the webview console (open DevTools with F12 or right-click > Inspect)
- [ ] Confirm the setup wizard works (GPU detection, model download with progress events)
- [ ] Confirm no blocked resources in the browser console

**Approval Gate:** Do not proceed to Phase 7 until the app runs cleanly with the new CSP and model downloads still work.

---

## Section C: Code Smell Cleanup (Phases 7-12)

---

### Phase 7: Fix index.html title
**Priority:** LOW
**Category:** Code Smell
**Files:** `index.html`
**Estimated Complexity:** Trivial

**Context:**
In `index.html` at line 7, the page title is still `"Tauri + React + Typescript"` — the default template title. This shows in the window title bar and taskbar.

**Instructions:**
1. In `index.html`, at line 7, change:
   ```html
   <title>Tauri + React + Typescript</title>
   ```
   to:
   ```html
   <title>WhisperType</title>
   ```

**Verification:**
- [ ] Run `npm run tauri dev` and confirm the window title bar shows "WhisperType" (note: Tauri may override this with the `title` field in `tauri.conf.json` — check both places)
- [ ] Confirm no build errors

**Approval Gate:** Do not proceed to Phase 8 until the window title shows "WhisperType".

---

### Phase 8: Fix StepComplete placeholder text
**Priority:** LOW
**Category:** Code Smell
**Files:** `frontend/src/components/setup-wizard/step-complete.tsx`
**Estimated Complexity:** Trivial

**Context:**
In `frontend/src/components/setup-wizard/step-complete.tsx` at line 8, the text `"Microphone"` is rendered in a `<div className="text-4xl">` where an icon or emoji should be. This is a leftover placeholder from development.

**Instructions:**
1. In `frontend/src/components/setup-wizard/step-complete.tsx`, at line 8, replace:
   ```tsx
   <div className="text-4xl">Microphone</div>
   ```
   with an appropriate checkmark or success indicator:
   ```tsx
   <div className="text-6xl">&#10003;</div>
   ```
   Or use a Unicode speech/microphone emoji if preferred: `🎤` or `✅`.

**Verification:**
- [ ] Run `npm run tauri dev`, navigate through the setup wizard to the final step, and confirm an icon/emoji is shown instead of the word "Microphone"
- [ ] Run `npm run build` — no TypeScript errors

**Approval Gate:** Do not proceed to Phase 9 until the StepComplete page shows a proper icon.

---

### Phase 9: Remove unnecessary unsafe block in main.rs
**Priority:** LOW
**Category:** Code Smell
**Files:** `backend/src/main.rs`
**Estimated Complexity:** Trivial

**Context:**
In `backend/src/main.rs` at lines 7-13, `std::env::set_var` is wrapped in an `unsafe` block. In Rust edition 2021 (which this project uses per `backend/Cargo.toml`), `set_var` is a safe function. The `unsafe` block is unnecessary noise.

**Instructions:**
1. In `backend/src/main.rs`, remove the `unsafe { }` wrapper around the `std::env::set_var` calls at lines 8-12. Keep the `#[cfg(target_os = "linux")]` attribute and the `set_var` calls themselves:
   ```rust
   #[cfg(target_os = "linux")]
   {
       std::env::set_var("GDK_BACKEND", "x11");
       std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
   }
   ```

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors or warnings related to unnecessary unsafe
- [ ] Run `npm run tauri dev` — app launches normally on Linux

**Approval Gate:** Do not proceed to Phase 10 until `cargo check` passes cleanly.

---

### Phase 10: Fix unnecessary type cast in StepGpu
**Priority:** LOW
**Category:** Code Smell
**Files:** `frontend/src/components/setup-wizard/step-gpu.tsx`
**Estimated Complexity:** Trivial

**Context:**
In `frontend/src/components/setup-wizard/step-gpu.tsx` at line 15, `info as unknown as GpuInfo` is an unnecessary double-cast. The `commands.getGpuInfo()` call already returns `Promise<GpuInfo>` via `invoke<GpuInfo>()`, so the cast is redundant and hides potential type mismatches.

**Instructions:**
1. In `frontend/src/components/setup-wizard/step-gpu.tsx`, at line 15, replace:
   ```typescript
   .then((info) => setGpu(info as unknown as GpuInfo))
   ```
   with:
   ```typescript
   .then((info) => setGpu(info))
   ```

**Verification:**
- [ ] Run `npm run build` from root — no TypeScript errors
- [ ] Run `npm run tauri dev`, go through setup wizard GPU detection step, confirm GPU info still displays correctly

**Approval Gate:** Do not proceed to Phase 11 until TypeScript builds cleanly and GPU detection still works.

---

### Phase 11: Normalize Config import paths
**Priority:** LOW
**Category:** Code Smell
**Files:** `backend/src/commands/dictation.rs`
**Estimated Complexity:** Trivial

**Context:**
Config is imported inconsistently across command modules. `commands/config.rs` and `commands/models.rs` use `use crate::config::Config;` (via the re-export in `config/mod.rs`), but `commands/dictation.rs` at line 5 uses `use crate::config::settings::Config;` (reaching into the submodule directly). Both work, but the inconsistency is confusing.

**Instructions:**
1. In `backend/src/commands/dictation.rs`, at line 5, change:
   ```rust
   use crate::config::settings::Config;
   ```
   to:
   ```rust
   use crate::config::Config;
   ```
   This matches the import style used in `commands/config.rs` and `commands/models.rs`.

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors
- [ ] Grep for `config::settings::Config` across the codebase to confirm no other inconsistent imports remain

**Approval Gate:** Do not proceed to Phase 12 until `cargo check` passes and imports are consistent.

---

### Phase 12: Remove unused `_toggle` and redundant `isListening`
**Priority:** LOW
**Category:** Code Smell
**Files:** `frontend/src/pages/main-window.tsx`, `frontend/src/hooks/use-dictation.ts`
**Estimated Complexity:** Simple

**Context:**
In `frontend/src/pages/main-window.tsx` at line 13, the `toggle` function from `useDictation()` is destructured as `_toggle` to suppress the unused variable warning — meaning the main window has no toggle button and relies entirely on the global hotkey. The `isListening` state in `frontend/src/hooks/use-dictation.ts` at line 7 is set but never read by any consumer — `status === "listening"` provides the same information.

**Instructions:**
1. In `frontend/src/pages/main-window.tsx`, at line 13, simplify the destructure:
   ```typescript
   const { status } = useDictation();
   ```
   Remove `toggle: _toggle` since it's unused.
2. In `frontend/src/hooks/use-dictation.ts`, remove the `isListening` state variable (line 7), its setter calls, and remove it from the return object (line 31). The `status` field already provides this information via `status === "listening"`.

**Verification:**
- [ ] Run `npm run build` from root — no TypeScript errors
- [ ] Confirm no other files import or use `isListening` from `useDictation` (search for `isListening` in `frontend/src/`)
- [ ] Run `npm run tauri dev` — dictation toggle via hotkey still works, status indicator still updates

**Approval Gate:** Do not proceed to Phase 13 until the build passes and dictation status still works.

---

## Section D: Dead Code Removal (Phases 13-16)

---

### Phase 13: Remove dead backend stubs: storage.rs and hotkey/manager.rs
**Priority:** LOW
**Category:** Dead Code
**Files:** `backend/src/model_manager/storage.rs`, `backend/src/model_manager/mod.rs`, `backend/src/hotkey/manager.rs`, `backend/src/hotkey/mod.rs`
**Estimated Complexity:** Simple

**Context:**
`backend/src/model_manager/storage.rs` contains a stub `list()` function with a TODO and `println!` that is never called from anywhere. `backend/src/hotkey/manager.rs` contains only comments — the actual hotkey logic is inline in `lib.rs`. Both are dead code that adds noise to the codebase.

**Instructions:**
1. Delete the file `backend/src/model_manager/storage.rs`.
2. In `backend/src/model_manager/mod.rs`, remove the line `pub mod storage;`.
3. Delete the file `backend/src/hotkey/manager.rs`.
4. In `backend/src/hotkey/mod.rs`, remove the line `pub mod manager;`. If this leaves the file empty or with only a comment, consider whether the entire `hotkey` module should be removed. Check `backend/src/lib.rs` for `mod hotkey;` — if nothing else is in the hotkey module, remove `mod hotkey;` from `lib.rs` and delete the `backend/src/hotkey/` directory entirely.

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors (no "module not found" or "unresolved import" errors)
- [ ] Confirm the deleted files no longer exist: `ls backend/src/model_manager/storage.rs` and `ls backend/src/hotkey/manager.rs` should both fail
- [ ] Run `npm run tauri dev` — app launches and hotkey still works

**Approval Gate:** Do not proceed to Phase 14 until `cargo check` passes and the app still works.

---

### Phase 14: Remove unused backend methods and hound dependency
**Priority:** LOW
**Category:** Dead Code
**Files:** `backend/src/audio/capture.rs`, `backend/src/audio/vad.rs`, `backend/src/audio/buffer.rs`, `backend/Cargo.toml`
**Estimated Complexity:** Simple

**Context:**
Several methods across the audio module are defined but never called: `AudioCapture::stop()` (line 72-74), `AudioCapture::is_active()` (line 76-78), `VoiceActivityDetector::reset()` (line 69-73), `AudioBuffer::clear()` (line 73-77). The `hound` crate (WAV file I/O) is listed in `Cargo.toml` at line 28 but never imported or used anywhere.

**Instructions:**
1. In `backend/src/audio/capture.rs`, delete the `stop()` method (lines 72-74) and `is_active()` method (lines 76-78).
2. In `backend/src/audio/vad.rs`, delete the `reset()` method (lines 69-73).
3. In `backend/src/audio/buffer.rs`, delete the `clear()` method (lines 73-77).
4. In `backend/Cargo.toml`, remove the line `hound = "3"` (line 28).

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors (no "method not found" errors, confirming nothing called these methods)
- [ ] Run `cargo build` from `backend/` — builds successfully without hound
- [ ] Run `npm run tauri dev` — app still works normally

**Approval Gate:** Do not proceed to Phase 15 until `cargo check` and `cargo build` pass cleanly.

---

### Phase 15: Remove dead frontend page and unused state
**Priority:** LOW
**Category:** Dead Code
**Files:** `frontend/src/pages/setup.tsx`, `frontend/src/hooks/use-models.ts`
**Estimated Complexity:** Trivial

**Context:**
`frontend/src/pages/setup.tsx` contains a stub `SetupPage` component that is never imported — the actual setup wizard is at `frontend/src/components/setup-wizard/index.tsx`. The `downloadProgress` state in `frontend/src/hooks/use-models.ts` at line 7 is returned from the hook but never consumed by any component.

**Instructions:**
1. Delete the file `frontend/src/pages/setup.tsx`.
2. Search the codebase for any import of `SetupPage` from `pages/setup` to confirm it's truly unused. If found, remove the import.
3. In `frontend/src/hooks/use-models.ts`, remove the `downloadProgress` state variable (line 7), its setter calls (wherever `setDownloadProgress` is called), and remove it from the return object. Keep the `download-progress` event subscription if it's used for other purposes (e.g., logging), otherwise remove it too.

**Verification:**
- [ ] Run `npm run build` from root — no TypeScript errors
- [ ] Confirm `frontend/src/pages/setup.tsx` no longer exists
- [ ] Run `npm run tauri dev` — setup wizard and model downloads still work correctly (progress is still shown via the setup wizard's own state management, not via the deleted hook state)

**Approval Gate:** Do not proceed to Phase 16 until `npm run build` passes and the setup wizard still works.

---

### Phase 16: Remove unused shadcn UI components
**Priority:** LOW
**Category:** Dead Code
**Files:** `frontend/src/components/ui/button.tsx`, `card.tsx`, `select.tsx`, `dropdown-menu.tsx`, `dialog.tsx`, `scroll-area.tsx`, `badge.tsx`, `separator.tsx`, `tooltip.tsx`, `progress.tsx`
**Estimated Complexity:** Simple

**Context:**
The project has 10+ shadcn UI components installed in `frontend/src/components/ui/` that are never imported or rendered anywhere in the application. These were likely installed during initial project setup and add unnecessary code weight. The only UI components actually used are the ones imported by active components (check imports before deleting).

**Instructions:**
1. Before deleting, verify each component is truly unused by searching for imports across the codebase. For each file below, run a search for its component name in `frontend/src/`:
   - `button.tsx` → search for `from.*ui/button`
   - `card.tsx` → search for `from.*ui/card`
   - `select.tsx` → search for `from.*ui/select`
   - `dropdown-menu.tsx` → search for `from.*ui/dropdown-menu`
   - `dialog.tsx` → search for `from.*ui/dialog`
   - `scroll-area.tsx` → search for `from.*ui/scroll-area`
   - `badge.tsx` → search for `from.*ui/badge`
   - `separator.tsx` → search for `from.*ui/separator`
   - `tooltip.tsx` → search for `from.*ui/tooltip`
   - `progress.tsx` → search for `from.*ui/progress`
2. Delete only the files that have zero imports. Do NOT delete files that are imported somewhere.
3. After deletion, check if any `@radix-ui/*` packages in `frontend/package.json` are only used by the deleted components. If so, remove those packages. If any radix packages are still used by remaining components, keep them.
4. Similarly, check if `lucide-react` is still used by any remaining component. If not, remove it from `package.json`.
5. Run `npm install` from root after modifying `package.json` to update the lockfile.

**Verification:**
- [ ] Run `npm run build` from root — no TypeScript errors (no "module not found" for deleted components)
- [ ] Run `npm run tauri dev` — app renders correctly, no missing components
- [ ] Confirm the deleted component files no longer exist in `frontend/src/components/ui/`

**Approval Gate:** Do not proceed to Phase 17 until the build passes and the app renders correctly.

---

## Section E: Optimizations (Phases 17-18)

---

### Phase 17: Eliminate heap allocation in audio callback
**Priority:** MEDIUM
**Category:** Optimization
**Files:** `backend/src/audio/capture.rs`, `backend/Cargo.toml`
**Estimated Complexity:** Moderate

**Context:**
In `backend/src/audio/capture.rs` at line 57, `data.to_vec()` allocates a new `Vec<f32>` on the heap on every audio callback invocation (typically every 5-10ms). The cpal audio callback runs on a real-time audio thread where heap allocations can cause audio glitches (xruns/dropouts). While the `mpsc::Sender` is already not lock-free, reducing allocations in the callback improves latency.

**Instructions:**
1. Add the `ringbuf` crate to `backend/Cargo.toml`:
   ```toml
   ringbuf = "0.4"
   ```
2. In `backend/src/audio/capture.rs`, replace the `mpsc::channel()` with a `ringbuf::HeapRb` (heap-allocated ring buffer with lock-free producer/consumer). Create it with a capacity of at least 1 second of audio at the capture sample rate (e.g., `48000` samples).
3. In the `build_input_stream` callback, use the ring buffer producer's `push_slice()` to write samples without allocation. Drop samples if the buffer is full (producer will return the count of written samples).
4. On the consumer side (in the transcription thread or wherever `receiver.recv()` currently reads), read from the ring buffer consumer in a loop with a small sleep between reads (e.g., 10ms).
5. Alternatively, if the `mpsc` channel approach is simpler to maintain, a less invasive fix is to use a pre-allocated buffer pool: create a `Vec<Vec<f32>>` of pre-allocated buffers and cycle through them, avoiding `to_vec()` allocation in the hot path.

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors
- [ ] Run `npm run tauri dev`, start dictation, speak for 30 seconds continuously
- [ ] Monitor for audio xruns/glitches — compare against the pre-fix behavior. On PipeWire/ALSA, check `pw-top` or `cat /proc/asound/card*/pcm*/sub*/xrun_debug` for xrun counts
- [ ] Confirm transcription still produces correct text

**Approval Gate:** Do not proceed to Phase 18 until audio capture works without regression and ideally with fewer xruns.

---

### Phase 18: Make model registry static
**Priority:** LOW
**Category:** Optimization
**Files:** `backend/src/transcription/models.rs`
**Estimated Complexity:** Simple

**Context:**
In `backend/src/transcription/models.rs` at line 13, `get_model_registry()` creates a new `Vec<WhisperModel>` with 5 structs and `String` allocations on every call. This function is called from `list_models`, `download_model`, `delete_model`, `is_model_downloaded`, and `load_model`. While the performance impact is negligible, the registry is constant data and should be allocated once.

**Instructions:**
1. In `backend/src/transcription/models.rs`, replace the `get_model_registry()` function with a static using `std::sync::LazyLock`:
   ```rust
   use std::sync::LazyLock;

   static MODEL_REGISTRY: LazyLock<Vec<WhisperModel>> = LazyLock::new(|| {
       vec![
           // ... existing WhisperModel entries ...
       ]
   });

   pub fn get_model_registry() -> &'static Vec<WhisperModel> {
       &MODEL_REGISTRY
   }
   ```
2. Update all call sites that use `get_model_registry()` to work with `&Vec<WhisperModel>` instead of `Vec<WhisperModel>`. Since they likely iterate over the result, this should be a seamless change (iterating over `&Vec` and `Vec` both work). Check:
   - `backend/src/commands/models.rs` — `list_models`, `download_model`, `delete_model`, `load_model`
   - `backend/src/model_manager/download.rs` — `is_model_downloaded` (if it calls `get_model_registry`)
3. If `WhisperModel` doesn't implement `Clone`, callers that need owned copies may need adjustment. But since most callers just search/filter the list, returning a reference should work.

**Verification:**
- [ ] Run `cargo check` from `backend/` — no compilation errors (especially no lifetime or borrow errors)
- [ ] Run `npm run tauri dev`, go to model management, confirm list_models still returns the correct model list
- [ ] Confirm model download and deletion still work

**Approval Gate:** This is the final phase. After verification, proceed to the Post-Fix Validation.

---

## Post-Fix Validation

After all 18 phases are complete, perform a full end-to-end test of the entire application pipeline:

1. **Clean start:** Delete `~/.whispertype/` directory to simulate a fresh install. Launch the app with `npm run tauri dev`.
2. **Setup wizard:** Confirm the setup wizard appears. Verify GPU detection shows your GPU name and VRAM. Select the `tiny` model. Download it — confirm the progress bar shows download progress and completes successfully.
3. **Model auto-load:** After completing the setup wizard, confirm the main window appears and the status indicator shows the `tiny` model is loaded (not "No model loaded").
4. **Dictation start:** Press Ctrl+Shift+Space. Confirm the status indicator changes to "Listening" (or equivalent). Confirm no errors appear in the terminal or the UI.
5. **Transcription:** Speak clearly for 10 seconds (e.g., "The quick brown fox jumps over the lazy dog"). Confirm transcription text appears in the transcript display area within a few seconds of speaking.
6. **Text output:** Open a text editor (e.g., gedit, kate, or a terminal). Start dictation with the text editor focused. Speak a sentence. Confirm the text is typed into the text editor (or copied to clipboard, depending on the output mode setting).
7. **Dictation stop:** Press Ctrl+Shift+Space again. Confirm the status indicator returns to "Idle". Confirm no orphan threads or errors in the terminal.
8. **Error handling:** Temporarily set the output mode to an invalid value or unplug the microphone mid-dictation. Confirm the UI shows an error message instead of silently failing.
9. **Settings:** Open the settings panel. Change a setting (e.g., output mode). Close and reopen the app. Confirm the setting persisted.
10. **Window title:** Confirm the window title bar shows "WhisperType".
11. **CSP:** Open browser DevTools (F12) in the Tauri webview. Confirm no CSP violations in the console.
12. **Build check:** Run `cargo check` from `backend/` — no warnings or errors. Run `npm run build` from root — no TypeScript errors.

If all 12 checks pass, the audit fix plan is complete.
