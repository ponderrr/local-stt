# REVIEW-TACTICS.md -- Tactical Intelligence per Phase
## Companion to PHASES-FIX.md

Each entry below maps 1:1 to a phase in PHASES-FIX.md. Read both files together before executing any phase.

---

## Phase 1: Fix audio capture to use device-native format with resampling

### GOTCHA ALERTS

1. **cpal `default_input_config()` returns `SupportedStreamConfig`, not `StreamConfig`.** You must call `.config()` on it (or `.sample_format()`, `.channels()`, `.sample_rate()`) to extract the usable `StreamConfig`. The `SupportedStreamConfig` also carries a `sample_format` field (e.g., `F32`, `I16`, `U16`) -- you MUST match this when calling `build_input_stream`. If the device's native format is `I16`, you cannot call `build_input_stream::<f32>` -- you must either use `build_input_stream::<i16>` and convert, or use `build_input_stream` with `SampleFormat` dispatch. On PipeWire, `F32` is the common default, but do not assume this.

2. **PipeWire default sample rate is almost always 48000Hz, not 44100Hz.** The `AudioRingBuffer` in `buffer.rs` is constructed at line 54 of `audio/mod.rs` with `AudioRingBuffer::new(16000, ...)`. After this phase, if resampling happens inside the pipeline thread, the buffer must be created with the TARGET rate (16000) since it receives resampled data. If resampling happens outside the pipeline thread (before the channel send), the buffer is fine as-is. Be clear about WHERE resampling occurs relative to the ring buffer.

3. **cpal audio callback is a realtime thread.** The current code already does `data.to_vec()` (heap alloc) and `sender.send()` (mutex-based mpsc) inside the callback. Phase 17 addresses this, but for Phase 1, do NOT add resampling logic inside the cpal callback. Resampling involves iteration and floating-point math that is fine for realtime, but the allocation patterns matter. If you resample inside the callback, you must pre-allocate the output buffer. The safest place for resampling is in the `audio/mod.rs` pipeline thread, between `audio_rx.recv()` and `buffer.write()`.

4. **Stereo-to-mono conversion ratio trap.** If the device reports 2 channels and 48000Hz, you receive interleaved stereo data: `[L0, R0, L1, R1, ...]`. The data slice length is `frames * channels`. After mono conversion the sample count is `data.len() / channels`. After resampling from 48000 to 16000 (ratio 3:1), the output length is `mono_samples.len() / 3`. A 10ms callback at 48000Hz stereo gives 960 samples (480 frames). After mono: 480 samples. After resample to 16000Hz: 160 samples. Validate these ratios in your implementation.

5. **`AudioCapture::new()` currently hardcodes `sample_rate: 16000`.** PHASES-FIX.md says to either remove the parameter or repurpose it as the target rate. The `sample_rate` field is read at line 49 (`SampleRate(self.sample_rate)`) -- this line is being replaced. But `sample_rate` is also `pub` and could be read externally. Grep for `capture.sample_rate` or `AudioCapture` usage outside `capture.rs`. Currently, `AudioPipeline::start()` in `audio/mod.rs` constructs `AudioCapture::new()` at line 44 and never reads `sample_rate`. Safe to change.

6. **The `device.default_input_config()` call can fail** on some PipeWire configurations, particularly when using Bluetooth audio or when PipeWire restarts. Always `map_err` the result. It returns `Result<SupportedStreamConfig, DefaultStreamConfigError>`.

7. **Linear interpolation resampling formula.** For downsampling from `src_rate` to `dst_rate`:
   ```
   ratio = src_rate / dst_rate  (e.g., 48000/16000 = 3.0)
   output_len = (input_len as f64 / ratio) as usize
   for i in 0..output_len:
       src_pos = i as f64 * ratio
       idx = src_pos as usize
       frac = src_pos - idx as f64
       output[i] = input[idx] * (1.0 - frac) + input[min(idx+1, input.len()-1)] * frac
   ```
   Do NOT use `input[idx+1]` without bounds checking -- the last sample will index out of bounds.

### ESCAPE HATCHES

- **If `build_input_stream` fails with "stream configuration not supported":** Run `pactl list sources short` to see PipeWire's reported sources and their sample rates. Then print the `SupportedStreamConfig` returned by `default_input_config()` to stderr: `eprintln!("device config: {:?}", supported_config);`. The supported config's sample rate and channels must match what you pass to `build_input_stream`.

- **If audio callback never fires after changing the config:** Check if the `Stream` is being dropped prematurely. The `Stream` must be stored in `self.stream` and kept alive. Also verify with `pw-top` that the PipeWire stream is actually running (you should see a new stream appear when capture starts).

- **If resampled audio sounds like chipmunks or slowed down:** The resample ratio is inverted. If you are downsampling from 48000 to 16000, the ratio is 3.0 (every 3 source samples become 1 output sample). If the audio sounds sped up, you are probably upsampling by mistake.

- **If transcription returns empty segments after this change:** Log the first 20 sample values of the resampled audio chunk right before it enters `engine.transcribe()`. If they are all 0.0, the stereo-to-mono conversion is wrong (possibly averaging L and R channels that are interleaved incorrectly). If they are non-zero but transcription is empty, the audio may be too short -- whisper needs at least ~1 second of audio to produce output.

- **If `cargo check` fails with `SupportedStreamConfig` not found:** Make sure you import it: `use cpal::SupportedStreamConfig;`. It lives in the `cpal` crate root, not in `cpal::traits`.

### PATTERN RECOMMENDATIONS

- Put the resampling function in a new helper in `audio/capture.rs` or a standalone function in `audio/mod.rs`. Signature: `fn resample_and_mono(input: &[f32], src_channels: u16, src_rate: u32, dst_rate: u32) -> Vec<f32>`. This keeps it testable.
- Pass `src_channels` and `src_rate` from the capture thread to the pipeline thread. The simplest way: add fields to `AudioCapture` that store the device's actual rate and channels after `start()` succeeds, then read them in the pipeline. OR, change the `mpsc::channel` to carry a struct `AudioChunk { samples: Vec<f32>, channels: u16, sample_rate: u32 }` -- but this is heavier. Since the rate/channels are constant for a given capture session, storing them once is better.
- The `AudioPipeline::start()` method in `audio/mod.rs` creates `AudioCapture` and calls `capture.start()` on the same thread (line 44-47). After `capture.start()` returns, you can read `capture.sample_rate` and `capture.channels` (if you add a channels field). Store these as local variables in the pipeline thread closure and use them for resampling.

### AGENT DIRECTIVES

- Read `backend/src/audio/mod.rs` lines 42-76 carefully. The pipeline thread is already spawned at line 42. The resampling must happen INSIDE this thread, after `audio_rx.recv()` returns samples and BEFORE `buffer.write()`.
- After editing, run `cargo check` from `/home/frosty/local-stt/backend/` immediately. Do not batch with other changes.
- Test this phase in isolation before proceeding. If the cpal stream does not start, nothing downstream works.
- The `AudioRingBuffer::new(16000, ...)` call at line 54 should remain 16000 because by the time data reaches the buffer, it should already be resampled to 16kHz mono.

