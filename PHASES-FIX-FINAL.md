# WhisperType -- Definitive Audit Fix Plan (FINAL)

Generated: 2026-02-13
Incorporates: PHASES-FIX.md (original), REVIEW-CRITIQUE.md (critic), REVIEW-TACTICS.md (tactical advisor)
Rust toolchain: rustc 1.93.0 (confirmed via `rustc --version`)
Edition: 2021 (per `backend/Cargo.toml` line 6)

---

## Summary Table

| Phase | Title | Priority | Category | Files | Depends On | Complexity |
|-------|-------|----------|----------|-------|------------|------------|
| 1 | Fix audio capture: device-native format + pipeline resampling | CRITICAL | Bug Fix | `audio/capture.rs`, `audio/mod.rs` | None | Complex |
| 2 | Surface output_text errors to frontend | HIGH | Bug Fix | `commands/dictation.rs`, `lib/tauri.ts`, `hooks/use-dictation.ts`, `pages/main-window.tsx` | None | Moderate |
| 3 | Surface transcription errors to frontend | HIGH | Bug Fix | `commands/dictation.rs`, `lib/tauri.ts`, `hooks/use-dictation.ts` | Phase 2 | Simple |
| 4 | Track and join transcription thread on toggle | HIGH | Bug Fix | `commands/dictation.rs`, `lib.rs` | None | Moderate |
| 5 | Fix model auto-load race condition after setup | MEDIUM | Bug Fix | `hooks/use-models.ts`, `pages/main-window.tsx` | None | Simple |
| 6 | Set Content Security Policy | MEDIUM | Bug Fix | `tauri.conf.json` | None | Trivial |
| 7 | Fix index.html title | LOW | Code Smell | `index.html` | None | Trivial |
| 8 | Fix StepComplete placeholder text | LOW | Code Smell | `step-complete.tsx` | None | Trivial |
| 9 | Add SAFETY comment to unsafe set_var in main.rs | LOW | Code Smell | `main.rs` | None | Trivial |
| 10 | Fix unnecessary type cast in StepGpu | LOW | Code Smell | `step-gpu.tsx` | None | Trivial |
| 11 | Normalize Config import paths across all files | LOW | Code Smell | `commands/dictation.rs`, `model_manager/download.rs`, `output/mod.rs` | None | Trivial |
| 12 | Remove unused `_toggle` and `isListening` | LOW | Code Smell | `pages/main-window.tsx`, `hooks/use-dictation.ts` | Phase 2, Phase 3 | Simple |
| 13 | Remove dead backend stubs: storage.rs, hotkey/ | LOW | Dead Code | `model_manager/storage.rs`, `model_manager/mod.rs`, `hotkey/*`, `lib.rs` | None | Simple |
| 14 | Remove unused backend methods and hound dependency | LOW | Dead Code | `audio/capture.rs`, `audio/vad.rs`, `audio/buffer.rs`, `Cargo.toml` | Phase 1 | Simple |
| 15 | Remove dead frontend page and unused state | LOW | Dead Code | `pages/setup.tsx`, `hooks/use-models.ts` | Phase 5 | Trivial |
| 16 | Remove unused shadcn UI components and deps | LOW | Dead Code | `components/ui/*`, `package.json` | None | Simple |
| 17 | Eliminate heap allocation in audio callback | MEDIUM | Optimization | `audio/capture.rs`, `audio/mod.rs`, `Cargo.toml` | Phase 1 | Moderate |
| 18 | Make model registry static | LOW | Optimization | `transcription/models.rs` | None | Simple |
| E2E | End-to-end acceptance test | CRITICAL | Validation | All | All phases | N/A |

All file paths below are relative to `/home/frosty/local-stt/` unless otherwise noted.

---

## Deferred / Known Limitations

| Issue | Reason |
|-------|--------|
| OPT-2: Whisper ctx Mutex held during entire inference | Architectural change (use separate state per inference) is out of scope. The Mutex contention means `load_model`/`unload_model`/`is_loaded` block during transcription. Acceptable for v0.1. |
| VAD uses simple RMS energy | Functional. WebRTC VAD or Silero VAD would be better but is a feature enhancement, not a bug fix. |
| React memoization | No measurable performance issue at current UI complexity. |

---

## Section A: Critical Pipeline Fixes (Phases 1-4)

---

### Phase 1: Fix audio capture: device-native format + pipeline resampling
**Priority:** CRITICAL
**Category:** Bug Fix
**Files:** `backend/src/audio/capture.rs`, `backend/src/audio/mod.rs`
**Depends On:** None
**Estimated Complexity:** Complex

**Context:**
In `backend/src/audio/capture.rs`, `start()` hardcodes a `StreamConfig` at lines 47-51 with `sample_rate: SampleRate(16000)` and `channels: 1`. Most microphones only support 44100Hz or 48000Hz stereo. `build_input_stream` fails on real hardware. The app is non-functional on most systems.

**DECISION (resolving ambiguity from original plan):** Resampling happens in the pipeline thread in `backend/src/audio/mod.rs`, between `audio_rx.recv()` and `buffer.write()`. The audio callback in `capture.rs` sends raw device-format samples. The `AudioRingBuffer` at line 54 of `audio/mod.rs` stays at 16000 because it receives already-resampled data. This approach:
- Keeps the cpal callback lightweight (no resampling in the hot path)
- Contains all format conversion in one location (pipeline thread)
- Avoids conflicting with Phase 17 (ring buffer replaces the cpal->pipeline channel, not the pipeline->buffer path)
- Means VAD frame sizes in `vad.rs` (480 samples = 30ms at 16kHz) remain correct

**Instructions:**
1. In `backend/src/audio/capture.rs`, add `use cpal::SupportedStreamConfig;` to the imports at line 2.
2. In `AudioCapture` struct (line 5-8), change the `sample_rate` field to store device-reported values and add a `channels` field:
   ```rust
   pub struct AudioCapture {
       stream: Option<Stream>,
       pub device_sample_rate: u32,
       pub device_channels: u16,
   }
   ```
3. In `AudioCapture::new()` (line 11-16), initialize with defaults:
   ```rust
   pub fn new() -> Self {
       Self {
           stream: None,
           device_sample_rate: 48000,
           device_channels: 1,
       }
   }
   ```
4. In `start()` (line 29-70), replace the hardcoded `StreamConfig` block (lines 47-51) with a query to the device's default config:
   ```rust
   let supported_config = device
       .default_input_config()
       .map_err(|e| format!("Failed to get default input config: {}", e))?;

   let sample_format = supported_config.sample_format();
   let config: StreamConfig = supported_config.into();
   self.device_sample_rate = config.sample_rate.0;
   self.device_channels = config.channels;
   ```
5. The `build_input_stream` call at lines 53-62 stays as `build_input_stream::<f32>`. On PipeWire (this system), the default sample format is F32. If `sample_format` is not F32, log a warning with `eprintln!` but attempt F32 anyway (cpal/PipeWire handles conversion). Do NOT add sample format dispatch unless needed (escape hatch below).
6. In `backend/src/audio/mod.rs`, add a helper function at the top of the file (after imports):
   ```rust
   /// Convert interleaved multi-channel audio to mono by averaging channels.
   fn to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
       if channels == 1 {
           return samples.to_vec();
       }
       let ch = channels as usize;
       samples
           .chunks_exact(ch)
           .map(|frame| frame.iter().sum::<f32>() / ch as f32)
           .collect()
   }

   /// Resample audio from src_rate to dst_rate using linear interpolation.
   fn resample(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
       if src_rate == dst_rate {
           return input.to_vec();
       }
       let ratio = src_rate as f64 / dst_rate as f64;
       let output_len = (input.len() as f64 / ratio) as usize;
       let mut output = Vec::with_capacity(output_len);
       for i in 0..output_len {
           let src_pos = i as f64 * ratio;
           let idx = src_pos as usize;
           let frac = (src_pos - idx as f64) as f32;
           let a = input[idx];
           let b = input[(idx + 1).min(input.len() - 1)];
           output.push(a * (1.0 - frac) + b * frac);
       }
       output
   }
   ```
7. In `AudioPipeline::start()` at line 42-76, after `capture.start()` succeeds (line 47-50), read the device's actual rate and channels:
   ```rust
   let device_rate = capture.device_sample_rate;
   let device_channels = capture.device_channels;
   ```
