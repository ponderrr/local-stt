# Changelog

All notable changes to WhisperType will be documented in this file.

## [0.3.0] - 2026-03-11

### Added
- **Moonshine ONNX engine** — CPU-only speech-to-text via transcribe-rs. Runs the Moonshine model family (Tiny ~28 MB, Base ~63 MB) through ONNX Runtime for instant streaming previews without GPU.
- **Dual-path transcription pipeline** — Moonshine provides real-time streaming previews during speech while Whisper produces the final high-accuracy transcription on EndOfSpeech. Configurable via Settings.
- **Stream Engine selector** — New setting to choose between Whisper-only mode (v0.2.0 behavior) and Moonshine + Whisper dual-path mode.
- **ModelType enum** — Distinguishes Whisper GGML models from Moonshine ONNX models in the registry. Prevents ONNX directories from being loaded through whisper.cpp.
- **Multi-file model downloads** — Model download manager supports Moonshine models that consist of multiple files (encoder, decoder, tokenizer).
- **Moonshine benchmark binary** — `moonshine_bench` binary for profiling Moonshine inference performance.

### Fixed
- **ORT session race condition** — Concurrent ONNX Runtime session creation between Silero VAD and Moonshine caused `GetElementType is not implemented` errors. Moonshine is now loaded sequentially before the audio pipeline starts.
- **ORT environment initialization** — Added eager `ort::init().commit()` at app startup to prevent environment initialization races between independent ORT consumers.
- **Moonshine model loading through Whisper engine** — Loading a Moonshine model ID previously passed the ONNX directory path to whisper.cpp, causing `invalid model data (bad magic)` errors. Moonshine models now skip the Whisper load path entirely.

### Changed
- Transcription thread receives a pre-loaded Moonshine engine instance instead of loading it inline, ensuring all ORT sessions are created on the same thread sequentially.
- Model registry expanded from 7 to 9 entries (7 Whisper GGML + 2 Moonshine ONNX).

### Dependencies
- Added `transcribe-rs` 0.2.9 with `moonshine` feature for Moonshine ONNX inference.
- `ort` 2.0.0-rc.10 shared between Silero VAD and transcribe-rs (zero conflicts via `load-dynamic`).

## [0.2.0] - 2026-03-07

### Added
- **distil-large-v3 model** — 6x faster than large-v3 with <1% WER difference on English. Now the default model.
- **large-v3-turbo model** — OpenAI's multilingual distilled variant.
- **Silero VAD** — Neural network voice activity detection replacing RMS energy threshold. Trained on 6000+ languages, 0.97 ROC-AUC. Runs at <1ms per frame on CPU.
- **Continuous inference** — Removed fixed 2-second chunk accumulation. Whisper now runs back-to-back on a growing audio buffer, producing ~14 inference passes per second during speech.
- **LocalAgreement-2 deduplication** — Words that appear at the same position in two consecutive inference passes are confirmed and locked. Confirmed text appears incrementally in the target app during speech.
- **Live preview** — Tentative (unconfirmed) text displays in gray italic in the WhisperType overlay, updating in real-time as you speak.
- **VAD backend selector** — Choose between Silero AI detection and energy-based detection in Settings.

### Changed
- Default model changed from large-v3 to distil-large-v3.
- Audio pipeline rewritten: DSP thread sends ~100ms segments instead of 2000ms chunks.
- Transcription thread now accumulates audio and runs continuous inference with 1.0s minimum / 30s maximum buffer.
- Keyboard/clipboard output fires incrementally as words are confirmed, not in bulk at utterance end.
- Frontend transcript display splits committed (white) and tentative (gray italic) text.

### Performance
- Per-inference latency: ~70ms (distil-large-v3) vs ~300ms (large-v3)
- Perceived word appearance: ~200ms after speaking (vs ~2000ms in v0.1.0)
- VRAM usage: ~2.0GB (distil-large-v3) vs ~3.1GB (large-v3)
- VAD: 32ms granularity (Silero) vs 2000ms (energy on chunk)

## [0.1.0] - 2026-02-24

### Added
- Initial release: local speech-to-text with Whisper large-v3
- Three-thread pipeline: PulseAudio capture → DSP → Whisper transcription
- Energy-based VAD with hysteresis
- Keyboard simulation (enigo) and clipboard output (arboard)
- Model download manager with progress streaming
- Tauri 2 desktop app with React frontend
- Global hotkey (Ctrl+Shift+Space)
- CUDA acceleration via whisper.cpp