---

## Phase 2: Surface output_text errors to frontend

### GOTCHA ALERTS

1. **Tauri v2 uses `app.emit()`, not `window.emit()`.** The code already uses `app_clone.emit(...)` correctly (see `dictation.rs` line 50-58). Follow the same pattern for the error event.

2. **`Emitter` trait must be in scope.** It already is -- `dictation.rs` line 2 imports `tauri::Emitter`. No new import needed.

3. **Event name must be kebab-case.** Use `"output-error"` (with hyphens), not `"output_error"` (underscores). The existing events use kebab-case: `"dictation-status"`, `"transcription-update"`, `"download-progress"`.

4. **The payload for `app.emit()` must be `Serialize + Clone`.** A plain `String` satisfies both. The `format!()` macro returns `String`, so `app_clone.emit("output-error", format!(...))` is correct.

5. **Frontend event listeners return a `Promise<UnlistenFn>`.** The existing pattern in `use-dictation.ts` and `use-transcription.ts` stores the promise and calls `.then(fn => fn())` in the cleanup. Follow this exact pattern for the new listener.

6. **Stale closure trap in React.** If you add an `outputError` state and subscribe to the event inside `useEffect`, the event handler captures the initial state. Use functional state updates (`setOutputError(msg)`) rather than reading `outputError` inside the handler. The existing code already does this correctly for `setStatus` in `use-dictation.ts`.

7. **enigo on Wayland will fail unless XWayland is available.** The `main.rs` already forces `GDK_BACKEND=x11`, but `enigo` checks a different env var (`DISPLAY` or `WAYLAND_DISPLAY`). On a pure Wayland session without XWayland, `Enigo::new()` will fail. This is the most common scenario where `output_text` errors occur -- making this phase's error surfacing essential.

### ESCAPE HATCHES

- **If the error event never arrives in the frontend:** Add `eprintln!("Emitting output-error: {}", e);` right before the `app_clone.emit("output-error", ...)` call. If you see it in the terminal but not in the frontend, the event name is mismatched. Compare the Rust `"output-error"` string with the TypeScript `"output-error"` string character by character.

- **If TypeScript complains about the event listener type:** The `listen<T>()` generic parameter specifies the payload type. For a string payload: `listen<string>("output-error", (event) => handler(event.payload))`.

- **If the error clears too fast or never clears:** Use `setTimeout` with a ref to clear the error. Store the timeout ID and clear it on unmount or on new error:
  ```typescript
  const timeoutRef = useRef<ReturnType<typeof setTimeout>>();
  // In handler:
  if (timeoutRef.current) clearTimeout(timeoutRef.current);
  setOutputError(msg);
  timeoutRef.current = setTimeout(() => setOutputError(null), 5000);
  ```

### PATTERN RECOMMENDATIONS

- Add the error event to `events` object in `frontend/src/lib/tauri.ts` alongside the existing event listeners. This keeps all event wiring in one place.
- For the UI, a simple conditional `<div>` below the `StatusIndicator` in `main-window.tsx` is sufficient. Do not over-engineer a toast system for this.
- Consider making the error handling reusable for Phase 3 (transcription errors). You could create a single `useErrorEvents()` hook that listens to both `"output-error"` and `"transcription-error"`, or add both error states to `useDictation`.

### AGENT DIRECTIVES

- Edit three files: `backend/src/commands/dictation.rs` (Rust error emission), `frontend/src/lib/tauri.ts` (event type), `frontend/src/hooks/use-dictation.ts` (state + listener), and `frontend/src/pages/main-window.tsx` (display). That is four files total.
- Run `cargo check` from `backend/` after the Rust change, then `npm run build` from root after the TypeScript changes. Do not batch these checks.

---

## Phase 3: Surface transcription errors to frontend

### GOTCHA ALERTS

1. **Same event emission pattern as Phase 2.** The `app_clone` is already available inside the transcription thread (line 42 of `dictation.rs`). Just add `app_clone.emit("transcription-error", ...)` inside the existing `Err(e)` branch at line 61-63.

2. **Keep the `eprintln!` alongside the emit.** The stderr log is useful for debugging even after adding the frontend event. Do not remove it.

3. **Transcription errors can be rapid-fire.** If the model is corrupted or audio is malformed, every chunk will fail. Consider debouncing on the frontend or only showing the most recent error, not accumulating them.

4. **GPU OOM errors from whisper-rs are not recoverable.** If you see "CUDA out of memory" in the error, the model must be unloaded and a smaller one loaded. The error message should be clear enough for the user to act on.

### ESCAPE HATCHES

- **If the error event fires but the text is unhelpful (e.g., "Transcription failed: Unknown error"):** Add more context by logging `audio_data.len()` and the first few sample values before the `engine.transcribe()` call. This helps distinguish between "bad audio" and "bad model".

- **If errors spam the UI:** Add a simple rate limiter: only emit the error if the last emission was more than 2 seconds ago. Use a `std::time::Instant` stored outside the loop:
  ```rust
  let mut last_error_emit = std::time::Instant::now() - std::time::Duration::from_secs(10);
  // In the Err branch:
  if last_error_emit.elapsed() > std::time::Duration::from_secs(2) {
      app_clone.emit("transcription-error", ...).ok();
      last_error_emit = std::time::Instant::now();
  }
  ```

### PATTERN RECOMMENDATIONS

- If you built a reusable error display mechanism in Phase 2, extend it here rather than creating a second one.
- Use the same 5-second auto-clear pattern from Phase 2.

### AGENT DIRECTIVES

- This phase is simple. Edit `backend/src/commands/dictation.rs` (add one emit call), `frontend/src/lib/tauri.ts` (add event), `frontend/src/hooks/use-dictation.ts` (add listener). Optionally update `main-window.tsx` if the error display is not already shared from Phase 2.
- Run `cargo check` then `npm run build`. Quick phase.

---

## Phase 4: Track and join transcription thread on toggle

### GOTCHA ALERTS

1. **The `AppState` struct is in `commands/dictation.rs` at line 9-13.** Adding a new field here means updating the construction site in `lib.rs` at line 19-23. Both files must be edited.

2. **`JoinHandle<()>` is NOT `Send + Sync` by default.** Wait -- actually it IS. `JoinHandle<T>` is `Send` when `T: Send`, and `()` is `Send`. It is also `Sync`. So wrapping it in `Mutex<Option<JoinHandle<()>>>` is fine.

3. **The `Mutex` wrapping the `JoinHandle` must NOT be held while calling `handle.join()`.** The `join()` call blocks until the thread finishes. If you hold the Mutex lock during `join()`, and the transcription thread tries to access any other `Mutex` in `AppState` (it accesses `engine.ctx` via `engine.transcribe()`), you risk a deadlock IF the thread ordering creates a lock cycle. In this case, the transcription thread does NOT lock `AppState.transcription_thread`, so there is no cycle. But for safety, always `.take()` the handle out of the Mutex first, then drop the lock, then join:
   ```rust
   let handle = state.transcription_thread.lock().unwrap().take();
   if let Some(h) = handle {
       h.join().ok();
   }
   ```