8. In the pipeline loop (line 57-74), after `Ok(samples)` at line 59, add resampling before `buffer.write()`:
   ```rust
   Ok(samples) => {
       let mono = to_mono(&samples, device_channels);
       let resampled = resample(&mono, device_rate, 16000);
       buffer.write(&resampled);
       // ... rest of chunk extraction and VAD unchanged
   }
   ```
9. The `AudioRingBuffer::new(16000, ...)` call at line 54 stays at 16000 because the buffer now receives 16kHz mono data.

**Gotcha Alerts:**
- `device.default_input_config()` returns `SupportedStreamConfig`, not `StreamConfig`. Call `.into()` to convert it. The `SupportedStreamConfig` also has `sample_format()` -- check it but do not gate on it.
- PipeWire default rate is almost always 48000Hz. The `AudioRingBuffer` must receive 16kHz data (after resampling), so keep the buffer at 16000.
- Stereo interleaved data: `[L0, R0, L1, R1, ...]`. After mono conversion, sample count = `data.len() / channels`. After resampling 48kHz->16kHz (ratio 3:1), output length = `mono.len() / 3`.
- `build_input_stream::<f32>` may fail if the device does not support F32. On PipeWire this is rare. See escape hatch below.
- The `resample()` function allocates a new `Vec` per call. This is fine for the pipeline thread (not the audio callback). Phase 17 will address the callback's allocation separately.

**If Stuck:**
- If `build_input_stream` fails with "stream configuration not supported": Run `pactl list sources short` to check PipeWire sources. Print the `SupportedStreamConfig` to stderr: `eprintln!("device config: {:?}", supported_config);`. Verify the sample format matches what you pass to `build_input_stream`.
- If the audio callback never fires after changing config: Ensure the `Stream` object is stored in `self.stream` and not dropped. Verify with `pw-top` that a PipeWire stream appears.
- If resampled audio sounds like chipmunks or slowed down: The resample ratio is inverted. Downsampling from 48kHz to 16kHz means ratio = 3.0 (every 3 source samples -> 1 output sample).
- If transcription returns empty segments: Log the first 20 sample values of the resampled chunk before `engine.transcribe()`. If all 0.0, the mono conversion is wrong. If non-zero but empty output, the audio chunk may be too short (whisper needs ~1 second minimum).
- If `cargo check` fails with `SupportedStreamConfig` not found: Import it with `use cpal::SupportedStreamConfig;`.

**Agent Directives:**
- Read `backend/src/audio/mod.rs` lines 42-76 carefully BEFORE editing. The pipeline thread is spawned at line 42. Resampling goes inside this thread, between `audio_rx.recv()` and `buffer.write()`.
- After editing, run `cargo check` from `/home/frosty/local-stt/backend/` immediately. Do NOT batch with other changes.
- Test this phase in isolation before proceeding.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `cargo run` or `npm run tauri dev` -- click dictation toggle or press Ctrl+Shift+Space -- no "stream configuration not supported" error in terminal
- [ ] Speak for 5 seconds and confirm transcription output appears in the terminal (even if garbled)
- [ ] Print the device's reported sample rate and channels to stderr on capture start to confirm detection works

**Approval Gate:** Audio capture must successfully start and audio data must reach the transcription engine on real hardware before proceeding.

---

### Phase 2: Surface output_text errors to frontend
**Priority:** HIGH
**Category:** Bug Fix
**Files:** `backend/src/commands/dictation.rs` (line 49), `frontend/src/lib/tauri.ts`, `frontend/src/hooks/use-dictation.ts`, `frontend/src/pages/main-window.tsx`
**Depends On:** None
**Estimated Complexity:** Moderate

**Context:**
In `backend/src/commands/dictation.rs` at line 49, `output::output_text(&segment.text, &output_mode).ok()` silently discards errors. If `enigo::text()` or `arboard::set_text()` fails (common on Wayland without XWayland), the user sees transcription text in the UI but nothing is typed into their application, with no error indication.

**Instructions:**
1. In `backend/src/commands/dictation.rs`, at line 49, replace:
   ```rust
   output::output_text(&segment.text, &output_mode).ok();
   ```
   with:
   ```rust
   if let Err(e) = output::output_text(&segment.text, &output_mode) {
       eprintln!("Output error: {}", e);
       app_clone.emit("output-error", format!("Failed to output text: {}", e)).ok();
   }
   ```
2. In `frontend/src/lib/tauri.ts`, add to the `events` object (after line 69):
   ```typescript
   onOutputError: (handler: (message: string) => void): Promise<UnlistenFn> =>
     listen<string>("output-error", (event) => handler(event.payload)),
   ```
3. In `frontend/src/hooks/use-dictation.ts`, add state and listener:
   - Add `const [error, setError] = useState<string | null>(null);` after line 8.
   - Add a `useRef` for the clear timeout: `const errorTimeoutRef = useRef<ReturnType<typeof setTimeout>>();`
   - Add a second `useEffect` block:
     ```typescript
     useEffect(() => {
       const unlisten = events.onOutputError((msg) => {
         if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
         setError(msg);
         errorTimeoutRef.current = setTimeout(() => setError(null), 5000);
       });
       return () => {
         unlisten.then((fn) => fn());
         if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
       };
     }, []);
     ```
   - Add `error` to the return object: `return { isListening, status, toggle, error };`
   - Add `useRef` to the React imports at line 1.
4. In `frontend/src/pages/main-window.tsx`, destructure `error` from `useDictation()` at line 13:
   ```typescript
   const { status, toggle: _toggle, error } = useDictation();
   ```
   Add an error banner below the `StatusIndicator` (after line 80):
   ```tsx
   {error && (
     <div className="mt-2 text-xs text-red-400 bg-red-400/10 border border-red-400/20 rounded px-3 py-2">
       {error}
     </div>
   )}
   ```

**Gotcha Alerts:**
- Tauri v2 uses `app.emit()`, not `window.emit()`. The existing code already uses `app_clone.emit(...)` (line 50-58). Follow the same pattern.
- Event name must be kebab-case: `"output-error"` (with hyphens). Existing events use kebab-case: `"dictation-status"`, `"transcription-update"`.
- The `Emitter` trait is already imported at `dictation.rs` line 2. No new import needed.
- Frontend event listeners return `Promise<UnlistenFn>`. Follow the existing `.then(fn => fn())` cleanup pattern in `use-dictation.ts` line 16-18.
- Use functional state updates (`setError(msg)`) to avoid stale closure issues in the event handler.

**If Stuck:**
- If the error event never arrives in the frontend: Add `eprintln!("Emitting output-error: {}", e);` before the emit call. If it appears in terminal but not frontend, compare the event name strings character by character.
- If TypeScript complains about the listener type: Use `listen<string>("output-error", (event) => handler(event.payload))`.

**Agent Directives:**
- Edit four files: `backend/src/commands/dictation.rs`, `frontend/src/lib/tauri.ts`, `frontend/src/hooks/use-dictation.ts`, `frontend/src/pages/main-window.tsx`.
- Run `cargo check` from `backend/` after the Rust change, then `npm run build` from root after the TypeScript changes. Do not batch these checks.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] To test: temporarily make `output_text` always return `Err("test error".into())` and confirm the error appears in the frontend UI
- [ ] Revert the test failure and confirm normal operation still works

**Approval Gate:** Output errors must be visibly surfaced in the frontend before proceeding to Phase 3.

---

### Phase 3: Surface transcription errors to frontend
**Priority:** HIGH
**Category:** Bug Fix
**Files:** `backend/src/commands/dictation.rs` (line 61-63), `frontend/src/lib/tauri.ts`, `frontend/src/hooks/use-dictation.ts`
**Depends On:** Phase 2 (reuses the error display mechanism)
**Estimated Complexity:** Simple

**Context:**
In `backend/src/commands/dictation.rs` at line 62, `engine.transcribe()` failures only print to stderr. The frontend has no way to know transcription failed. This can happen with malformed audio, GPU OOM, or model corruption.

**Instructions:**
1. In `backend/src/commands/dictation.rs`, inside the `Err(e)` branch at lines 61-63, add an event emission after the `eprintln!`:
   ```rust
   Err(e) => {
       eprintln!("Transcription error: {}", e);
       app_clone.emit("transcription-error", format!("Transcription failed: {}", e)).ok();
   }
   ```
