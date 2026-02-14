# PHASES-FIX.md -- Critical Review

Reviewer: Senior Code Review Agent
Date: 2026-02-13
Verdict: **Plan has significant gaps. Do NOT execute without addressing findings below.**

---

## 1. DEPENDENCY AUDIT

### 1.1 Phase Ordering Issues

**PHASE 1 <-> PHASE 17 CONFLICT (CRITICAL)**
Phase 1 (audio capture resampling) and Phase 17 (ring buffer for audio callback) both modify `backend/src/audio/capture.rs` and fundamentally change the audio data flow. Phase 1 says "replace the hardcoded `StreamConfig` with `device.default_input_config()`" and potentially add resampling in the callback or the processing thread. Phase 17 says "replace the `mpsc::channel()` with a `ringbuf::HeapRb`" inside the same callback. These two changes are deeply intertwined:

- If Phase 1 adds resampling *inside* the callback (option 5 in the instructions), Phase 17 then rips out the entire channel mechanism the resampling was attached to.
- If Phase 1 adds resampling in the transcription thread (option 4), then Phase 17's switch to a ring buffer changes the consumer API from `receiver.recv()` to a polling loop, breaking whatever resampling logic was inserted.
- **The agent executing Phase 17 will effectively need to re-implement Phase 1's resampling around the new ring buffer architecture.**

Recommendation: Merge Phase 1 and Phase 17 into a single phase, or at minimum Phase 17 must explicitly state it will re-integrate the resampling logic from Phase 1.

**PHASE 1 DOES NOT ACCOUNT FOR AudioPipeline (CRITICAL)**
Phase 1's instructions focus exclusively on `backend/src/audio/capture.rs`. But the hardcoded `16000` also appears in `backend/src/audio/mod.rs` at line 54:
```rust
let mut buffer = AudioRingBuffer::new(16000, chunk_duration_ms, overlap_ms, 30);
```
If Phase 1 implements resampling Option 5 (resample inside `AudioCapture::start()` before sending over the channel), this line is fine because the receiver always gets 16kHz. But if Phase 1 implements Option 4 (resample in the transcription thread), this `AudioRingBuffer` is still sized for 16kHz while receiving 44.1kHz or 48kHz data. The buffer would overflow silently, the chunk sizes would be wrong, and VAD frame sizes (hardcoded to 480 = 30ms at 16kHz in `vad.rs` line 57) would be incorrect.

The plan's instructions give the agent **two contradictory options (steps 4 and 5) without deciding which one to use**. An executing agent must choose, and the wrong choice breaks the pipeline in ways not caught until runtime.

Recommendation: The plan must commit to ONE approach and enumerate ALL files that need changes. If resampling in the callback (option 5), only `capture.rs` changes. If resampling in the consumer, then `capture.rs`, `audio/mod.rs` (buffer sample rate), and `audio/vad.rs` (frame size) all need updates.

**PHASE 4 DOES NOT UPDATE `stop_dictation` COMMAND**
Phase 4 adds a `transcription_thread: Mutex<Option<JoinHandle<()>>>` field to `AppState` and instructs the agent to join the thread in `toggle_dictation_inner()`. However, the `stop_dictation` command at line 87-93 also stops dictation but is a completely separate code path. It calls `pipeline.stop()` but would NOT join the transcription thread under Phase 4's instructions. This means `stop_dictation` (callable via IPC from the frontend) remains a thread-leaking code path even after the fix.

Recommendation: Phase 4 must explicitly instruct the agent to also join the thread in `stop_dictation()`.

### 1.2 Implicit Dependencies (Merge Conflict Risk)

**PHASES 2 and 3 both modify the same file regions.** Both touch `backend/src/commands/dictation.rs` (the spawned thread body) and `frontend/src/hooks/use-dictation.ts`. An agent executing Phase 3 after Phase 2 will find the file already changed from its expected baseline. The instructions reference specific line numbers (e.g., "line 62") that will have shifted after Phase 2's edits. This is manageable but fragile.