4. **When `pipeline.stop()` is called, it sets `is_running` to `false`.** This causes the pipeline thread (in `audio/mod.rs` line 57) to exit its loop. The pipeline thread holds the `audio_tx` sender. When that thread exits, `audio_tx` is dropped, so `audio_rx.recv()` in the transcription thread... wait, no. Look at the architecture again: the pipeline thread in `audio/mod.rs` reads from `audio_rx` and sends to `chunk_tx`. The transcription thread in `dictation.rs` reads from `chunk_rx` (which is the `receiver` at line 45). When the pipeline thread exits, it drops `chunk_tx`, which causes `receiver.recv()` in the transcription thread to return `Err(RecvError)`, which exits the `while let Ok(chunk)` loop. So stopping the pipeline DOES cause the transcription thread to exit -- but there is a latency: the pipeline thread must first notice `is_running == false` (up to 100ms timeout at line 58), then exit, then the drop of `chunk_tx` propagates. So `join()` on the transcription thread handle may block for up to ~100ms. This is acceptable.

5. **On the stop-dictation path (line 17-20 of `dictation.rs`), the `pipeline.stop()` at line 18 is called, but the transcription thread is NOT joined.** PHASES-FIX.md says to also join here. This is important for the `stop_dictation` command (line 87-93) which calls `pipeline.stop()` without going through `toggle_dictation_inner`. You need to join in BOTH the stop branch of `toggle_dictation_inner` AND in the `stop_dictation` command, OR better: extract a helper that does stop + join.

6. **The `stop_dictation` command at line 87-93 does NOT access `state.transcription_thread`.** After adding the field, update `stop_dictation` to also join the thread. Otherwise rapid calls to `stop_dictation` followed by `start_dictation` can still race.

### ESCAPE HATCHES

- **If `handle.join()` hangs forever:** The transcription thread is stuck in `engine.transcribe()`. This can happen if whisper-rs enters an infinite loop on corrupted audio. Add a timeout mechanism: instead of `handle.join()`, check if the thread has finished with a short poll loop, and log a warning if it takes more than 5 seconds. Unfortunately, `JoinHandle` does not support timeout. Workaround: use a shared `AtomicBool` that the transcription thread sets to `true` before exiting, and poll that bool with a timeout before calling `join()`.

- **If `cargo check` fails with "field `transcription_thread` not found in struct `AppState`":** You forgot to add the field to the struct definition at line 9-13 of `dictation.rs`, or you forgot to initialize it in `lib.rs` at line 19-23.

- **If rapid toggling causes a deadlock:** Print thread IDs and Mutex acquisition attempts. The likely cause is holding the `transcription_thread` Mutex while also trying to acquire the `config` Mutex or the `engine.ctx` Mutex. Solution: take the JoinHandle out of the Mutex immediately (`.take()`), drop the Mutex guard, THEN join.

### PATTERN RECOMMENDATIONS

- Create a small helper method on `AppState` or a free function:
  ```rust
  fn join_transcription_thread(state: &AppState) {
      let handle = state.transcription_thread.lock().unwrap().take();
      if let Some(h) = handle {
          h.join().ok();
      }
  }
  ```
  Call this from both the stop branch of `toggle_dictation_inner` and from `stop_dictation`.

### AGENT DIRECTIVES

- Edit two files: `backend/src/commands/dictation.rs` (struct + toggle logic + stop logic) and `backend/src/lib.rs` (AppState construction).
- Read the entire `dictation.rs` file before editing. The struct is at line 9, the toggle logic starts at line 16, the stop command is at line 87. All three locations need changes.
- After editing, run `cargo check`. Then test rapid toggling: `npm run tauri dev`, press Ctrl+Shift+Space 10 times quickly.

---

## Phase 5: Fix model auto-load race condition after setup

### GOTCHA ALERTS

1. **The race condition:** `useModels` calls `refresh()` in a `useEffect` on mount (line 20-37 of `use-models.ts`). This is async. Meanwhile, `MainWindow` renders and its `useEffect` (line 18-28 of `main-window.tsx`) runs synchronously during the same render cycle. At that point, `models` is still `[]` (initial state), so `models.find(...)` returns `undefined`, and the auto-load never triggers. The effect DOES have `models` in its dependency array, so it WILL re-run when `models` updates. But the timing depends on React's scheduler -- in StrictMode (which is enabled in `main.tsx` line 7), effects run twice in dev, which may mask the bug in development but not in production builds.

2. **Adding `loading` state to `useModels`:** Initialize as `true`, set to `false` in the `refresh()` callback after setting `models` and `activeModel`. The guard `if (loading || ...) return;` ensures the effect waits for the first fetch.

3. **The `useCallback` memoization of `loadModel` (line 39-46 of `use-models.ts`) has an empty dependency array.** This means `loadModel`'s identity is stable across renders. If you add `loading` to the return value, it should NOT appear in `loadModel`'s deps. No change needed to `loadModel`.

4. **If `activeModel` is already set (e.g., from a previous session), the auto-load effect should NOT trigger.** The existing guard `!activeModel` handles this. But after adding `loading`, ensure the guard is: `if (loading || !config || activeModel) return;` -- note that `activeModel` is truthy when a model is active, so this correctly skips.

5. **React StrictMode double-firing effects:** In development (`npm run tauri dev`), the effect may fire twice. The `loadModel` call is idempotent (loading the same model twice just re-loads it), so this is not harmful but may cause a brief "Loading..." flash. This is a dev-only behavior.

### ESCAPE HATCHES

- **If the model still does not auto-load after the fix:** Add `console.log` inside the effect: log `loading`, `config`, `models`, `activeModel` to see which guard is blocking. If `models` is `[]` even after `loading` becomes `false`, the `refresh()` call is failing silently -- check the browser console for errors.

- **If TypeScript complains about the new `loading` return value:** Make sure you add `loading` to the return object of `useModels`: `return { models, activeModel, loadModel, downloadModel, deleteModel, downloadProgress, refresh, loading };`. Then destructure it in `main-window.tsx`: `const { models, activeModel, loadModel, loading } = useModels();`.

### PATTERN RECOMMENDATIONS

- Use the `loading` flag pattern consistently. A simple boolean is sufficient here; no need for a loading/error/success enum.
- Set `loading = false` AFTER both `setModels` and `setActiveModel` complete in `refresh()`. Since both are synchronous state updates within the same async function, they batch in React 18+.

### AGENT DIRECTIVES

- Edit two files: `frontend/src/hooks/use-models.ts` (add `loading` state) and `frontend/src/pages/main-window.tsx` (consume `loading` in effect guard).
- Run `npm run build` after both edits. Quick phase.

---

## Phase 6: Set Content Security Policy