2. In `frontend/src/lib/tauri.ts`, add to the `events` object:
   ```typescript
   onTranscriptionError: (handler: (message: string) => void): Promise<UnlistenFn> =>
     listen<string>("transcription-error", (event) => handler(event.payload)),
   ```
3. In `frontend/src/hooks/use-dictation.ts`, add a listener for `"transcription-error"` that reuses the `error` state from Phase 2. Add it inside the same `useEffect` block (or add a third one):
   ```typescript
   useEffect(() => {
     const unlisten = events.onTranscriptionError((msg) => {
       if (errorTimeoutRef.current) clearTimeout(errorTimeoutRef.current);
       setError(msg);
       errorTimeoutRef.current = setTimeout(() => setError(null), 5000);
     });
     return () => {
       unlisten.then((fn) => fn());
     };
   }, []);
   ```
4. No change needed in `main-window.tsx` -- the error banner from Phase 2 already displays `error`.

**Gotcha Alerts:**
- Keep the `eprintln!` alongside the emit. Stderr logging is useful for debugging.
- Transcription errors can be rapid-fire if the model is corrupted. The 5-second auto-clear with reset-on-new-error handles this (each new error resets the timeout).
- GPU OOM errors from whisper-rs are not recoverable without unloading and loading a smaller model.

**If Stuck:**
- If errors spam the UI: Add a rate limiter in the Rust code. Use a `std::time::Instant` stored before the `while let` loop. Only emit if `last_error_emit.elapsed() > Duration::from_secs(2)`.
- If the error text is unhelpful: Log `audio_data.len()` before `engine.transcribe()` to distinguish bad audio from bad model.

**Agent Directives:**
- Edit three files: `backend/src/commands/dictation.rs`, `frontend/src/lib/tauri.ts`, `frontend/src/hooks/use-dictation.ts`.
- Run `cargo check` then `npm run build`. Quick phase.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] To test: temporarily make `engine.transcribe()` return an error, confirm the error message appears in the frontend UI
- [ ] Revert and confirm normal transcription still works

**Approval Gate:** Transcription errors must be visibly surfaced in the frontend before proceeding to Phase 4.

---

### Phase 4: Track and join transcription thread on toggle
**Priority:** HIGH
**Category:** Bug Fix
**Files:** `backend/src/commands/dictation.rs` (lines 9-13 struct, line 17-20 stop branch, line 44 spawn, line 87-93 stop_dictation), `backend/src/lib.rs` (line 19-23 construction)
**Depends On:** None
**Estimated Complexity:** Moderate

**Context:**
In `backend/src/commands/dictation.rs`, `std::thread::spawn` at line 44 creates a transcription thread but never stores the `JoinHandle`. Rapid toggling creates multiple threads fighting over the `ctx` Mutex. The `stop_dictation` command at lines 87-93 is a separate code path that also does not join the thread.

**Instructions:**
1. In `backend/src/commands/dictation.rs`, add a field to the `AppState` struct at line 9-13:
   ```rust
   pub struct AppState {
       pub engine: Arc<TranscriptionEngine>,
       pub pipeline: AudioPipeline,
       pub config: Mutex<Config>,
       pub transcription_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
   }
   ```
2. In `backend/src/lib.rs`, at line 19-23, add the new field to the `AppState` construction:
   ```rust
   let app_state = AppState {
       engine: Arc::new(TranscriptionEngine::new()),
       pipeline: audio::AudioPipeline::new(),
       config: Mutex::new(config),
       transcription_thread: Mutex::new(None),
   };
   ```
3. Create a helper function in `dictation.rs` (before `toggle_dictation_inner`):
   ```rust
   fn join_transcription_thread(state: &AppState) {
       let handle = state.transcription_thread.lock().unwrap().take();
       if let Some(h) = handle {
           h.join().ok();
       }
   }
   ```
   **CRITICAL:** Use `.take()` to extract the handle from the Mutex, then drop the lock, THEN join. Do NOT hold the Mutex lock during `join()`.
4. In `toggle_dictation_inner()`, in the stop-dictation branch (lines 17-20), after `state.pipeline.stop()` at line 18, add:
   ```rust
   join_transcription_thread(state);
   ```
5. In `toggle_dictation_inner()`, in the start-dictation branch, before spawning the new thread (before line 44), add:
   ```rust
   join_transcription_thread(state);
   ```
6. After `std::thread::spawn(...)` at line 44, store the handle:
   ```rust
   let handle = std::thread::spawn(move || {
       // ... existing thread body ...
   });
   *state.transcription_thread.lock().unwrap() = Some(handle);
   ```
7. In `stop_dictation()` at lines 87-93, after `state.pipeline.stop()` and `app.emit(...)`, add:
   ```rust
   join_transcription_thread(&state);
   ```

**Gotcha Alerts:**
- `JoinHandle<()>` is `Send + Sync`. Wrapping in `Mutex<Option<JoinHandle<()>>>` is safe.
- When `pipeline.stop()` is called, it sets `is_running` to `false`. The pipeline thread notices within ~100ms (its `recv_timeout`), then exits, dropping `chunk_tx`. This causes `receiver.recv()` in the transcription thread to return `Err(RecvError)`, exiting the `while let Ok(chunk)` loop. So `join()` blocks for at most ~100ms plus any in-flight transcription. This is acceptable.
- The transcription thread does NOT lock `AppState.transcription_thread`, so there is no deadlock cycle. But always use `.take()` before joining to be safe.
- `stop_dictation()` at line 87-93 is a separate IPC command that does NOT go through `toggle_dictation_inner()`. It MUST also join the thread. Without this fix, `stop_dictation` followed by `start_dictation` can still race.

**If Stuck:**
- If `handle.join()` hangs: The transcription thread is stuck in `engine.transcribe()` (possible on corrupted audio or GPU hang). Add a shared `AtomicBool` that the transcription thread sets before exiting, then poll it with a 5-second timeout before calling `join()`. Log a warning if the timeout expires.
- If `cargo check` fails with "field `transcription_thread` not found": You forgot to add the field to either the struct definition (dictation.rs) or the construction site (lib.rs).
- If rapid toggling causes a deadlock: Print thread IDs and Mutex acquisition attempts. The cause is likely holding `transcription_thread` Mutex while joining. Solution: always use `.take()` first.

**Agent Directives:**
- Edit two files: `backend/src/commands/dictation.rs` (struct + helper + toggle logic + stop logic) and `backend/src/lib.rs` (AppState construction).
- Read the entire `dictation.rs` file before editing. The struct is at line 9, the toggle logic at line 16, stop_dictation at line 87. All three locations need changes.
- After editing, run `cargo check`. Then test rapid toggling with `npm run tauri dev`.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `npm run tauri dev`, rapidly toggle dictation on/off 10 times via Ctrl+Shift+Space -- no panics or thread errors in terminal
- [ ] Dictation starts and stops cleanly after rapid toggling
- [ ] Add temporary `println!("transcription thread started/ended")` to verify only one thread is active at a time
- [ ] Call `stop_dictation` via the frontend (if exposed) and verify no thread leak

**Approval Gate:** Rapid toggle cycles must produce no thread leaks or panics before proceeding.

---

## Section B: Frontend & Config Fixes (Phases 5-6)

---

### Phase 5: Fix model auto-load race condition after setup
**Priority:** MEDIUM
**Category:** Bug Fix
**Files:** `frontend/src/hooks/use-models.ts` (lines 4-18, 66), `frontend/src/pages/main-window.tsx` (lines 15, 18-28)
**Depends On:** None
**Estimated Complexity:** Simple

**Context:**
In `frontend/src/pages/main-window.tsx` at lines 18-28, the `useEffect` that auto-loads the default model runs on mount when `models` is still `[]` (empty initial state). The effect has `models` in its dependency array so it re-runs when models load, but timing depends on React's scheduler and StrictMode double-firing may mask the bug in dev.

**Instructions:**
1. In `frontend/src/hooks/use-models.ts`, add a `loading` state at line 7 (after the existing state declarations):
   ```typescript
   const [loading, setLoading] = useState(true);
   ```
2. In the `refresh` function (line 9-18), set `loading` to `false` after completion. Modify the try/catch:
   ```typescript
   const refresh = useCallback(async () => {
     try {
       const modelList = await commands.listModels();
       setModels(modelList);
       const active = await commands.getActiveModel();
       setActiveModel(active);
     } catch (err) {
       console.error("Failed to fetch models:", err);
     } finally {
       setLoading(false);
     }
   }, []);
   ```
