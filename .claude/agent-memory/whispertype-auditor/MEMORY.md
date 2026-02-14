# WhisperType Auditor Memory

## Project Layout
- Backend Rust: `backend/src/` with `lib.rs` as entry, `main.rs` as binary
- Frontend React: `frontend/src/` with `main.tsx` as entry, `App.tsx` as root component
- Config: `backend/tauri.conf.json`, `backend/capabilities/default.json`
- Build: `vite.config.ts` at root, `tsconfig.json` at root
- Models dir at runtime: `~/.whispertype/models/`

## Key Architecture Observations
- whisper-rs 0.15.1 segment API: `get_segment(c_int) -> Option<WhisperSegment>`, `to_str() -> Result<&str, WhisperError>`
- Serde OutputMode: `#[serde(rename_all = "snake_case")]` maps TypeIntoField -> "type_into_field" (matches TS)
- Config struct has NO rename_all, fields stay snake_case (matches TS interface)
- Tauri v2 IPC camelCase auto-conversion applies to command params (e.g., `modelId` in JS -> `model_id` in Rust)
- AudioCapture creates cpal stream requesting 16kHz mono f32 - may fail on devices not supporting this config
- Ring buffer write_pos increments without bound but uses modulo - safe for ~36B years at 16kHz
- Capabilities file only has `core:default` and `opener:default` - missing `global-shortcut:default`

## Critical Bugs Found
1. Missing `global-shortcut` permission in capabilities (may prevent hotkey from working)
2. AudioCapture hardcodes 16kHz - cpal may reject this if device doesn't support it natively
3. AudioPipeline::stop() only sets flag but doesn't clean up cpal stream synchronously
4. Transcription thread not tracked/joined - potential orphan thread on rapid toggle
5. No model auto-load at startup (setup wizard downloads but doesn't load)
6. CSP is null in tauri.conf.json (security concern)

## Dead Code
- `model_manager/storage.rs` - TODO stub, never called
- `hotkey/manager.rs` - empty, hotkey handled in lib.rs
- `pages/setup.tsx` - unused, replaced by setup-wizard component
- All shadcn UI components (button, card, select, dialog, etc.) - installed but unused