### GOTCHA ALERTS

1. **The CSP must allow connections to `huggingface.co` for model downloads.** The `download.rs` fetches from `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/...`. Without `connect-src` including `https://huggingface.co` and `https://*.huggingface.co`, downloads will fail. BUT: the download happens in the Rust backend via `reqwest`, NOT in the webview. The CSP only governs the webview. So actually, the `connect-src` only matters for any fetch/XHR calls made from JavaScript. Since all downloads go through Tauri commands (which invoke Rust code), the CSP `connect-src` may not need HuggingFace at all. However, include it anyway for safety in case future frontend code makes direct API calls.

2. **`'unsafe-inline'` for `style-src` is needed.** Tailwind CSS v4 (used here, see `package.json` line 21: `"tailwindcss": "^4.1.18"`) injects styles. Without `'unsafe-inline'`, the entire UI will be unstyled.

3. **`ipc:` and `http://ipc.localhost` are needed for `connect-src` in Tauri v2.** These are the internal protocols Tauri uses for IPC communication between the webview and the Rust backend. Without them, all `invoke()` calls from the frontend will fail silently.

4. **`asset:` and `https://asset.localhost` in `img-src` are for Tauri asset protocol.** If the app ever loads local images through Tauri's asset protocol, these are needed. Including them is defensive.

5. **After setting the CSP, test the ENTIRE app flow.** CSP violations are silent by default (blocked resources just fail). Open the browser DevTools (F12 in the Tauri webview) and check the Console tab for CSP violation messages.

### ESCAPE HATCHES

- **If the app loads but the UI is completely unstyled (no colors, no layout):** The `style-src` is too restrictive. Ensure `'unsafe-inline'` is included. For Tailwind v4, you may also need `'unsafe-eval'` in `script-src` if it uses runtime evaluation (unlikely but possible).

- **If `invoke()` calls fail after adding CSP:** The `connect-src` is missing `ipc:` or `http://ipc.localhost`. Check the Tauri v2 documentation for the exact CSP directives needed for IPC.

- **If model downloads fail after adding CSP:** Verify whether the download happens in Rust (via reqwest) or JavaScript (via fetch). If Rust, CSP is irrelevant. If the download progress events stop working, that is a different issue.

- **General debugging:** Open DevTools with F12, go to the Console tab. Any CSP violation will be logged as `Refused to load ...`. The error message tells you exactly which directive to adjust.

### PATTERN RECOMMENDATIONS

- Start with a permissive CSP and tighten incrementally. If in doubt, temporarily add `'unsafe-eval'` to `script-src` and `*` to `connect-src`, verify the app works, then remove the permissive directives one by one.
- The CSP string goes in `backend/tauri.conf.json` at `app.security.csp`. It is a single string, not an object.

### AGENT DIRECTIVES

- Edit one file: `backend/tauri.conf.json`. Replace `"csp": null` with the CSP string.
- Test by running `npm run tauri dev`. Open DevTools (F12) and check for CSP violations.
- This is a configuration-only change. No `cargo check` needed, but run `npm run tauri dev` to validate.

---

## Phase 7: Fix index.html title

### GOTCHA ALERTS

1. **Tauri v2 uses the `title` field in `tauri.conf.json` for the native window title.** See `backend/tauri.conf.json` line 15: `"title": "WhisperType"`. This is already correct. The `index.html` `<title>` tag affects the HTML document title, which is visible in the taskbar tooltip and in any tab-like UI. On Linux with WebKitGTK, the HTML title may override the window title in some desktop environments.

2. **The file is at the project root:** `/home/frosty/local-stt/index.html`, NOT in `frontend/` or `backend/`.

### ESCAPE HATCHES

- **If the window title still shows "Tauri + React + Typescript" after changing index.html:** Clear the Vite cache (`rm -rf node_modules/.vite`) and restart `npm run tauri dev`. The old HTML may be cached by the dev server.

### PATTERN RECOMMENDATIONS

- None. This is a one-line change.

### AGENT DIRECTIVES

- Edit `/home/frosty/local-stt/index.html` line 7. Change the `<title>` tag content. Done.

---

## Phase 8: Fix StepComplete placeholder text

### GOTCHA ALERTS

1. **PHASES-FIX.md suggests using an HTML entity `&#10003;` (checkmark) or an emoji.** Note that the instructions say "Or use a Unicode speech/microphone emoji if preferred." The user prompt says to avoid emojis in output. However, the PHASES-FIX.md is the plan being executed, not agent output. If the plan says to use a Unicode character in source code, that is fine -- it is a UI element, not agent communication. Use the HTML entity `&#10003;` (checkmark) for maximum compatibility rather than an emoji that may not render on all systems.

2. **The file is at:** `frontend/src/components/setup-wizard/step-complete.tsx` line 8.

### ESCAPE HATCHES

- **If the checkmark does not render:** Use the Unicode escape directly in JSX: `{'\u2713'}` or `{'\u2714'}` (heavier checkmark). Or use an SVG checkmark icon.

### PATTERN RECOMMENDATIONS

- A simple HTML entity is the lowest-risk change. Do not add a dependency (like an icon library) for a single checkmark.

### AGENT DIRECTIVES

- Edit one file, one line. Run `npm run build` to verify no TypeScript errors. Done.

---

## Phase 9: Remove unnecessary unsafe block in main.rs

### GOTCHA ALERTS

1. **CRITICAL: In Rust edition 2024 (1.85+), `std::env::set_var` IS unsafe.** However, this project uses edition 2021 (see `backend/Cargo.toml` line 6: `edition = "2021"`). In edition 2021, `set_var` is safe. BUT: if `rustup` has a newer default edition or if the project is upgraded later, this change would need to be reverted. For now, removing `unsafe` is correct per the plan.

2. **Actually, check the Rust version more carefully.** Starting from Rust 1.66, `set_var` emits a deprecation warning encouraging `unsafe` usage. In Rust 1.83+, `set_var` is actually marked as `unsafe` regardless of edition. Run `rustc --version` to check. If the Rust compiler is 1.83+, removing `unsafe` will cause a compilation ERROR, not just a warning. The PHASES-FIX.md claim that `unsafe` is unnecessary in edition 2021 may be outdated.

3. **Test with `cargo check` before and after.** If `cargo check` passes with the `unsafe` removed, it is safe to remove. If it fails, the `unsafe` block is required and this phase should be SKIPPED.

### ESCAPE HATCHES

- **If `cargo check` fails with "call to unsafe function `set_var` is unsafe":** The Rust toolchain version requires `unsafe` for `set_var`. SKIP this phase entirely. The `unsafe` block is actually necessary. Add a comment explaining why:
  ```rust
  // SAFETY: set_var is unsafe in Rust 1.83+ because it is not thread-safe.
  // We call it before any threads are spawned (main() entry point).
  ```

- **If removing `unsafe` produces a warning instead of an error:** Run `cargo check` and look for warnings. Warnings are acceptable -- the code compiles. But if the warning says "use of deprecated function," consider keeping the `unsafe` with a safety comment.