3. Add `loading` to the return object at line 66:
   ```typescript
   return { models, activeModel, loadModel, downloadModel, deleteModel, downloadProgress, refresh, loading };
   ```
4. In `frontend/src/pages/main-window.tsx`, destructure `loading` from `useModels()` at line 15:
   ```typescript
   const { models, activeModel, loadModel, loading } = useModels();
   ```
5. Update the `useEffect` at line 18-28 to guard on `loading`:
   ```typescript
   useEffect(() => {
     if (loading || !config || activeModel) return;
     const defaultModel = config.default_model;
     const isDownloaded = models.find(
       (m) => m.id === defaultModel && m.downloaded
     );
     if (isDownloaded) {
       loadModel(defaultModel);
     }
   }, [loading, config, models, activeModel, loadModel]);
   ```

**Gotcha Alerts:**
- Set `loading = false` in the `finally` block so it triggers even if `refresh()` fails.
- `loadModel`'s `useCallback` has an empty dependency array (line 39-46 of `use-models.ts`). Adding `loading` to the return value does NOT affect `loadModel`'s deps.
- In React StrictMode (enabled in `main.tsx` line 7), effects fire twice in dev. `loadModel` is idempotent, so double-firing is harmless but may cause a brief "Loading..." flash.

**If Stuck:**
- If the model still does not auto-load: Add `console.log` inside the effect logging `loading`, `config`, `models.length`, `activeModel`. Check which guard is blocking.
- If TypeScript complains about `loading`: Ensure it is added to both the return object of `useModels` and the destructure in `main-window.tsx`.

**Agent Directives:**
- Edit two files: `frontend/src/hooks/use-models.ts` and `frontend/src/pages/main-window.tsx`.
- Run `npm run build` after both edits. Quick phase.

**Verification:**
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] `npm run tauri dev`, complete setup wizard, confirm default model auto-loads (status shows "Model loaded", not "No model loaded")
- [ ] Close and reopen the app, confirm model auto-loads on startup

**Approval Gate:** Model auto-load must work reliably after both first-time setup and app restart.

---

### Phase 6: Set Content Security Policy
**Priority:** MEDIUM
**Category:** Bug Fix
**Files:** `backend/tauri.conf.json` (line 27)
**Depends On:** None
**Estimated Complexity:** Trivial

**Context:**
In `backend/tauri.conf.json` at line 27, `"csp": null` disables the CSP entirely. The webview has unrestricted access to load scripts, styles, and network requests from any origin.

**Instructions:**
1. In `backend/tauri.conf.json`, at line 27, replace `"csp": null` with:
   ```json
   "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' asset: https://asset.localhost; connect-src ipc: http://ipc.localhost https://huggingface.co https://*.huggingface.co"
   ```

**Notes on CSP choices:**
- `'unsafe-inline'` for `style-src`: Required. Tailwind CSS v4 injects styles. Without it, the UI is completely unstyled.
- `connect-src ipc: http://ipc.localhost`: Required for Tauri v2 IPC. Without these, all `invoke()` calls fail silently.
- `connect-src https://huggingface.co https://*.huggingface.co`: Defensive inclusion. Model downloads actually go through Rust backend via `reqwest` (not the webview), so CSP `connect-src` may not strictly need HuggingFace URLs. Included for safety if future frontend code makes direct API calls.
- `navigator.clipboard.writeText()` (used in `main-window.tsx` line 86): The Clipboard API is controlled by the Permissions API, not CSP. This CSP does not block it.

**Gotcha Alerts:**
- A wrong CSP silently breaks IPC, model downloads, or styling. The only way to detect issues is opening DevTools (F12) and checking the Console tab for "Refused to load" messages.
- If the app loads but the UI is completely unstyled: `style-src` is missing `'unsafe-inline'`.
- If `invoke()` calls fail silently: `connect-src` is missing `ipc:` or `http://ipc.localhost`.

**If Stuck:**
- Start with a more permissive CSP and tighten incrementally. Temporarily add `'unsafe-eval'` to `script-src` and `*` to `connect-src`, verify the app works, then remove permissive directives one by one.
- Open DevTools (F12), go to Console tab. Any CSP violation is logged as "Refused to load..." with the exact directive that needs adjusting.

**Agent Directives:**
- Edit one file: `backend/tauri.conf.json`. This is a config-only change.
- Test by running `npm run tauri dev`. Open DevTools (F12) and check for CSP violations.
- Walk through the ENTIRE app flow: setup wizard GPU detection, model download, main window, dictation toggle.

**Verification:**
- [ ] `npm run tauri dev` -- app loads without CSP errors in webview console
- [ ] Setup wizard works: GPU detection, model download with progress
- [ ] Main window works: dictation toggle, transcription display
- [ ] No "Refused to load" messages in the Console tab
- [ ] Copy button (which uses `navigator.clipboard.writeText()`) still works

**Approval Gate:** The app must run cleanly with the new CSP and all features must work.

---

## Section C: Code Smell Cleanup (Phases 7-12)

---

### Phase 7: Fix index.html title
**Priority:** LOW
**Category:** Code Smell
**Files:** `/home/frosty/local-stt/index.html` (line 7)
**Depends On:** None
**Estimated Complexity:** Trivial

**Context:**
In `index.html` at line 7, the page title is `"Tauri + React + Typescript"` -- the default Vite template title.

**Instructions:**
1. In `/home/frosty/local-stt/index.html`, at line 7, change:
   ```html
   <title>Tauri + React + Typescript</title>
   ```
   to:
   ```html
   <title>WhisperType</title>
   ```

**Gotcha Alerts:**
- The file is at the project root (`/home/frosty/local-stt/index.html`), NOT in `frontend/` or `backend/`.
- Tauri v2 uses the `title` field in `tauri.conf.json` (line 15: `"title": "WhisperType"`) for the native window title. The HTML `<title>` tag affects the taskbar tooltip and document title.

**If Stuck:**
- If the window title still shows the old value: Clear the Vite cache (`rm -rf node_modules/.vite`) and restart `npm run tauri dev`.

**Agent Directives:**
- Edit one file, one line. Done.

**Verification:**
- [ ] `npm run tauri dev` -- window title / taskbar shows "WhisperType"
- [ ] No build errors

**Approval Gate:** Window title shows "WhisperType".

---

### Phase 8: Fix StepComplete placeholder text
**Priority:** LOW
**Category:** Code Smell
**Files:** `frontend/src/components/setup-wizard/step-complete.tsx` (line 8)
**Depends On:** None
**Estimated Complexity:** Trivial

**Context:**
In `step-complete.tsx` at line 8, `<div className="text-4xl">Microphone</div>` is a leftover placeholder. It should be a visual indicator, not the word "Microphone".

**Instructions:**
1. In `frontend/src/components/setup-wizard/step-complete.tsx`, at line 8, replace:
   ```tsx
   <div className="text-4xl">Microphone</div>
   ```
   with:
   ```tsx
   <div className="text-6xl">{'\u2714'}</div>
   ```
   This renders a heavy checkmark character (Unicode U+2714) which is universally supported.

**Gotcha Alerts:**
- Use the Unicode escape `{'\u2714'}` in JSX, not an HTML entity. HTML entities like `&#10003;` work in JSX but the Unicode escape is cleaner.
- Do NOT add a dependency (like an icon library) for a single checkmark.

**If Stuck:**
- If the checkmark does not render on your system: Try `{'\u2713'}` (lighter checkmark) or use an inline SVG checkmark.

**Agent Directives:**
- Edit one file, one line. Run `npm run build` to verify.

**Verification:**
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] `npm run tauri dev`, navigate through setup wizard to final step, confirm a checkmark icon is shown instead of the word "Microphone"

**Approval Gate:** StepComplete shows a proper icon.

---

### Phase 9: Add SAFETY comment to unsafe set_var in main.rs
**Priority:** LOW
**Category:** Code Smell
**Files:** `backend/src/main.rs` (lines 7-13)
**Depends On:** None
**Estimated Complexity:** Trivial

**Context:**
The original plan proposed removing the `unsafe` block around `std::env::set_var` calls, claiming it was unnecessary in Rust edition 2021. **This is WRONG.** `std::env::set_var` was made `unsafe` starting in Rust 1.83 (stable December 2024), regardless of edition. This system runs rustc 1.93.0. Removing `unsafe` would cause a compilation error.

