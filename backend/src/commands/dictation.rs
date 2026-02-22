//! Core dictation commands: toggle, start, and stop. Manages the audio pipeline
//! and transcription thread lifecycle via shared AppState.

use ringbuf::traits::Split;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};

use crate::audio::AudioPipeline;
use crate::config::Config;
use crate::output;
use crate::transcription::engine::TranscriptionEngine;

pub struct AppState {
    pub engine: Arc<TranscriptionEngine>,
    pub pipeline: AudioPipeline,
    pub config: Mutex<Config>,
    pub transcription_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    pub audio_handle: Mutex<Option<crate::audio::capture::AudioHandle>>,
    pub last_shortcut: Mutex<Option<Instant>>,
}

fn join_transcription_thread(state: &AppState) {
    let handle = state.transcription_thread.lock().unwrap().take();
    if let Some(h) = handle {
        h.join().ok();
    }
    // Send Quit to terminate the pulse-actor thread, then drop the handle
    let mut audio_lock = state.audio_handle.lock().unwrap();
    if let Some(h) = audio_lock.as_ref() {
        let _ = h.cmd_tx.send(crate::audio::capture::AudioCommand::Quit);
    }
    audio_lock.take();
}

/// Core toggle logic, callable from both Tauri commands and the global hotkey handler.
pub fn toggle_dictation_inner(state: &AppState, app: &AppHandle) -> Result<bool, String> {
    if state.pipeline.is_running() {
        state.pipeline.stop();

        if let Some(handle) = state.audio_handle.lock().unwrap().as_ref() {
            let _ = handle
                .cmd_tx
                .send(crate::audio::capture::AudioCommand::Stop);
        }

        join_transcription_thread(state);
        app.emit("dictation-status", "idle").ok();
        Ok(false)
    } else {
        if !state.engine.is_loaded() {
            return Err(
                "No model loaded. Please load a model before starting dictation.".to_string(),
            );
        }

        let config = state.config.lock().map_err(|e| e.to_string())?;

        let mut handle_lock = state.audio_handle.lock().unwrap();

        if handle_lock.is_some() {
            return Ok(true);
        }

        let rb = ringbuf::HeapRb::<f32>::new(48000 * 5);
        let (prod, cons) = rb.split();

        let device_name = config.audio_device.clone();
        let new_handle = crate::audio::capture::AudioCapture::spawn_audio_actor(device_name, prod)?;

        let device_rate = new_handle.sample_rate;
        let device_channels = new_handle.channels;

        *handle_lock = Some(new_handle);

        let receiver = state.pipeline.start(
            Some(cons),
            config.vad_threshold,
            config.chunk_duration_ms,
            config.overlap_ms,
            device_rate,
            device_channels,
        )?;

        let language = config.language.clone();
        let output_mode = config.output_mode.clone();
        drop(config);
        drop(handle_lock);

        app.emit("dictation-status", "listening").ok();

        // Join any previous transcription thread, but do NOT clear the audio
        // handle — that belongs to this session. join_transcription_thread()
        // is only for the stop path where full cleanup is needed.
        if let Some(h) = state.transcription_thread.lock().unwrap().take() {
            h.join().ok();
        }

        let engine = state.engine.clone();
        let app_clone = app.clone();

        let handle = std::thread::spawn(move || {
            // Phase A: Create WhisperState ONCE for this dictation session.
            // Reused across all chunks — eliminates per-chunk CUDA state init (~300-400ms).
            let mut whisper_state = match engine.create_inference_state() {
                Ok(s) => {
                    eprintln!("whisper: inference state created (once per session)");
                    s
                }
                Err(e) => {
                    app_clone
                        .emit(
                            "transcription-error",
                            format!("Failed to create inference state: {}", e),
                        )
                        .ok();
                    app_clone.emit("dictation-status", "idle").ok();
                    return;
                }
            };

            while let Ok(chunk) = receiver.recv() {
                match engine.transcribe(&mut whisper_state, &chunk, &language) {
                    Ok(segments) => {
                        for segment in &segments {
                            if let Err(e) = output::output_text(&segment.text, &output_mode) {
                                app_clone
                                    .emit("output-error", format!("Failed to output text: {}", e))
                                    .ok();
                            }

                            app_clone
                                .emit(
                                    "transcription-update",
                                    serde_json::json!({
                                        "text": segment.text,
                                        "is_partial": false,
                                    }),
                                )
                                .ok();
                        }
                    }
                    Err(e) => {
                        app_clone
                            .emit(
                                "transcription-error",
                                format!("Transcription failed: {}", e),
                            )
                            .ok();
                    }
                }
            }
            // whisper_state dropped here when thread exits, before join returns
            app_clone.emit("dictation-status", "idle").ok();
        });
        *state.transcription_thread.lock().unwrap() = Some(handle);

        Ok(true)
    }
}

#[tauri::command]
pub fn toggle_dictation(state: State<'_, AppState>, app: AppHandle) -> Result<bool, String> {
    toggle_dictation_inner(&state, &app)
}

#[tauri::command]
pub fn start_dictation(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    if !state.pipeline.is_running() {
        toggle_dictation_inner(&state, &app)?;
    }
    Ok(())
}

#[tauri::command]
pub fn stop_dictation(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    if state.pipeline.is_running() {
        if let Some(handle) = state.audio_handle.lock().unwrap().as_ref() {
            let _ = handle
                .cmd_tx
                .send(crate::audio::capture::AudioCommand::Stop);
        }
        state.pipeline.stop();
        join_transcription_thread(&state);
        app.emit("dictation-status", "idle").ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that AppState can be safely initialized and that its internal Locks
    /// do not immediately poison or panic.
    #[test]
    fn test_appstate_initialization_does_not_panic() {
        let config = Config::default();
        let engine = Arc::new(TranscriptionEngine::new());
        let pipeline = AudioPipeline::new();

        let state = AppState {
            engine,
            pipeline,
            config: Mutex::new(config),
            transcription_thread: Mutex::new(None),
            audio_handle: Mutex::new(None),
            last_shortcut: Mutex::new(None),
        };

        assert!(
            state.audio_handle.lock().unwrap().is_none(),
            "Audio handle should start empty"
        );
        assert!(
            state.transcription_thread.lock().unwrap().is_none(),
            "Transcription thread should start empty"
        );
    }

    /// Verifies that join_transcription_thread safely operates on an empty state without panicking.
    #[test]
    fn test_join_transcription_thread_empty_state_safety() {
        let state = AppState {
            engine: Arc::new(TranscriptionEngine::new()),
            pipeline: AudioPipeline::new(),
            config: Mutex::new(Config::default()),
            transcription_thread: Mutex::new(None),
            audio_handle: Mutex::new(None),
            last_shortcut: Mutex::new(None),
        };

        // Should not panic or block
        join_transcription_thread(&state);

        let handle_lock = state.audio_handle.lock().unwrap();
        assert!(
            handle_lock.is_none(),
            "Audio handle must remain empty after join on empty state"
        );
    }
}