### PATTERN RECOMMENDATIONS

- If `unsafe` is removed successfully, keep the `#[cfg(target_os = "linux")]` block with braces for scoping.
- The env vars must be set before any GTK/WebKit initialization, which happens inside `tauri_app_lib::run()`. Setting them in `main()` before `run()` is correct.

### AGENT DIRECTIVES

- **FIRST:** Run `rustc --version` from `/home/frosty/local-stt/backend/` to check the compiler version.
- **SECOND:** Try removing the `unsafe` block and run `cargo check`.
- **THIRD:** If `cargo check` fails, revert and skip this phase. Document why.
- This is a hot-path-free change, but edition/compiler version matters enormously.

---

## Phase 10: Fix unnecessary type cast in StepGpu

### GOTCHA ALERTS

1. **The double cast `info as unknown as GpuInfo` exists because `getGpuInfo()` returns `invoke<GpuInfo>("get_gpu_info")`, which should already return `GpuInfo`.** The cast is likely a remnant from when the return type was untyped or was `serde_json::Value`. Removing it is safe.

2. **BUT: the Rust side returns `serde_json::Value` (see `commands/system.rs` line 9: `get_gpu_info() -> Result<serde_json::Value, String>`).** The TypeScript side calls `invoke<GpuInfo>("get_gpu_info")`, which ASSERTS the return type is `GpuInfo` but does NOT validate at runtime. The actual JSON payload has the same shape as `GpuInfo` (keys: `name`, `vram_total_mb`, `cuda_available`), so the assertion is correct. Removing the cast is safe because TypeScript's type assertion on `invoke<GpuInfo>` already provides the correct type. The `as unknown as GpuInfo` was double-insurance that is now unnecessary.

### ESCAPE HATCHES

- **If TypeScript reports a type error after removing the cast:** The `invoke<GpuInfo>` return type might not match `GpuInfo`. Check that the Rust JSON keys exactly match the TypeScript interface fields. The Rust side uses `serde_json::json!` with keys `"name"`, `"vram_total_mb"`, `"cuda_available"` -- these match `GpuInfo` interface in `tauri.ts` lines 28-32.

### PATTERN RECOMMENDATIONS

- None. Straightforward removal.

### AGENT DIRECTIVES

- Edit one file: `frontend/src/components/setup-wizard/step-gpu.tsx` line 15. Remove `as unknown as GpuInfo`. Run `npm run build`.

---

## Phase 11: Normalize Config import paths

### GOTCHA ALERTS

1. **The import `use crate::config::Config;` works because `config/mod.rs` line 2 exports: `pub use settings::{Config, OutputMode};`.** This re-export makes `Config` accessible at `crate::config::Config`. The current import in `dictation.rs` (`use crate::config::settings::Config;`) bypasses the re-export and reaches into the submodule directly. Both work, but the re-export path is canonical.

2. **Check that `OutputMode` is not also imported via the long path.** In `dictation.rs`, `OutputMode` is used indirectly through `output::output_text(&segment.text, &output_mode)` where `output_mode` is a `Config.output_mode` value. The `OutputMode` type is not explicitly imported in `dictation.rs` -- it is carried as a field of `Config`. So only the `Config` import needs changing.

3. **Also check `model_manager/download.rs` line 1: `use crate::config::settings::Config;`.** This also uses the long path. PHASES-FIX.md only mentions `dictation.rs`, but `download.rs` has the same inconsistency. Consider fixing both, but only do what the plan says unless explicitly expanding scope.

### ESCAPE HATCHES

- **If `cargo check` fails after changing the import:** The re-export in `config/mod.rs` might not include what you need. Verify `backend/src/config/mod.rs` contains `pub use settings::Config;`.

### PATTERN RECOMMENDATIONS

- Grep for `config::settings::` across all Rust files to find all instances of the long path. Fix them all in one pass.

### AGENT DIRECTIVES

- Edit `backend/src/commands/dictation.rs` line 5. Change `use crate::config::settings::Config;` to `use crate::config::Config;`. Run `cargo check`.
- Optionally also fix `backend/src/model_manager/download.rs` line 1 and `backend/src/output/mod.rs` line 4 if they use the long path. Check first.

---

## Phase 12: Remove unused `_toggle` and `isListening`

### GOTCHA ALERTS

1. **`_toggle` is used as `toggle: _toggle` in destructuring at `main-window.tsx` line 13.** This means `toggle` IS returned by `useDictation` -- it is just unused in `MainWindow`. Simply remove it from the destructure.

2. **`isListening` is returned from `useDictation` at line 31 of `use-dictation.ts`.** Before removing it, verify no other component imports and uses `isListening`. Search for `isListening` in the entire `frontend/src/` directory.

3. **Removing `isListening` from the hook's return object is a breaking change for any consumer.** But since `MainWindow` is the only consumer of `useDictation` (verify this), it is safe.

4. **The `setIsListening` calls at lines 13 and 24 of `use-dictation.ts` should also be removed.** Removing the state variable but leaving the setter calls will cause a TypeScript error.

### ESCAPE HATCHES

- **If `npm run build` fails with "Property 'isListening' does not exist":** You removed the state but a consumer still references it. Grep for `isListening` in `frontend/src/`.

- **If removing `toggle` from the `MainWindow` destructure causes TypeScript to warn about unused exports:** That is fine. `toggle` is still in the hook's return value -- it just is not destructured in `MainWindow`. Other components could still use it. Do NOT remove `toggle` from the hook itself.

### PATTERN RECOMMENDATIONS

- Only remove `isListening` from the hook return and its internal state. Keep `toggle` in the hook's return -- it may be used by future components or by the setup wizard.

### AGENT DIRECTIVES

- Edit two files: `frontend/src/pages/main-window.tsx` (simplify destructure) and `frontend/src/hooks/use-dictation.ts` (remove `isListening` state, setters, and return value).
- Run `npm run build` after both edits.

---

## Phase 13: Remove dead backend stubs: storage.rs and hotkey/manager.rs

### GOTCHA ALERTS

1. **After deleting `storage.rs`, remove `pub mod storage;` from `model_manager/mod.rs` (line 2).** If you forget, `cargo check` will fail with "file not found for module `storage`".

2. **The `hotkey/mod.rs` contains ONLY `pub mod manager;`.** Removing `pub mod manager;` leaves `hotkey/mod.rs` empty. Check `lib.rs` line 4: `pub mod hotkey;`. Since the module is now empty, it is dead weight. Remove `pub mod hotkey;` from `lib.rs` and delete the entire `backend/src/hotkey/` directory.

3. **Verify nothing imports from `hotkey`.** Grep for `use crate::hotkey` and `mod hotkey` in the entire `backend/src/` directory. The only reference should be `lib.rs` line 4.

4. **File deletion order matters.** Delete files first, then update `mod.rs` / `lib.rs`. Or update the module declarations first, then delete. Either way, `cargo check` must pass after all changes.

### ESCAPE HATCHES