The `unsafe` block is CORRECT and REQUIRED. The fix is to add a `// SAFETY:` comment explaining why the unsafe usage is sound.

**Instructions:**
1. In `backend/src/main.rs`, at line 8, add a safety comment before the `unsafe` block:
   ```rust
   #[cfg(target_os = "linux")]
   // SAFETY: set_var is unsafe in Rust 1.83+ because it is not thread-safe.
   // We call it in main() before any threads are spawned and before Tauri
   // initialization, so there are no concurrent readers of the environment.
   unsafe {
       std::env::set_var("GDK_BACKEND", "x11");
       // Disable WebKitGTK DMA-BUF renderer -- GBM buffer creation fails on newer NVIDIA GPUs,
       // causing the webview to render as a black screen.
       std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
   }
   ```

**Gotcha Alerts:**
- Do NOT remove the `unsafe` block. On rustc 1.93.0, `set_var` requires `unsafe`.
- The `// SAFETY:` comment is a Rust convention for documenting why unsafe code is sound.

**If Stuck:**
- N/A. This is a comment-only change.

**Agent Directives:**
- Edit one file. Add the comment. Run `cargo check` to verify no regressions.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors or warnings
- [ ] `npm run tauri dev` -- app launches normally on Linux

**Approval Gate:** `cargo check` passes cleanly.

---

### Phase 10: Fix unnecessary type cast in StepGpu
**Priority:** LOW
**Category:** Code Smell
**Files:** `frontend/src/components/setup-wizard/step-gpu.tsx` (line 15)
**Depends On:** None
**Estimated Complexity:** Trivial

**Context:**
In `step-gpu.tsx` at line 15, `info as unknown as GpuInfo` is an unnecessary double-cast. `commands.getGpuInfo()` returns `Promise<GpuInfo>` via `invoke<GpuInfo>()`, so the cast is redundant.

**Instructions:**
1. In `frontend/src/components/setup-wizard/step-gpu.tsx`, at line 15, replace:
   ```typescript
   .then((info) => setGpu(info as unknown as GpuInfo))
   ```
   with:
   ```typescript
   .then((info) => setGpu(info))
   ```

**Gotcha Alerts:**
- The Rust side returns `serde_json::Value` (`commands/system.rs` line 9), but the TypeScript side calls `invoke<GpuInfo>("get_gpu_info")` which asserts the type. The JSON keys (`name`, `vram_total_mb`, `cuda_available`) match the `GpuInfo` interface in `tauri.ts` lines 28-32. The cast removal is safe.

**If Stuck:**
- If TypeScript reports a type error: Verify the `GpuInfo` interface fields match the Rust JSON keys exactly.

**Agent Directives:**
- Edit one file, one line. Run `npm run build`.

**Verification:**
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] `npm run tauri dev`, GPU detection step in setup wizard still displays correctly

**Approval Gate:** TypeScript builds cleanly and GPU detection works.

---

### Phase 11: Normalize Config import paths across all files
**Priority:** LOW
**Category:** Code Smell
**Files:** `backend/src/commands/dictation.rs` (line 5), `backend/src/model_manager/download.rs` (line 1), `backend/src/output/mod.rs` (line 4)
**Depends On:** None
**Estimated Complexity:** Trivial

**Context:**
`Config` and `OutputMode` are re-exported via `config/mod.rs` line 2: `pub use settings::{Config, OutputMode};`. Some files use the canonical path (`crate::config::Config`), while three files bypass the re-export and reach into the submodule directly. Confirmed by grep:
- `backend/src/commands/dictation.rs` line 5: `use crate::config::settings::Config;`
- `backend/src/model_manager/download.rs` line 1: `use crate::config::settings::Config;`
- `backend/src/output/mod.rs` line 4: `use crate::config::settings::OutputMode;`

**Instructions:**
1. In `backend/src/commands/dictation.rs`, at line 5, change:
   ```rust
   use crate::config::settings::Config;
   ```
   to:
   ```rust
   use crate::config::Config;
   ```
2. In `backend/src/model_manager/download.rs`, at line 1, change:
   ```rust
   use crate::config::settings::Config;
   ```
   to:
   ```rust
   use crate::config::Config;
   ```
3. In `backend/src/output/mod.rs`, at line 4, change:
   ```rust
   use crate::config::settings::OutputMode;
   ```
   to:
   ```rust
   use crate::config::OutputMode;
   ```

**Gotcha Alerts:**
- The re-exports exist in `backend/src/config/mod.rs` line 2: `pub use settings::{Config, OutputMode};`. Both types are covered.
- Verify the re-export includes `OutputMode` (it does -- confirmed by reading `config/mod.rs`).

**If Stuck:**
- If `cargo check` fails: Verify `backend/src/config/mod.rs` contains the `pub use` line.

**Agent Directives:**
- Edit three files. Run `cargo check`. Then grep for `config::settings::` to confirm zero remaining instances.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `grep -r "config::settings::" backend/src/` returns zero results

**Approval Gate:** `cargo check` passes and all imports are consistent.

---

### Phase 12: Remove unused `_toggle` and `isListening`
**Priority:** LOW
**Category:** Code Smell
**Files:** `frontend/src/pages/main-window.tsx` (line 13), `frontend/src/hooks/use-dictation.ts` (lines 7, 13, 24, 31)
**Depends On:** Phase 2, Phase 3 (they modify `use-dictation.ts` first, changing line numbers)
**Estimated Complexity:** Simple

**Context:**
In `main-window.tsx` at line 13, `toggle` is destructured as `_toggle` to suppress unused-variable warnings. The main window relies entirely on the global hotkey. `isListening` in `use-dictation.ts` is set but never read by any consumer (confirmed: grep found `isListening` only in `use-dictation.ts` itself).

**Instructions:**
1. In `frontend/src/pages/main-window.tsx`, simplify the destructure (currently at line 13, but may have shifted after Phase 2 edits):
   ```typescript
   const { status, error } = useDictation();
   ```
   Remove `toggle: _toggle`. Note: After Phase 2, `error` is also in the destructure.
2. In `frontend/src/hooks/use-dictation.ts`:
   - Remove the `isListening` state declaration: `const [isListening, setIsListening] = useState(false);`
   - Remove the `setIsListening(newStatus === "listening");` call inside the `onDictationStatus` handler.
   - Remove the `setIsListening(result);` call inside the `toggle` callback.
   - Remove `isListening` from the return object. The return becomes: `return { status, toggle, error };`
3. Keep `toggle` in the hook's return value. It is not used by `MainWindow` but could be used by other components in the future.

**Gotcha Alerts:**
- After Phase 2 and Phase 3, the line numbers in `use-dictation.ts` will have shifted. Read the file fresh before editing.
- `toggle` must stay in the hook's return object. Only remove it from the `MainWindow` destructure.
- Verify no other file imports or uses `isListening` from `useDictation`. Confirmed by grep: only `use-dictation.ts` references it.

**If Stuck:**
- If `npm run build` fails with "Property 'isListening' does not exist": A component still references it. Grep for `isListening` in `frontend/src/`.

**Agent Directives:**
- Edit two files. Run `npm run build` after both edits.

**Verification:**
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] Grep for `isListening` in `frontend/src/` returns zero results
- [ ] `npm run tauri dev` -- dictation toggle via hotkey still works, status indicator updates

**Approval Gate:** Build passes and dictation status still works.

---

## Section D: Dead Code Removal (Phases 13-16)

---

### Phase 13: Remove dead backend stubs: storage.rs, hotkey/
**Priority:** LOW
**Category:** Dead Code
**Files:** `backend/src/model_manager/storage.rs`, `backend/src/model_manager/mod.rs` (line 2), `backend/src/hotkey/manager.rs`, `backend/src/hotkey/mod.rs`, `backend/src/lib.rs` (line 4)
**Depends On:** None
**Estimated Complexity:** Simple

**Context:**
`storage.rs` contains a stub `list()` function with a TODO. `hotkey/manager.rs` contains only comments. The actual hotkey logic is in `lib.rs`. Both are dead code. Confirmed by grep: no file imports from `crate::hotkey`.

**Instructions:**
1. Delete file: `backend/src/model_manager/storage.rs`
2. In `backend/src/model_manager/mod.rs`, remove line 2: `pub mod storage;`
   After removal, `mod.rs` should contain:
   ```rust
   pub mod download;
   pub use download::{delete_model, download_model, is_model_downloaded};
   ```