**PHASES 1, 14, and 17 all modify `backend/src/audio/capture.rs`.** Phase 1 changes the stream config and callback. Phase 14 deletes `stop()` and `is_active()` methods. Phase 17 replaces the channel with a ring buffer. If any phase is reverted, the others break.

**PHASES 14 and 17 both modify `backend/Cargo.toml`.** Phase 14 removes `hound`. Phase 17 adds `ringbuf`. No conflict, but ordering matters for a clean git history.

### 1.3 No Circular Dependencies Found

The phase ordering is a DAG. No cycles detected.

---

## 2. AMBIGUITY SCAN

### 2.1 Critical Ambiguities

**PHASE 1: Two contradictory implementation paths with no decision.**
Steps 4 and 5 describe mutually exclusive approaches:
- Step 4: "add a resampling step in the transcription thread (in `backend/src/commands/dictation.rs`)"
- Step 5: "Alternatively, implement the resampling inside `AudioCapture::start()` before sending over the channel"

An agent must choose. The plan does not state which is preferred. Each choice has different downstream consequences for buffer sizing, VAD frame sizing, and Phase 17 compatibility. Step 6 says "Or keep it as the *target* sample rate" -- more ambiguity.

**PHASE 1: "A simple linear interpolation resampler is sufficient for speech."**
This is hand-waving. A correct linear interpolation resampler for arbitrary rate conversion (e.g., 44100->16000, ratio 0.3628...) is NOT trivial. The agent needs to either:
- Write a fractional resampler from scratch (error-prone)
- Use a crate like `rubato` or `dasp` (not mentioned, not in Cargo.toml)
- Use a naive "skip every Nth sample" approach (destroys audio quality)

The plan should specify which resampling crate to use or provide the resampling algorithm explicitly.

**PHASE 1: No mention of stereo-to-mono conversion specifics.**
Step 4 says "convert stereo to mono if needed." How? Average the channels? Take the left channel? The plan doesn't specify. For a speech application, channel averaging is standard, but this is left to the agent to guess.

**PHASE 4: "Initialize it as `Mutex::new(None)` wherever `AppState` is constructed (in `backend/src/lib.rs`)."**
The word "wherever" is vague. There is exactly one construction site (lib.rs line 19-23), but the agent shouldn't have to search for it. The plan should say: "In `backend/src/lib.rs` at line 19, add `transcription_thread: Mutex::new(None)` to the `AppState` struct literal."

**PHASE 6: CSP string may be wrong.**
The CSP includes `connect-src ipc: http://ipc.localhost https://huggingface.co https://*.huggingface.co`. However:
- Tauri v2 uses `ipc:` protocol. The `http://ipc.localhost` may or may not be needed depending on the Tauri version's IPC implementation. If wrong, all IPC commands break and the app is dead.
- The main-window.tsx at line 86 uses `navigator.clipboard.writeText()`. Some CSP configurations block clipboard access. The plan doesn't address whether the proposed CSP permits Clipboard API usage.
- The plan says `'unsafe-inline'` is needed for "Tailwind/CSS-in-JS", but Tailwind v4 (used here per package.json: `tailwindcss ^4.1.18`) generates utility classes at build time, not runtime inline styles. The `'unsafe-inline'` may be unnecessary and weakens the CSP. But removing it could break things if any component uses inline styles. The plan doesn't investigate this.

The verification step says "confirm no CSP errors in the webview console" -- but this requires manual testing. An agent that only does `cargo check` and `npm run build` will not catch CSP regressions.

**PHASE 13: "consider whether the entire `hotkey` module should be removed."**
This is a conditional instruction that forces the agent to make a judgment call. Let me resolve it here: `backend/src/hotkey/mod.rs` contains only `pub mod manager;`. If `manager.rs` is deleted and that line is removed, `mod.rs` is empty. `backend/src/lib.rs` line 4 declares `pub mod hotkey;`. Since nothing else exists in the hotkey module, the entire module (directory + `mod hotkey;` declaration) should be removed. The plan should state this definitively.