- **If `cargo check` fails with "unresolved import `crate::hotkey`":** Something imports from the hotkey module. Grep for it and remove the import.

- **If `cargo check` fails with "file not found for module":** You declared a module but deleted its file, or vice versa. Ensure the `mod` declaration and the file are either both present or both removed.

### PATTERN RECOMMENDATIONS

- Use `git rm` to delete files so they are tracked in version control.

### AGENT DIRECTIVES

- Step 1: Grep for `crate::hotkey` and `storage` usage across all Rust source files.
- Step 2: Delete files: `backend/src/model_manager/storage.rs`, `backend/src/hotkey/manager.rs`, `backend/src/hotkey/mod.rs`.
- Step 3: Edit `backend/src/model_manager/mod.rs` -- remove `pub mod storage;`.
- Step 4: Edit `backend/src/lib.rs` -- remove `pub mod hotkey;`.
- Step 5: Delete directory `backend/src/hotkey/`.
- Step 6: Run `cargo check`.

---

## Phase 14: Remove unused backend methods and hound dependency

### GOTCHA ALERTS

1. **Before deleting methods, verify they are truly unused.** Run `cargo check` with `#[warn(dead_code)]` (which is default). The compiler will warn about unused methods. Alternatively, grep for each method name across all `.rs` files.

2. **`AudioCapture::stop()` (line 72-74 of `capture.rs`):** The pipeline in `audio/mod.rs` never calls `capture.stop()`. The `AudioCapture` is dropped when the pipeline thread exits, which drops the `Stream`, which stops capture. The explicit `stop()` is dead code.

3. **`AudioCapture::is_active()` (line 76-78):** Not called anywhere. Dead code.

4. **`VoiceActivityDetector::reset()` (line 69-73 of `vad.rs`):** Not called anywhere. The VAD is created fresh each time the pipeline starts.

5. **`AudioRingBuffer::clear()` (line 73-77 of `buffer.rs`):** Not called anywhere.

6. **`hound` crate (line 28 of `Cargo.toml`):** Grep for `use hound` or `hound::` in all Rust files. It should return zero results.

7. **After removing methods, if tests reference them, those tests must be updated too.** Check the test modules at the bottom of each file. The tests in `buffer.rs` (lines 80-105) do NOT use `clear()`. The tests in `vad.rs` (lines 76-97) do NOT use `reset()`. Safe to proceed.

### ESCAPE HATCHES

- **If `cargo check` fails with "method not found":** Something calls one of the deleted methods. Grep for the method name and fix the caller.

- **If `cargo build` fails after removing `hound`:** A dependency of `hound` may be transitively required by another crate. This is unlikely since `hound` is a standalone WAV I/O library. Run `cargo build` to confirm.

### PATTERN RECOMMENDATIONS

- Remove all dead methods in a single edit pass. Then remove `hound` from `Cargo.toml`. Then `cargo check`.

### AGENT DIRECTIVES

- Edit three source files (`capture.rs`, `vad.rs`, `buffer.rs`) and one config file (`Cargo.toml`). Run `cargo check` after all edits.
- Read each file before editing to confirm the exact line numbers of the methods to delete.

---

## Phase 15: Remove dead frontend page and unused state

### GOTCHA ALERTS

1. **`frontend/src/pages/setup.tsx` exports `SetupPage`.** Grep for `import.*SetupPage` and `from.*pages/setup` across all TypeScript files. If zero results, safe to delete. The actual setup UI is in `frontend/src/components/setup-wizard/index.tsx` which exports `SetupWizard` (imported by `App.tsx` line 3).

2. **`downloadProgress` in `use-models.ts` (line 7):** This state is returned from the hook (line 66: `return { ..., downloadProgress, ... }`) but the question is whether any consumer reads it. The `MainWindow` destructures `const { models, activeModel, loadModel } = useModels();` -- it does NOT destructure `downloadProgress`. Check if any other component uses `useModels` and reads `downloadProgress`. Grep for `downloadProgress` in `frontend/src/`.

3. **The `download-progress` event listener in `use-models.ts` (lines 23-31) updates `downloadProgress` state AND calls `refresh()` when download completes.** If you remove `downloadProgress` state, the setter `setDownloadProgress` is gone, but the event listener still exists. You should keep the event listener for the `refresh()` call at line 30-31 (which updates the model list when a download finishes). Only remove the `setDownloadProgress` call, not the entire listener.

4. **After removing `downloadProgress` from the return object, update any destructuring at call sites.** Since `MainWindow` does not destructure it, no change needed there.

### ESCAPE HATCHES

- **If `npm run build` fails with "Property 'downloadProgress' does not exist":** Some component still references it. Grep and fix.

- **If the setup wizard's download progress bar stops working after this change:** The setup wizard uses its OWN local progress state in `step-download.tsx` (line 10: `const [progress, setProgress] = useState<Record<string, number>>({});`). It subscribes to the `download-progress` event independently (line 16). So removing `downloadProgress` from `use-models.ts` does NOT affect the setup wizard. They are separate subscriptions.

### PATTERN RECOMMENDATIONS

- Delete the file, remove the state, clean up the return object. Straightforward.

### AGENT DIRECTIVES

- Delete `frontend/src/pages/setup.tsx`.
- Edit `frontend/src/hooks/use-models.ts`: remove `downloadProgress` state variable, remove `setDownloadProgress` call inside the event listener, remove `downloadProgress` from return object.
- Run `npm run build`.

---

## Phase 16: Remove unused shadcn UI components

### GOTCHA ALERTS

1. **MUST verify each component is unused before deleting.** For each of the 10 files, grep for its import in the entire `frontend/src/` directory. The pattern to search for each:
   - `from.*components/ui/button` or `from.*ui/button`
   - `from.*components/ui/card` or `from.*ui/card`
   - etc.

2. **The `package.json` uses `"radix-ui": "^1.4.3"` (line 16), NOT individual `@radix-ui/*` packages.** This is the monorepo package. Shadcn components typically import from `@radix-ui/react-dialog`, `@radix-ui/react-select`, etc. But with the monorepo package, all radix components are available through `radix-ui`. If you remove all UI components, check whether any REMAINING component still uses radix-ui. If not, remove `"radix-ui"` from `package.json`.

3. **`class-variance-authority` (line 15), `clsx` (devDeps line 27), `tailwind-merge` (devDeps line 30) are utility libraries used by shadcn components.** If ALL shadcn components are removed, these may become unused. But check if any remaining component uses `cn()` from a utils file (common shadcn pattern: `lib/utils.ts`). If `cn()` is used in any remaining component, keep these dependencies.

4. **`lucide-react` (devDeps line 28) is an icon library.** Check if any remaining component imports from `lucide-react`. The app uses inline SVGs for icons (see `main-window.tsx` lines 42-59), so `lucide-react` may be entirely unused.

5. **After deleting files and modifying `package.json`, run `npm install` to update the lockfile, then `npm run build` to verify.**

### ESCAPE HATCHES