3. Delete file: `backend/src/hotkey/manager.rs`
4. Delete file: `backend/src/hotkey/mod.rs`
5. Delete directory: `backend/src/hotkey/`
6. In `backend/src/lib.rs`, remove line 4: `pub mod hotkey;`

**Gotcha Alerts:**
- Nothing imports from `crate::hotkey` (confirmed by grep). Safe to remove the entire module.
- `model_manager/mod.rs` has `pub mod storage;` at line 2. After removing `storage.rs`, this line must go too or `cargo check` fails with "file not found for module".
- Use `git rm` for file deletions so they are tracked in version control.

**If Stuck:**
- If `cargo check` fails with "unresolved import `crate::hotkey`": Something imports from the hotkey module. Grep for it and remove the import.
- If `cargo check` fails with "file not found for module": A `mod` declaration references a deleted file. Remove the `mod` line.

**Agent Directives:**
- Step 1: Delete files with `git rm`.
- Step 2: Edit `model_manager/mod.rs` and `lib.rs`.
- Step 3: Run `cargo check`.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `ls backend/src/model_manager/storage.rs` fails (file gone)
- [ ] `ls backend/src/hotkey/` fails (directory gone)
- [ ] `npm run tauri dev` -- app launches and hotkey still works

**Approval Gate:** `cargo check` passes and app works.

---

### Phase 14: Remove unused backend methods and hound dependency
**Priority:** LOW
**Category:** Dead Code
**Files:** `backend/src/audio/capture.rs` (lines 72-78), `backend/src/audio/vad.rs` (lines 69-73), `backend/src/audio/buffer.rs` (lines 72-77), `backend/Cargo.toml` (line 28)
**Depends On:** Phase 1 (Phase 1 changes `capture.rs` struct and `start()` method; this phase deletes `stop()` and `is_active()` which may have already been removed or relocated by Phase 1. Read the file fresh.)
**Estimated Complexity:** Simple

**Context:**
Several methods are defined but never called:
- `AudioCapture::stop()` (capture.rs lines 72-74) -- pipeline drops `AudioCapture` to stop; never calls `stop()` explicitly
- `AudioCapture::is_active()` (capture.rs lines 76-78) -- not called anywhere
- `VoiceActivityDetector::reset()` (vad.rs lines 69-73) -- not called anywhere; VAD is created fresh each pipeline start
- `AudioRingBuffer::clear()` (buffer.rs lines 72-77) -- not called anywhere
- `hound` crate (Cargo.toml line 28) -- never imported anywhere (confirmed by grep: zero `use hound` results)

**Instructions:**
1. Read `backend/src/audio/capture.rs` FRESH (Phase 1 may have changed it). Find and delete the `stop()` and `is_active()` methods. After Phase 1, the field name may have changed from `sample_rate` to `device_sample_rate`, and a `device_channels` field was added. The `stop()` and `is_active()` methods should still exist unchanged unless Phase 1 removed them.
2. In `backend/src/audio/vad.rs`, delete the `reset()` method (lines 69-73).
3. In `backend/src/audio/buffer.rs`, delete the `clear()` method (lines 72-77).
4. In `backend/Cargo.toml`, remove line 28: `hound = "3"`.

**Gotcha Alerts:**
- Phase 1 changes `capture.rs` significantly. Read the file AFTER Phase 1 is complete to get correct line numbers.
- The tests in `buffer.rs` (lines 80-105) do NOT use `clear()`. The tests in `vad.rs` (lines 76-97) do NOT use `reset()`. Safe to delete without test changes.
- After removing `hound` from `Cargo.toml`, no other crate depends on it transitively.

**If Stuck:**
- If `cargo check` fails with "method not found": Something calls one of the deleted methods. Grep for the method name and fix the caller.
- If `cargo build` fails after removing `hound`: A transitive dependency issue. Restore `hound` and investigate.

**Agent Directives:**
- Read all three source files fresh before editing (they may have changed from earlier phases).
- Edit three source files and one config file. Run `cargo check` after all edits.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `cargo build` from `backend/` -- builds successfully without hound
- [ ] `npm run tauri dev` -- app still works

**Approval Gate:** `cargo check` and `cargo build` pass cleanly.

---

### Phase 15: Remove dead frontend page and unused state
**Priority:** LOW
**Category:** Dead Code
**Files:** `frontend/src/pages/setup.tsx`, `frontend/src/hooks/use-models.ts` (lines 7, 24-27, 66)
**Depends On:** Phase 5 (Phase 5 modifies `use-models.ts`; this phase also modifies it)
**Estimated Complexity:** Trivial

**Context:**
`setup.tsx` exports `SetupPage` but is never imported (confirmed: grep for `from.*pages/setup` returned zero results). The actual setup wizard is `components/setup-wizard/index.tsx`. The `downloadProgress` state in `use-models.ts` is returned from the hook but never consumed by any component (confirmed: only `use-models.ts` itself references `downloadProgress`).

**Instructions:**
1. Delete file: `frontend/src/pages/setup.tsx`
2. In `frontend/src/hooks/use-models.ts` (read FRESH after Phase 5):
   - Remove the `downloadProgress` state declaration: `const [downloadProgress, setDownloadProgress] = useState<Record<string, number>>({});`
   - In the `onDownloadProgress` event handler (lines 23-31), remove the `setDownloadProgress(...)` call. Keep the `if (data.percent >= 100) { refresh(); }` logic -- this is needed to refresh the model list after downloads complete.
   - Remove `downloadProgress` from the return object.
   After edit, the event handler should look like:
   ```typescript
   const unlisten = events.onDownloadProgress((data) => {
     if (data.percent >= 100) {
       refresh();
     }
   });
   ```

**Gotcha Alerts:**
- The setup wizard's download progress bar is independent. It uses its own local state in `step-download.tsx` (line 10). Removing `downloadProgress` from `use-models.ts` does NOT affect the setup wizard.
- Keep the `onDownloadProgress` listener -- it calls `refresh()` on download completion, which is needed to update the model list.
- After Phase 5, `use-models.ts` will have a `loading` state and `finally` block. Read the file fresh.

**If Stuck:**
- If `npm run build` fails with "Property 'downloadProgress' does not exist": A component references it. Grep for `downloadProgress` in `frontend/src/` (excluding `use-models.ts`).

**Agent Directives:**
- Delete one file, edit one file. Run `npm run build`.

**Verification:**
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] `ls frontend/src/pages/setup.tsx` fails (file gone)
- [ ] `npm run tauri dev` -- setup wizard and model downloads still work correctly

**Approval Gate:** Build passes and setup wizard works.

---

### Phase 16: Remove unused shadcn UI components and deps
**Priority:** LOW
**Category:** Dead Code
**Files:** `frontend/src/components/ui/*.tsx` (10 files), `/home/frosty/local-stt/package.json`
**Depends On:** None
**Estimated Complexity:** Simple

**Context:**
10 shadcn UI component files exist in `frontend/src/components/ui/` but none are imported by any application component. Confirmed by grep: the ONLY import of any UI component is `dialog.tsx` importing `button.tsx` -- both are dead code since `dialog.tsx` itself is never imported by application code.

The components are: `button.tsx`, `card.tsx`, `select.tsx`, `dropdown-menu.tsx`, `dialog.tsx`, `scroll-area.tsx`, `badge.tsx`, `separator.tsx`, `tooltip.tsx`, `progress.tsx`.

**Instructions:**
1. Delete ALL 10 component files:
   ```
   frontend/src/components/ui/button.tsx
   frontend/src/components/ui/card.tsx
   frontend/src/components/ui/select.tsx
   frontend/src/components/ui/dropdown-menu.tsx
   frontend/src/components/ui/dialog.tsx
   frontend/src/components/ui/scroll-area.tsx
   frontend/src/components/ui/badge.tsx
   frontend/src/components/ui/separator.tsx
   frontend/src/components/ui/tooltip.tsx
   frontend/src/components/ui/progress.tsx
   ```
   All 10 are confirmed unused by application code. `dialog.tsx` imports `button.tsx`, but `dialog.tsx` itself is never imported.
2. Run `npm run build` to confirm no breakage.
3. Check if `frontend/src/lib/utils.ts` exists. If it does, check if any REMAINING component (non-UI) imports `cn` from it. If not, delete `lib/utils.ts` too.
   Confirmed by grep: `cn` from `@/lib/utils` is imported ONLY by the 10 UI components being deleted. No application component imports it. So delete `lib/utils.ts` if it exists.