**PHASE 16: "check if any `@radix-ui/*` packages in `frontend/package.json` are only used by the deleted components."**
The actual `package.json` at `/home/frosty/local-stt/package.json` uses `"radix-ui": "^1.4.3"` (the unified package), not individual `@radix-ui/*` packages. The plan's instruction to search for `@radix-ui/*` packages is based on a wrong assumption about the dependency structure. The agent would search, find nothing matching `@radix-ui/*`, and move on without cleaning up `radix-ui`.

Also, `lucide-react` is in `devDependencies` (line 28 of package.json), not `dependencies`. The plan doesn't note this distinction.

### 2.2 Minor Ambiguities

**PHASE 2: "a simple red text banner or toast below the status indicator is sufficient."**
This gives the agent artistic license but no specific implementation. An agent with zero project context would need to decide on layout, positioning, animation, and timeout behavior. Not a blocker, but increases variance.

**PHASE 8: "Or use a Unicode speech/microphone emoji if preferred."**
The plan's primary suggestion uses an HTML entity (`&#10003;`) but also suggests emojis. An agent might use an emoji, which could render inconsistently across systems. The plan should pick one.

**PHASE 12: "remove it from the return object (line 31)".**
After Phase 2 and Phase 3 have modified `use-dictation.ts`, the return object will no longer be at line 31. Line number references to files that earlier phases modify are unreliable.

**PHASE 18: Return type change from `Vec<WhisperModel>` to `&'static Vec<WhisperModel>` is underspecified.**
The plan says "Update all call sites that use `get_model_registry()` to work with `&Vec<WhisperModel>` instead of `Vec<WhisperModel>`." It lists callers in `commands/models.rs` and `model_manager/download.rs`, but the actual impact depends on what each caller does. For instance, `download.rs` line 10 does `let registry = get_model_registry();` and then `registry.iter().find(...)`. This would work with a reference. But if any caller does `let mut registry = get_model_registry()` or passes it to something expecting owned, it breaks. The plan should audit each call site concretely.

Additionally, `get_model_registry()` is re-exported via `backend/src/transcription/mod.rs` line 3: `pub use models::{get_model_registry, WhisperModel};`. The return type change from `Vec<WhisperModel>` to `&'static Vec<WhisperModel>` changes the public API of the `transcription` module. All consumers of `crate::transcription::get_model_registry()` must also be checked. The plan lists `commands/models.rs` but it imports via `use crate::transcription::{get_model_registry, WhisperModel};` (line 6). This should work with a reference, but the plan should be explicit.

---

## 3. COMPLETENESS CHECK

### 3.1 Coverage Matrix

| Finding | Phase | Status | Notes |
|---------|-------|--------|-------|
| BUG-1: Hardcoded 16kHz | Phase 1 | COVERED (with issues) | See ambiguity findings above |
| BUG-2: Thread leak on rapid toggle | Phase 4 | PARTIALLY COVERED | `stop_dictation` not updated |
| BUG-3: output_text errors swallowed | Phase 2 | COVERED | |
| BUG-4: Transcription errors only stderr | Phase 3 | COVERED | |
| BUG-5: Model auto-load race | Phase 5 | COVERED | |
| BUG-6: CSP is null | Phase 6 | COVERED (with risks) | See CSP ambiguity above |
| SMELL-1: Dead storage.rs | Phase 13 | COVERED | |
| SMELL-2: Dead hotkey/manager.rs | Phase 13 | COVERED | |
| SMELL-3: Dead setup.tsx | Phase 15 | COVERED | |
| SMELL-4: Inconsistent Config imports | Phase 11 | **INCOMPLETE** | See section 3.2 |
| SMELL-5: StepComplete placeholder | Phase 8 | COVERED | |
| SMELL-6: _toggle unused | Phase 12 | COVERED | |
| SMELL-7: isListening never consumed | Phase 12 | COVERED | |
| SMELL-8: Unnecessary unsafe main.rs | Phase 9 | **WRONG** | See section 3.3 |
| SMELL-9: Unnecessary cast StepGpu | Phase 10 | COVERED | |
| SMELL-10: hound unused | Phase 14 | COVERED | |
| DEAD: AudioCapture::stop(), is_active() | Phase 14 | COVERED | |
| DEAD: VoiceActivityDetector::reset() | Phase 14 | COVERED | |
| DEAD: AudioBuffer::clear() | Phase 14 | COVERED | |
| DEAD: Unused shadcn UI (10 files) | Phase 16 | COVERED (with issues) | See radix-ui finding above |
| DEAD: downloadProgress state | Phase 15 | COVERED | |
| OPT-1: Heap alloc in callback | Phase 17 | COVERED (conflict with Phase 1) | |
| OPT-2: Whisper ctx Mutex held during inference | -- | **MISSING** | See section 3.4 |
| OPT-3: get_model_registry() allocates | Phase 18 | COVERED | |
| OPT-4: VAD uses simple RMS | -- | NOTED (no phase needed) | Acceptable |
| OPT-5: React memoization | -- | NOTED (no phase needed) | Acceptable |