- **If `npm run build` fails with "Cannot find module './components/ui/X'":** You deleted a component that IS imported somewhere. Restore it and remove it from the delete list.

- **If removing a dependency from `package.json` breaks the build:** The dependency is used transitively. Restore it.

### PATTERN RECOMMENDATIONS

- Do this in two passes:
  1. First pass: Delete the component files. Run `npm run build`. If it passes, all deletions are safe.
  2. Second pass: Check if `radix-ui`, `class-variance-authority`, `clsx`, `tailwind-merge`, `lucide-react` are still used. Remove unused ones from `package.json`. Run `npm install && npm run build`.

### AGENT DIRECTIVES

- Grep for imports of each UI component across `frontend/src/` BEFORE deleting anything.
- Delete only confirmed-unused files.
- Run `npm run build` after deletions.
- Then audit `package.json` dependencies. Run `npm install && npm run build` after any `package.json` changes.
- Check for a `frontend/src/lib/utils.ts` file that might use `clsx` and `tailwind-merge`. If it exists and is imported by remaining components, keep those deps.

---

## Phase 17: Eliminate heap allocation in audio callback

### GOTCHA ALERTS

1. **The audio callback is at `capture.rs` line 56-58.** Currently: `sender.send(data.to_vec()).ok();`. The `data.to_vec()` allocates a new `Vec<f32>` on every callback invocation.

2. **`mpsc::Sender::send()` itself acquires an internal mutex.** Even with a lock-free ring buffer replacing `to_vec()`, the send itself is not lock-free. To make the entire path lock-free, replace BOTH the allocation AND the channel.

3. **`ringbuf` 0.4 API:** `HeapRb::new(capacity)` creates the ring buffer. `rb.split()` returns `(Producer, Consumer)`. The producer has `push_slice(&[T])` which writes without allocation. The consumer has `pop_slice(&mut [T])` which reads into a pre-allocated buffer.

4. **Capacity must be in samples, not bytes.** For 1 second at 48000Hz stereo: `48000 * 2 = 96000` samples. Use a larger buffer to handle processing delays: 2-3 seconds is safe (e.g., `48000 * 2 * 3 = 288000`).

5. **The consumer side cannot use `recv()` (blocking).** With `ringbuf`, the consumer must poll. Use `consumer.pop_slice()` in a loop with `std::thread::sleep(Duration::from_millis(10))` between polls. This changes the blocking model of the pipeline thread in `audio/mod.rs`.

6. **IMPORTANT: The `ringbuf` producer and consumer must be moved to different threads.** Producer goes into the cpal callback closure (`move |data, _| { producer.push_slice(data); }`). Consumer goes into the pipeline thread. The `Producer` and `Consumer` types from `ringbuf` are `Send` but NOT `Sync` -- they can be moved to another thread but not shared. This is correct for the use case.

7. **The `move` closure in `build_input_stream` currently captures `sender`.** After this change, it captures `producer` instead. The `producer` must be `Send` -- `ringbuf`'s producer is `Send`. Confirmed.

8. **Dropped samples:** If the pipeline thread falls behind, the ring buffer fills up and `push_slice` returns the number of samples written (which may be less than `data.len()`). Dropped audio samples are acceptable for speech -- a few ms of lost audio is imperceptible. Log dropped samples for debugging: `let written = producer.push_slice(data); if written < data.len() { /* log */ }`. But do NOT use `eprintln!` in the callback (it allocates and may block). Use an `AtomicUsize` counter instead.

9. **After Phase 1 (which adds resampling), the pipeline thread structure may have changed.** This phase should be aware of the resampling logic added in Phase 1. The ring buffer should carry raw device-format data (before resampling), and resampling should happen on the consumer side (pipeline thread), not in the audio callback.

### ESCAPE HATCHES

- **If audio quality degrades (clicks, pops):** The ring buffer capacity is too small or the consumer is polling too slowly. Increase capacity and decrease sleep time.

- **If `cargo check` fails with `Producer` not implementing some trait:** Check the `ringbuf` version. Version 0.4 has different type names than 0.3. In 0.4, the types are `HeapProducer<f32>` and `HeapConsumer<f32>` (or `Prod<Arc<HeapRb<f32>>>` and `Cons<Arc<HeapRb<f32>>>`). Check the `ringbuf` 0.4 documentation.

- **If the pipeline thread never receives data:** The producer and consumer may have been split from different ring buffer instances. They MUST come from the same `HeapRb::new(n).split()` call.

- **Alternative escape hatch: bounded channel.** If `ringbuf` proves too complex, use `crossbeam_channel::bounded(N)` with pre-allocated `Box<[f32; CHUNK_SIZE]>` objects. This is less optimal but much simpler to integrate.

### PATTERN RECOMMENDATIONS

- Keep the `mpsc` channel for the chunk data (pipeline thread to transcription thread). Only replace the cpal callback -> pipeline thread channel with a ring buffer.
- The `audio/mod.rs` pipeline thread becomes a polling loop instead of a blocking `recv()` loop. Adjust the `recv_timeout` to a sleep-and-poll pattern.
- Pre-allocate the consumer read buffer outside the loop: `let mut read_buf = vec![0.0f32; 4800];` (100ms at 48000Hz).

### AGENT DIRECTIVES

- This phase modifies `backend/src/audio/capture.rs` (ring buffer producer in callback) and `backend/src/audio/mod.rs` (ring buffer consumer in pipeline thread). Also `backend/Cargo.toml` (add `ringbuf`).
- Read both `capture.rs` and `audio/mod.rs` fully before editing. The data flow is: cpal callback -> (ringbuf) -> pipeline thread -> (mpsc) -> transcription thread.
- After editing, run `cargo check`, then `cargo clippy`. Test with `npm run tauri dev` and speak for 30 seconds.
- If Phase 1 has changed the audio pipeline significantly, re-read the modified files before starting this phase.

---

## Phase 18: Make model registry static

### GOTCHA ALERTS

1. **`std::sync::LazyLock` is stable since Rust 1.80.** If the project's Rust toolchain is older than 1.80, use `once_cell::sync::Lazy` instead (add `once_cell = "1"` to Cargo.toml). But since the project uses whisper-rs 0.15 and Tauri v2, the toolchain is likely recent enough for `LazyLock`.

2. **The return type changes from `Vec<WhisperModel>` to `&'static Vec<WhisperModel>`.** All callers that currently receive `Vec<WhisperModel>` (owned) will now receive a reference. This affects:
   - `commands/models.rs` line 33: `get_model_registry().iter().map(...)` -- `.iter()` works on both `Vec` and `&Vec`. No change needed.
   - `commands/models.rs` line 72: `let registry = get_model_registry();` then `registry.iter()...` -- works with `&Vec`. No change.
   - `model_manager/download.rs` line 10: `let registry = get_model_registry();` then `registry.iter()...` -- same. No change.
   - `model_manager/download.rs` line 91: same pattern. No change.
   - `model_manager/download.rs` line 105: same pattern. No change.
   All call sites iterate over the registry. Since `&Vec<T>` implements `IntoIterator` and has `.iter()`, no call site changes are needed.