4. Audit `package.json` dependencies after the deletions:
   - `lucide-react` (devDeps line 28): Only imported by `dialog.tsx`, `dropdown-menu.tsx`, `select.tsx` -- all being deleted. **Remove it.**
   - `radix-ui` (deps line 16): Only imported by the shadcn UI components. **Remove it.**
   - `class-variance-authority` (deps line 15): Only imported by `button.tsx`. **Remove it.**
   - `clsx` (devDeps line 27): Only used by `lib/utils.ts` (via the `cn` function). **Remove it.**
   - `tailwind-merge` (devDeps line 30): Only used by `lib/utils.ts`. **Remove it.**
   - `shadcn` (devDeps line 29): CLI tool for installing shadcn components. No longer needed if all components are removed. **Remove it.**
   - `tw-animate-css` (devDeps line 32): Check if any remaining CSS or component uses it. If it is only referenced from `index.css` or shadcn components, it can be removed. Check before removing.
5. After modifying `package.json`, run `npm install` from root to update the lockfile.
6. Run `npm run build` again to confirm everything still compiles.

**Gotcha Alerts:**
- The `package.json` uses `"radix-ui": "^1.4.3"` (unified package), NOT individual `@radix-ui/*` packages. The original plan incorrectly referenced `@radix-ui/*`.
- `lucide-react` is in `devDependencies`, not `dependencies`.
- After removing all UI components and `lib/utils.ts`, check if the `frontend/src/components/ui/` directory is empty. If so, delete it.
- Do the file deletions in one pass, THEN audit `package.json` in a second pass. Run `npm run build` after each pass.

**If Stuck:**
- If `npm run build` fails with "Cannot find module": A component you deleted is imported somewhere. Restore it and check imports.
- If removing a `package.json` dependency breaks the build: The dependency is used transitively. Restore it.

**Agent Directives:**
- Pass 1: Delete the 10 component files (and `lib/utils.ts` if applicable). Run `npm run build`.
- Pass 2: Edit `package.json` to remove unused deps. Run `npm install && npm run build`.
- Check for `tw-animate-css` usage before removing it.

**Verification:**
- [ ] `npm run build` from root -- no TypeScript errors
- [ ] `npm run tauri dev` -- app renders correctly, no missing components
- [ ] `ls frontend/src/components/ui/` -- directory empty or gone
- [ ] No "Cannot find module" errors in the build

**Approval Gate:** Build passes and app renders correctly.

---

## Section E: Optimizations (Phases 17-18)

---

