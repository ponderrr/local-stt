# WhisperType Test Report

Generated: 2026-02-14

## Summary

| Metric | Count |
|--------|-------|
| Total Tests | 137 |
| Passed | 137 |
| Failed | 0 |
| Backend (Rust) | 103 |
| Frontend (TypeScript) | 34 |

## Backend Test Coverage (103 tests)

### Audio Buffer (`audio/buffer.rs`) — 20 tests
- Construction and initialization (sizes, zero-fill)
- Write operations (advance, store, empty, wrap-around, batches)
- has_chunk logic (empty, below threshold, after enough samples, after extraction, after overlap)
- extract_chunk (none when empty, correct size, correct data, resets counter, preserves write_pos)
- Overlap content verification
- Multiple consecutive extractions
- Edge cases (exact boundary, single sample writes)

### Audio Resampling (`audio/mod.rs`) — 12 tests
- to_mono: passthrough single channel, stereo averaging, multichannel, empty, remainder, signal shape preservation
- resample: same-rate passthrough, empty input, 48kHz→16kHz ratio, 44.1kHz→16kHz ratio, DC offset preservation, low-frequency signal fidelity, upsampling, single sample edge case, output range validation

### Voice Activity Detection (`audio/vad.rs`) — 20 tests
- RMS energy: silence, empty slice, constant signal, sine wave, negative values, single sample
- process_frame: silence detection, single frame, consecutive frames, speech detection, consecutive speech frames required, persistence after detection
- Speech-to-silence transition: min consecutive frames, speech frame resets counter
- Threshold boundary: below threshold, just above threshold
- contains_speech: bulk silence, bulk speech, partial speech, non-multiple frame size
- Threshold variation: high threshold ignores quiet, zero threshold detects any signal

### Config/Settings (`config/settings.rs`) — 17 tests
- Default config: all fields validated
- Serialization: roundtrip, all fields, OutputMode snake_case, deserialization, invalid mode, null/present audio_device, missing field
- Path construction: app_dir, models_dir, config_path relative to home
- File system: save/load roundtrip (temp dir), corrupted file, empty file
- OutputMode equality

### Model Registry (`transcription/models.rs`) — 13 tests
- Registry contents: non-empty, 5 models, all expected IDs
- Uniqueness: IDs, filenames
- Validation: valid URLs, nonzero sizes, display names, .bin filenames
- Size ordering: models increase in size from tiny→large-v3
- Static consistency: two calls return same data
- Find: by ID, nonexistent returns none

### Model Manager (`model_manager/download.rs`) — 5 tests
- is_model_downloaded: nonexistent ID returns false, valid ID (best-effort)
- delete_model: nonexistent ID returns error, valid ID no file OK, creates and deletes file

### Output Routing (`output/mod.rs`) — 7 tests
- Mode routing: TypeIntoField, Clipboard, Both (all complete without panic)
- Exhaustive match arms
- Edge cases: empty string, unicode, multiline

### Existing Tests (pre-Wave 2) — 9 tests
- 2 buffer tests, 2 VAD tests, 2 config tests, 1 model registry test, 2 additional

## Frontend Test Coverage (34 tests)

### Tauri IPC Commands (`lib/tauri.test.ts`) — 19 tests
- All 12 commands verify correct invoke call and command name
- Parameter passing verified for: downloadModel, deleteModel, loadModel, updateConfig
- Return value verification for: toggleDictation, listModels, getActiveModel, getConfig, listAudioDevices, getGpuInfo
- Null return handling for getActiveModel

### Tauri Events (`lib/tauri.test.ts`) — 6 tests
- All 5 event listeners verify correct event name strings
- Unlisten function return type verified

### useDictation Hook (`hooks/use-dictation.test.ts`) — 8 tests
- Initial state: idle status, null error
- Status updates via dictation-status event
- Toggle calls toggleDictation command
- Error status on toggle failure
- Error state from output-error event
- Error state from transcription-error event
- 5-second auto-clear of errors
- Error timeout reset on new error

### useModels Hook (`hooks/use-models.test.ts`) — 7 tests
- Loading state on mount
- Model list loaded after mount
- loadModel invokes command and updates activeModel
- downloadModel invokes command and refreshes
- deleteModel invokes command and refreshes
- Load failure handling (activeModel not updated)
- List failure handling (graceful degradation)

## Test Gaps

| Area | Reason |
|------|--------|
| AudioCapture (cpal integration) | Requires real audio device; cannot run in CI |
| AudioPipeline (full pipeline) | Requires cpal + ringbuf integration; hardware-dependent |
| TranscriptionEngine (whisper-rs) | Requires CUDA GPU and downloaded model files |
| Output keyboard/clipboard | Requires display server (X11/Wayland); headless tests verify routing only |
| Frontend components (TSX) | Would require more extensive @testing-library/react setup; hooks cover core logic |
| Tauri event payload extraction | Events are mocked; full integration requires Tauri runtime |
| Download with real HTTP | Would require network/mock HTTP server; tested path construction only |

## How to Run

```bash
# Backend tests
cd backend && cargo test

# Frontend tests
npm test

# Frontend tests in watch mode
npm run test:watch
```