3. **`WhisperModel` fields are all `String`.** The `LazyLock` initializer uses `.to_string()` on string literals, which allocates. This happens exactly once (on first access). After that, all access is zero-cost.

4. **The existing tests in `models.rs` (lines 63-74) call `get_model_registry()` and work with the result.** With the new signature returning `&'static Vec<WhisperModel>`, the tests should still work because they call `.iter()` on the result. Test: `let registry = get_model_registry(); assert!(!registry.is_empty());` -- `is_empty()` works on `&Vec`. `registry.iter().any(...)` -- works on `&Vec`. No test changes needed.

5. **Thread safety: `LazyLock` is `Sync`.** Multiple threads can call `get_model_registry()` concurrently. The first call initializes; subsequent calls return the cached reference. This is safe.

### ESCAPE HATCHES

- **If `cargo check` fails with "LazyLock not found":** Your Rust version is below 1.80. Use `once_cell::sync::Lazy` instead:
  ```rust
  use once_cell::sync::Lazy;
  static MODEL_REGISTRY: Lazy<Vec<WhisperModel>> = Lazy::new(|| { ... });
  ```
  Add `once_cell = "1"` to `Cargo.toml`.

- **If a caller needs an owned `Vec<WhisperModel>` (not a reference):** That caller must `.clone()` the result. But based on the current code, no caller needs ownership -- they all iterate.

- **If lifetime errors appear at call sites:** A call site may be trying to store the `&'static Vec` in a shorter-lived variable in a way the compiler dislikes. Since the reference is `'static`, it should satisfy any lifetime requirement. If a function signature expects `Vec<WhisperModel>` (owned), it must be changed to `&[WhisperModel]` or `&Vec<WhisperModel>`.

### PATTERN RECOMMENDATIONS

- Return `&'static [WhisperModel]` instead of `&'static Vec<WhisperModel>`. Slices are more idiomatic than references to Vec. Change: `pub fn get_model_registry() -> &'static [WhisperModel] { &MODEL_REGISTRY }`. This is a strictly more general type.
- Alternatively, if you want to minimize call-site changes, return `&'static Vec<WhisperModel>` as the plan suggests.

### AGENT DIRECTIVES

- Edit `backend/src/transcription/models.rs`. Replace the function body with a `LazyLock` static and change the return type.
- Run `cargo check` to validate. All call sites should work without changes due to auto-deref.
- Run `cargo test` from `backend/` to confirm the existing tests pass.
- This is the final phase. After this, proceed to the Post-Fix Validation checklist in PHASES-FIX.md.

---

## Cross-Phase Notes

### Build Command Reference
- Rust check: `cd /home/frosty/local-stt/backend && cargo check`
- Rust build: `cd /home/frosty/local-stt/backend && cargo build`
- Rust test: `cd /home/frosty/local-stt/backend && cargo test`
- Rust lint: `cd /home/frosty/local-stt/backend && cargo clippy`
- TypeScript build: `cd /home/frosty/local-stt && npm run build`
- Full dev run: `cd /home/frosty/local-stt && npm run tauri dev`

### File Location Quick Reference
All paths are relative to `/home/frosty/local-stt/`:

| Module | Key Files |
|--------|-----------|
| Audio capture | `backend/src/audio/capture.rs` |
| Audio pipeline | `backend/src/audio/mod.rs` |
| Audio buffer | `backend/src/audio/buffer.rs` |
| VAD | `backend/src/audio/vad.rs` |
| Dictation commands | `backend/src/commands/dictation.rs` |
| Model commands | `backend/src/commands/models.rs` |
| Config commands | `backend/src/commands/config.rs` |
| System commands | `backend/src/commands/system.rs` |
| Transcription engine | `backend/src/transcription/engine.rs` |
| Model registry | `backend/src/transcription/models.rs` |
| Config/settings | `backend/src/config/settings.rs` |
| Model download | `backend/src/model_manager/download.rs` |
| Output routing | `backend/src/output/mod.rs` |
| Keyboard output | `backend/src/output/keyboard.rs` |
| Clipboard output | `backend/src/output/clipboard.rs` |
| App entry | `backend/src/lib.rs` |
| Main | `backend/src/main.rs` |
| Cargo config | `backend/Cargo.toml` |
| Tauri config | `backend/tauri.conf.json` |
| Tauri IPC types | `frontend/src/lib/tauri.ts` |
| Main window | `frontend/src/pages/main-window.tsx` |
| App root | `frontend/src/App.tsx` |
| Dictation hook | `frontend/src/hooks/use-dictation.ts` |
| Models hook | `frontend/src/hooks/use-models.ts` |
| Transcription hook | `frontend/src/hooks/use-transcription.ts` |
| Config hook | `frontend/src/hooks/use-config.ts` |
| Setup wizard | `frontend/src/components/setup-wizard/index.tsx` |
| Step GPU | `frontend/src/components/setup-wizard/step-gpu.tsx` |
| Step Models | `frontend/src/components/setup-wizard/step-models.tsx` |
| Step Download | `frontend/src/components/setup-wizard/step-download.tsx` |
| Step Complete | `frontend/src/components/setup-wizard/step-complete.tsx` |
| Settings panel | `frontend/src/components/settings-panel.tsx` |
| Status indicator | `frontend/src/components/status-indicator.tsx` |
| Model selector | `frontend/src/components/model-selector.tsx` |
| Transcript display | `frontend/src/components/transcript-display.tsx` |
| HTML entry | `index.html` |
| NPM config | `package.json` |

### Dependency Graph (data flow)
```
cpal (audio device)
  -> AudioCapture::start() callback [capture.rs]
    -> mpsc::Sender<Vec<f32>>
      -> AudioPipeline thread [audio/mod.rs]
        -> AudioRingBuffer [buffer.rs]
        -> VoiceActivityDetector [vad.rs]
        -> mpsc::Sender<Vec<f32>> (speech chunks)
          -> Transcription thread [dictation.rs]
            -> TranscriptionEngine::transcribe() [engine.rs]
              -> whisper-rs (WhisperContext, FullParams)
            -> output::output_text() [output/mod.rs]
              -> keyboard::type_text() [keyboard.rs] (enigo)
              -> clipboard::copy_to_clipboard() [clipboard.rs] (arboard)
            -> app.emit("transcription-update", ...)
              -> Frontend event listener
                -> useTranscription hook -> TranscriptDisplay component
```

### Phase Execution Order and Dependencies
- Phases 1-4 are sequential (each depends on the previous being verified).
- Phases 5-6 can be done in parallel with each other but after Phase 4.
- Phases 7-12 are independent of each other and can be done in any order.
- Phases 13-16 are independent of each other but should be done after Phases 7-12 (to avoid editing files that will be deleted).
- Phase 17 depends on Phase 1 (the audio pipeline structure).
- Phase 18 is independent of all other phases.