### Phase 17: Eliminate heap allocation in audio callback
**Priority:** MEDIUM
**Category:** Optimization
**Files:** `backend/src/audio/capture.rs`, `backend/src/audio/mod.rs`, `backend/Cargo.toml`
**Depends On:** Phase 1 (Phase 1 changes the audio pipeline structure; this phase MUST build on Phase 1's output)
**Estimated Complexity:** Moderate

**Context:**
After Phase 1, the cpal audio callback in `capture.rs` does `sender.send(data.to_vec()).ok()` -- allocating a new `Vec<f32>` on every callback invocation (~every 5-10ms). The `mpsc::Sender::send()` also acquires an internal mutex. Both operations are inappropriate for the real-time audio thread.

This phase replaces the `mpsc::channel` between the cpal callback and the pipeline thread with a lock-free ring buffer (`ringbuf` crate), eliminating both the allocation and the mutex.

**IMPORTANT:** This phase only replaces the cpal-to-pipeline channel. The pipeline-to-transcription channel (`chunk_tx`/`chunk_rx` in `audio/mod.rs`) stays as `mpsc::channel`. The resampling logic added in Phase 1 stays in the pipeline thread (consumer side).

**Instructions:**
1. Add `ringbuf` to `backend/Cargo.toml`:
   ```toml
   ringbuf = "0.4"
   ```
2. In `backend/src/audio/capture.rs`:
   - Remove `use std::sync::mpsc;` from imports.
   - Add: `use ringbuf::traits::{Producer, Split};` and `use ringbuf::HeapRb;`
   - Change the `start()` method signature: replace `sender: mpsc::Sender<Vec<f32>>` parameter with `producer: ringbuf::HeapProd<f32>`.
   - In the `build_input_stream` callback, replace:
     ```rust
     sender.send(data.to_vec()).ok();
     ```
     with:
     ```rust
     let written = producer.push_slice(data);
     if written < data.len() {
         // Ring buffer full -- samples dropped. Acceptable for speech.
     }
     ```
   - Note: `producer` must be moved into the closure. The existing `move` keyword on the closure handles this.
3. In `backend/src/audio/mod.rs`:
   - Add imports: `use ringbuf::HeapRb;` and `use ringbuf::traits::{Consumer, Split};`
   - In `AudioPipeline::start()`, replace the `mpsc::channel::<Vec<f32>>()` for audio data (currently `let (audio_tx, audio_rx) = mpsc::channel::<Vec<f32>>();` inside the spawned thread) with a ring buffer:
     ```rust
     // 3 seconds of audio at device sample rate, stereo
     let rb_capacity = (48000 * 2 * 3) as usize; // Conservative; will be correct for most devices
     let rb = HeapRb::<f32>::new(rb_capacity);
     let (producer, mut consumer) = rb.split();
     ```
   - Change `capture.start(device_name.as_deref(), audio_tx)` to `capture.start(device_name.as_deref(), producer)`.
   - Replace the blocking `audio_rx.recv_timeout(...)` loop with a polling loop:
     ```rust
     let mut read_buf = vec![0.0f32; 4800]; // 100ms at 48kHz
     while running.load(Ordering::SeqCst) {
         let n = consumer.pop_slice(&mut read_buf);
         if n > 0 {
             let mono = to_mono(&read_buf[..n], device_channels);
             let resampled = resample(&mono, device_rate, 16000);
             buffer.write(&resampled);
             // ... chunk extraction and VAD logic unchanged
         } else {
             std::thread::sleep(std::time::Duration::from_millis(10));
         }
     }
     ```
   - Keep the `init_tx`/`init_rx` synchronization channel as `mpsc` -- it is only used once for init signaling.
   - Keep the `chunk_tx`/`chunk_rx` channel as `mpsc` -- the transcription thread uses blocking `recv()`.

**Gotcha Alerts:**
- `ringbuf` 0.4 API: `HeapRb::new(capacity)` creates the buffer. `.split()` returns `(HeapProd<T>, HeapCons<T>)`. `HeapProd` has `push_slice(&[T]) -> usize`. `HeapCons` has `pop_slice(&mut [T]) -> usize`.
- Capacity is in samples, not bytes. For 3 seconds at 48kHz stereo: `48000 * 2 * 3 = 288000` samples.
- `HeapProd` and `HeapCons` are `Send` but NOT `Sync`. They can be moved to different threads (correct for this use case) but not shared.
- The consumer must poll (no blocking `recv()`). Use `std::thread::sleep(Duration::from_millis(10))` between polls when no data is available.
- Dropped samples (when the ring buffer is full) are acceptable for speech. Do NOT log drops with `eprintln!` in the callback (it allocates). Use an `AtomicUsize` counter if you want to track drops.
- After Phase 1, the pipeline thread already does resampling. This phase changes how data enters the pipeline thread (ring buffer instead of mpsc) but the resampling logic stays the same.

**If Stuck:**
- If audio quality degrades (clicks, pops): Increase ring buffer capacity or decrease poll sleep time.
- If `cargo check` fails with type errors on `Producer`/`Consumer`: Check `ringbuf` 0.4 docs. Import paths may be `ringbuf::traits::{Producer, Consumer, Split}` and types are `ringbuf::HeapProd<f32>` and `ringbuf::HeapCons<f32>`.
- If the pipeline thread never receives data: Ensure producer and consumer come from the same `HeapRb::new(n).split()` call.
- **Alternative escape hatch:** If `ringbuf` proves too complex, use `crossbeam-channel` with a bounded channel and pre-allocated buffers. This is less optimal but simpler.

**Agent Directives:**
- Read BOTH `capture.rs` and `audio/mod.rs` fully AFTER Phase 1 is complete. Phase 1 changes these files significantly.
- Edit `capture.rs` (producer in callback), `audio/mod.rs` (consumer in pipeline), `Cargo.toml` (add ringbuf).
- Run `cargo check`, then test with `npm run tauri dev` -- speak for 30 seconds.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `npm run tauri dev`, start dictation, speak for 30 seconds continuously
- [ ] Transcription still produces correct text
- [ ] Monitor `pw-top` for audio stream health (optional)

**Approval Gate:** Audio capture works without regression and transcription produces correct output.

---

### Phase 18: Make model registry static
**Priority:** LOW
**Category:** Optimization
**Files:** `backend/src/transcription/models.rs` (lines 13-61)
**Depends On:** None
**Estimated Complexity:** Simple

**Context:**
`get_model_registry()` creates a new `Vec<WhisperModel>` with 5 struct allocations on every call. Called from `list_models`, `download_model`, `delete_model`, `is_model_downloaded`, `load_model`. The registry is constant data and should be allocated once.

**Instructions:**
1. In `backend/src/transcription/models.rs`, replace the `get_model_registry()` function (lines 13-61) with:
   ```rust
   use std::sync::LazyLock;

   static MODEL_REGISTRY: LazyLock<Vec<WhisperModel>> = LazyLock::new(|| {
       vec![
           // ... paste the existing 5 WhisperModel entries here unchanged ...
       ]
   });

   pub fn get_model_registry() -> &'static [WhisperModel] {
       &MODEL_REGISTRY
   }
   ```
   Return `&'static [WhisperModel]` (slice) instead of `&'static Vec<WhisperModel>`. Slices are more idiomatic.
2. All call sites iterate over the result with `.iter()` and `.find()`. Both work on `&[WhisperModel]`. No call site changes needed. Verified call sites:
   - `commands/models.rs` line 33: `get_model_registry().iter().map(...)` -- works
   - `commands/models.rs` line 72: `let registry = get_model_registry(); registry.iter().find(...)` -- works
   - `model_manager/download.rs` line 10: `let registry = get_model_registry(); registry.iter().find(...)` -- works
   - `model_manager/download.rs` line 91: same pattern -- works
   - `model_manager/download.rs` line 105: same pattern -- works
3. The re-export in `transcription/mod.rs` (`pub use models::{get_model_registry, WhisperModel};`) continues to work. The function signature change is transparent to callers importing via `crate::transcription::get_model_registry`.
4. Update the test at line 69 if needed. `let registry = get_model_registry();` now returns `&[WhisperModel]`. `registry.is_empty()` works on slices. `registry.iter().any(...)` works on slices. No test changes needed.

**Gotcha Alerts:**
- `std::sync::LazyLock` is stable since Rust 1.80. This system runs rustc 1.93.0. Safe to use.
- `WhisperModel` fields are all `String`. The `LazyLock` initializer calls `.to_string()` on literals, which allocates. This happens exactly once (on first access).
- `LazyLock` is `Sync`. Multiple threads can call `get_model_registry()` concurrently. The first call initializes; subsequent calls return the cached reference.

**If Stuck:**
- If `cargo check` fails with "LazyLock not found": Import with `use std::sync::LazyLock;`. On older Rust (< 1.80), use `once_cell::sync::Lazy` instead and add `once_cell = "1"` to `Cargo.toml`.
- If lifetime errors appear at call sites: A function expects `Vec<WhisperModel>` (owned). Change it to `&[WhisperModel]`. Based on current code, no such function exists.

**Agent Directives:**
- Edit one file: `backend/src/transcription/models.rs`.
- Run `cargo check` and `cargo test` from `backend/`.

**Verification:**
- [ ] `cargo check` from `backend/` -- no compilation errors
- [ ] `cargo test` from `backend/` -- all tests pass (including `test_registry_contains_models`)
- [ ] `npm run tauri dev`, check model list displays correctly, model download and deletion work

**Approval Gate:** This is the final code phase. After verification, proceed to the End-to-End Acceptance Test.

---

## Phase E2E: End-to-End Acceptance Test
**Priority:** CRITICAL
**Category:** Validation
**Files:** All
**Depends On:** All previous phases
**Estimated Complexity:** N/A

**Instructions:**
Perform a full end-to-end test of the entire application pipeline:

1. **Clean start:** Delete `~/.whispertype/` directory. Launch with `npm run tauri dev`.
2. **Setup wizard:** Confirm the wizard appears. GPU detection shows GPU name and VRAM. Select the `tiny` model. Download it -- progress bar works and completes.
3. **StepComplete:** Confirm the final setup step shows a checkmark icon (not the word "Microphone").
4. **Model auto-load:** After completing setup, confirm main window shows the model is loaded.
5. **Dictation start:** Press Ctrl+Shift+Space. Status changes to "Listening". No errors in terminal or UI.
6. **Transcription:** Speak clearly for 10 seconds. Transcription text appears within seconds.
7. **Text output:** Open a text editor, start dictation, speak a sentence. Text is typed/copied into the editor.
8. **Dictation stop:** Press Ctrl+Shift+Space. Status returns to "Idle". No orphan threads or errors.
9. **Error handling:** Temporarily edit `~/.whispertype/config.json` and set `output_mode` to an invalid value (e.g., `"invalid"`). Restart the app. Confirm an error appears rather than silent failure. Revert.
10. **Rapid toggling:** Press Ctrl+Shift+Space 10 times rapidly. No panics, thread errors, or UI glitches.
11. **Settings persistence:** Open settings, change a setting, close and reopen app. Confirm the setting persisted.
12. **Window title:** Confirm title bar shows "WhisperType".
13. **CSP check:** Open DevTools (F12). Confirm no CSP violation messages in Console.
14. **Build check:** `cargo check` from `backend/` -- zero warnings. `npm run build` from root -- zero TypeScript errors.
15. **Dead code check:** `cargo check` should not warn about unused functions. If it does, investigate.

**Verification:**
- [ ] All 15 checks above pass

**Approval Gate:** Plan is complete when all checks pass.

---

## Build Command Reference

| Task | Command | Working Directory |
|------|---------|-------------------|
| Rust type check | `cargo check` | `/home/frosty/local-stt/backend/` |
| Rust build | `cargo build` | `/home/frosty/local-stt/backend/` |
| Rust tests | `cargo test` | `/home/frosty/local-stt/backend/` |
| Rust lint | `cargo clippy` | `/home/frosty/local-stt/backend/` |
| TypeScript build | `npm run build` | `/home/frosty/local-stt/` |
| Full dev run | `npm run tauri dev` | `/home/frosty/local-stt/` |
| Install deps | `npm install` | `/home/frosty/local-stt/` |

## File Location Quick Reference

All paths relative to `/home/frosty/local-stt/`:

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
| Config/settings | `backend/src/config/settings.rs`, `backend/src/config/mod.rs` |
| Model download | `backend/src/model_manager/download.rs`, `backend/src/model_manager/mod.rs` |
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
| Setup wizard | `frontend/src/components/setup-wizard/index.tsx` |
| Step GPU | `frontend/src/components/setup-wizard/step-gpu.tsx` |
| Step Complete | `frontend/src/components/setup-wizard/step-complete.tsx` |
| Settings panel | `frontend/src/components/settings-panel.tsx` |
| HTML entry | `index.html` |
| NPM config | `package.json` |

## Dependency Graph (data flow)

```
cpal (audio device)
  -> AudioCapture::start() callback [capture.rs]
    -> ringbuf::HeapProd (after Phase 17; mpsc::Sender before)
      -> AudioPipeline thread [audio/mod.rs]
        -> to_mono() + resample() [audio/mod.rs] (Phase 1)
        -> AudioRingBuffer [buffer.rs] (receives 16kHz mono)
        -> VoiceActivityDetector [vad.rs]
        -> mpsc::Sender<Vec<f32>> (speech chunks)
          -> Transcription thread [dictation.rs]
            -> TranscriptionEngine::transcribe() [engine.rs]
              -> whisper-rs (WhisperContext, FullParams)
            -> output::output_text() [output/mod.rs]
              -> keyboard::type_text() [keyboard.rs] (enigo)
              -> clipboard::copy_to_clipboard() [clipboard.rs] (arboard)
            -> app.emit("transcription-update", ...)
            -> app.emit("output-error", ...) (Phase 2)
            -> app.emit("transcription-error", ...) (Phase 3)
              -> Frontend event listeners
                -> useDictation hook -> error state
                -> useTranscription hook -> TranscriptDisplay
```