### 3.2 SMELL-4 Incomplete: `model_manager/download.rs` Also Uses Direct Path

Phase 11 only fixes `commands/dictation.rs`, changing `use crate::config::settings::Config` to `use crate::config::Config`. But `backend/src/model_manager/download.rs` line 1 has the exact same inconsistent import:
```rust
use crate::config::settings::Config;
```
This file is not listed in Phase 11's scope. After Phase 11, the codebase will STILL have an inconsistent import. The verification step says "Grep for `config::settings::Config` across the codebase to confirm no other inconsistent imports remain" -- this would catch it, but the agent might not know to also fix `download.rs` since it's not in the instructions.

### 3.3 SMELL-8 Is Factually Wrong: `set_var` IS Unsafe

Phase 9 claims: "In Rust edition 2021 (which this project uses per `backend/Cargo.toml`), `set_var` is a safe function. The `unsafe` block is unnecessary noise."

**This is incorrect.** `std::env::set_var` was made `unsafe` starting in Rust 1.83 (stable December 2024), regardless of edition. The change applies to edition 2024, but even on edition 2021 with a modern compiler, calling `set_var` without `unsafe` will produce a deprecation warning (and eventually an error). On a Rust toolchain >= 1.83, removing the `unsafe` block will cause a compiler warning or error.

Since this system is running a 2026 kernel (Linux 6.19.0), the Rust toolchain is almost certainly >= 1.83. **Executing Phase 9 as written will break the build.**

The correct fix would be to verify the compiler version first. If the compiler requires `unsafe` for `set_var`, the `unsafe` block should stay. The phase should be either:
- Removed entirely (the `unsafe` is correct and necessary on modern Rust)
- Changed to acknowledge the `unsafe` is correct but add a `// SAFETY:` comment explaining why it's sound (single-threaded at this point in `main()`)

### 3.4 OPT-2 Missing: No Phase for Whisper Context Mutex Contention

The audit identified that the `ctx` Mutex in `TranscriptionEngine` is held for the entire duration of `transcribe()` (which can take seconds for a 3-second audio chunk on GPU). This means:
- `load_model()` blocks while transcription is in progress
- `unload_model()` blocks while transcription is in progress
- `is_loaded()` blocks while transcription is in progress

The original audit noted this "may not need a phase," which is acceptable. But the plan has NO mention of it at all, not even a "noted but deferred" entry. For completeness, it should appear in a "Known Limitations" or "Deferred" section.

### 3.5 Verification Steps Assessment

| Phase | Verification Quality | Issues |
|-------|---------------------|--------|
| 1 | Good | "Test with at least one real microphone" is correct but hard for CI |
| 2 | Good | Includes inject-error-and-check approach |
| 3 | Good | Same pattern as Phase 2 |
| 4 | Good | Specific observable: "rapidly toggle 10 times" |
| 5 | Adequate | "Status indicator should show model loaded" is observable |
| 6 | **Weak** | Requires manual DevTools check. No automated test. |
| 7 | Adequate | Observable: window title |
| 8 | Adequate | Observable: visual check |
| 9 | **WRONG** | "no compilation errors" -- will FAIL because removing `unsafe` on modern Rust is a compile error |
| 10 | Good | Build check + visual |
| 11 | Good | Includes grep verification |
| 12 | Good | Includes search for residual usage |
| 13 | Good | Includes file-not-exists check |
| 14 | Good | Both check and build |
| 15 | Good | Build + file-not-exists |
| 16 | Good | Thorough pre-deletion search |
| 17 | Adequate | Xrun monitoring is good but impractical for automated CI |
| 18 | Good | Functional verification |

### 3.6 End-to-End Test Phase

The Post-Fix Validation section (12 checks) is thorough and well-structured. However:
- Step 8 says "Temporarily set the output mode to an invalid value" -- there is no way to do this through the UI (the dropdown has fixed options). The agent would need to manually edit `~/.whispertype/config.json`. This should be stated explicitly.
- Step 8 also says "unplug the microphone mid-dictation" -- this is hardware-dependent and not automatable.
- There is no step that verifies the dead code removal didn't break anything (e.g., "confirm `cargo check` produces zero warnings about unused code").

---

## 4. RISK ASSESSMENT

### 4.1 Cascading Failure Risk (ranked highest to lowest)

1. **Phase 9 (Remove unsafe) -- WILL BREAK BUILD.** On any Rust toolchain >= 1.83, `std::env::set_var` requires `unsafe`. Executing this phase as written will cause a compile error. If the agent force-removes it and suppresses the warning, it may break on stricter CI settings. **Probability of failure: ~95%.**

2. **Phase 1 (Audio capture resampling) -- HIGH RISK.** This is the most complex phase, involves real-time audio, has two contradictory implementation paths, doesn't account for all affected files (`audio/mod.rs`, `audio/vad.rs`), and doesn't specify a resampling algorithm or crate. An agent without deep audio programming knowledge will likely produce broken resampling. **Probability of agent getting stuck: ~60%.**

3. **Phase 17 (Ring buffer) -- HIGH RISK of regression.** Replacing `mpsc::channel` with `ringbuf::HeapRb` changes the consumer from a blocking `recv()` to a polling loop. The entire audio pipeline thread in `audio/mod.rs` (lines 42-76) is built around `mpsc` semantics (`recv_timeout`, `Disconnected`). The plan only mentions `capture.rs` and `Cargo.toml` but the actual consumer is in `audio/mod.rs`. Furthermore, the `AudioPipeline::start()` method uses `mpsc` for both the raw audio channel AND the init synchronization channel. The plan conflates these. **Probability of agent getting stuck: ~50%.**

4. **Phase 6 (CSP) -- MEDIUM RISK.** A wrong CSP silently breaks IPC, model downloads, or clipboard access. The only verification is manual DevTools inspection. If the CSP is wrong, the app appears to load but nothing works. **Probability of silent regression: ~30%.**

5. **Phase 4 (Thread join on toggle) -- MEDIUM RISK.** Joining a thread on the main/IPC thread can cause UI freezes if the transcription thread is stuck in a long `engine.transcribe()` call. The plan says "it should already be finishing since `pipeline.stop()` was called" but `pipeline.stop()` only sets `is_running` to false in the AudioPipeline thread -- it does NOT cause the transcription thread's `receiver.recv()` to return immediately. The receiver only returns `Err` when ALL senders are dropped. The sender is held by the audio pipeline thread, which may take up to 100ms (recv_timeout) to notice the stop flag, then drop. During that 100ms, the transcription thread could receive another chunk and start a multi-second inference. **The join could block for seconds.** The plan should use a timeout on the join or send a sentinel value.

### 4.2 Audio Callback Hot Path (Phases 1 and 17)

Phase 1 might add resampling in the audio callback (option 5). Resampling involves floating-point multiplication and buffer management. If done naively (allocating a `Vec` for the resampled output), this reintroduces the exact heap allocation that Phase 17 tries to eliminate. The plan doesn't address this contradiction.

Phase 17 replaces `to_vec()` (heap allocation) with `push_slice()` into a ring buffer (no allocation). This is correct. But if Phase 1 already changed the callback to do resampling with a temporary buffer, Phase 17's ring buffer approach needs to accommodate the resampled data, not the raw data.

### 4.3 Agent "Gets Stuck" Probability

| Phase | Stuck Probability | Reason |
|-------|-------------------|--------|
| 1 | **HIGH** | Ambiguous instructions, requires audio domain knowledge, multiple files affected |
| 2 | LOW | Clear instructions with code snippets |
| 3 | LOW | Nearly identical to Phase 2 |
| 4 | MEDIUM | Thread synchronization is subtle; join-blocking risk |
| 5 | LOW | Clear instructions |
| 6 | LOW | Trivial edit, but testing is hard |
| 7 | TRIVIAL | One-line change |
| 8 | TRIVIAL | One-line change |
| 9 | **HIGH** | Instructions are factually wrong |
| 10 | TRIVIAL | One-line change |
| 11 | LOW | Simple import change, but incomplete scope |
| 12 | LOW | Clear instructions |
| 13 | LOW | File deletion, but needs judgment on hotkey module |
| 14 | LOW | Method deletion + dependency removal |
| 15 | LOW | File deletion + state removal |
| 16 | MEDIUM | Requires dependency analysis of radix-ui vs @radix-ui/* |
| 17 | **HIGH** | Architecture change to real-time audio pipeline |
| 18 | LOW | Straightforward LazyLock conversion |

---

## 5. SUMMARY OF BLOCKING ISSUES

These must be resolved before execution:

1. **Phase 9 is factually wrong.** `std::env::set_var` is unsafe on modern Rust (>= 1.83). Remove this phase or rewrite it to add a `// SAFETY:` comment instead.

2. **Phase 1 gives two contradictory options with no decision.** Pick one (option 5, resample in callback, is architecturally cleaner). List ALL affected files including `audio/mod.rs` line 54 (buffer sample rate).

3. **Phase 1 does not specify a resampling approach.** Either add a crate (e.g., `rubato`) to Cargo.toml or provide the algorithm. "Simple linear interpolation" is not implementable by a non-audio-expert agent.

4. **Phase 4 does not update `stop_dictation()`.** This leaves a thread-leaking code path via the IPC command.

5. **Phase 11 misses `model_manager/download.rs`.** The inconsistent import exists in TWO files, not one.

6. **Phase 1 and Phase 17 conflict.** Both restructure the audio callback. Merge them or make Phase 17 explicitly account for Phase 1's changes.

7. **Phase 4 join-blocking risk.** The thread join can block the IPC handler for seconds. Use `join` with a timeout or restructure to avoid blocking the main thread.

## 6. NON-BLOCKING ISSUES

These should be fixed but won't prevent execution:

1. Phase 6 CSP should be tested for `navigator.clipboard` compatibility (used in main-window.tsx line 86).
2. Phase 13 should definitively state the entire `hotkey/` directory should be removed (not "consider whether").
3. Phase 16 references `@radix-ui/*` packages but the actual dependency is the unified `radix-ui` package.
4. Line number references in Phases 3, 12, and others will be stale after earlier phases modify the same files.
5. Post-fix validation step 8 needs explicit instructions for how to set an invalid output mode.
6. OPT-2 (Mutex held during inference) should appear in a "Deferred" section for completeness.
7. Phase 12 should note that `toggle` is still returned from the `useDictation` hook and consumed nowhere -- the hook's return type should remove it too, or keep it for future use (the plan only removes `_toggle` from the destructure in main-window.tsx but the hook still exports it).
