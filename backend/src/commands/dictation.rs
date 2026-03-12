//! Core dictation commands: toggle, start, and stop. Manages the audio pipeline
//! and transcription thread lifecycle via shared AppState.

use ringbuf::traits::Split;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, State};

use crate::audio::{AudioMessage, AudioPipeline};
use crate::config::{Config, StreamEngineConfig};
use crate::output;
use crate::transcription::agreement::LocalAgreement;
use crate::transcription::engine::TranscriptionEngine;
use crate::transcription::moonshine::MoonshineEngine;

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
            config.vad_backend.clone(),
            device_rate,
            device_channels,
        )?;

        let language = config.language.clone();
        let output_mode = config.output_mode.clone();
        let stream_engine_config = config.stream_engine.clone();
        drop(config);
        drop(handle_lock);

        // Load Moonshine BEFORE starting the audio pipeline so that all ORT
        // sessions (Moonshine + Silero VAD) are created sequentially rather
        // than concurrently.  Concurrent session creation against the same
        // load-dynamic ORT environment causes "GetElementType is not
        // implemented" crashes in Silero VAD inference.
        let moonshine = if stream_engine_config == StreamEngineConfig::Moonshine {
            let moonshine_model_dir = Config::models_dir().join("moonshine-tiny");
            match MoonshineEngine::load(&moonshine_model_dir) {
                Ok(engine) => {
                    eprintln!("moonshine: engine loaded for streaming display");
                    Some(engine)
                }
                Err(e) => {
                    eprintln!(
                        "moonshine: failed to load ({}), falling back to whisper-only",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

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

            let mut moonshine = moonshine;

            let mut audio_buf: Vec<f32> = Vec::new();
            let mut agreement = LocalAgreement::new();
            let min_samples: usize = 16000; // 1.0s minimum to avoid hallucination
            let max_samples: usize = 16000 * 30; // 30s maximum buffer
            let mut has_output = false; // tracks whether we've typed anything this utterance

            while let Ok(msg) = receiver.recv() {
                let mut got_end = false;

                match msg {
                    AudioMessage::Segment(seg) => audio_buf.extend_from_slice(&seg),
                    AudioMessage::EndOfSpeech => got_end = true,
                }

                // Drain all queued messages to get latest audio state
                while let Ok(m) = receiver.try_recv() {
                    match m {
                        AudioMessage::Segment(seg) => audio_buf.extend_from_slice(&seg),
                        AudioMessage::EndOfSpeech => got_end = true,
                    }
                }

                // Cap buffer at maximum window
                if audio_buf.len() > max_samples {
                    let excess = audio_buf.len() - max_samples;
                    audio_buf.drain(..excess);
                }

                // Skip inference if not enough audio and not end of speech
                if audio_buf.len() < min_samples {
                    if got_end {
                        audio_buf.clear();
                        agreement.reset();
                        has_output = false;
                    }
                    continue;
                }

                // === Dual-path inference ===
                if got_end {
                    // --- EndOfSpeech: quality pass ---
                    // One last Moonshine stream pass through agreement (dual-path only)
                    if let Some(ref mut ms) = moonshine {
                        if let Ok(ms_text) = ms.transcribe(&audio_buf) {
                            if !ms_text.is_empty() {
                                let result = agreement.process(&ms_text);
                                if !result.newly_confirmed.is_empty() {
                                    let output = if has_output {
                                        format!(" {}", result.newly_confirmed)
                                    } else {
                                        result.newly_confirmed.clone()
                                    };
                                    if let Err(e) = output::output_text(&output, &output_mode) {
                                        app_clone
                                            .emit("output-error", format!("Output error: {}", e))
                                            .ok();
                                    }
                                    has_output = true;
                                    app_clone
                                        .emit(
                                            "transcription-update",
                                            serde_json::json!({
                                                "text": result.newly_confirmed,
                                                "is_partial": false,
                                            }),
                                        )
                                        .ok();
                                }
                            }
                        }
                    }

                    // Whisper quality pass on full utterance
                    match engine.transcribe(&mut whisper_state, &audio_buf, &language) {
                        Ok(segments) => {
                            let text: String = segments
                                .iter()
                                .map(|s| s.text.as_str())
                                .collect::<Vec<_>>()
                                .join(" ");
                            let text = text.trim().to_string();

                            // In whisper-only mode, process through agreement (v0.2.0 behavior)
                            if moonshine.is_none() && !text.is_empty() {
                                let result = agreement.process(&text);
                                if !result.newly_confirmed.is_empty() {
                                    let output = if has_output {
                                        format!(" {}", result.newly_confirmed)
                                    } else {
                                        result.newly_confirmed.clone()
                                    };
                                    if let Err(e) = output::output_text(&output, &output_mode) {
                                        app_clone
                                            .emit("output-error", format!("Output error: {}", e))
                                            .ok();
                                    }
                                    has_output = true;
                                    app_clone
                                        .emit(
                                            "transcription-update",
                                            serde_json::json!({
                                                "text": result.newly_confirmed,
                                                "is_partial": false,
                                            }),
                                        )
                                        .ok();
                                }
                                // Update tentative display (v0.2.0 behavior)
                                app_clone
                                    .emit(
                                        "transcription-update",
                                        serde_json::json!({
                                            "text": result.tentative,
                                            "is_partial": true,
                                        }),
                                    )
                                    .ok();
                            }

                            // Finalize: confirm remaining tentative words
                            let remaining = agreement.finalize();
                            if !remaining.is_empty() {
                                let output = if has_output {
                                    format!(" {}", remaining)
                                } else {
                                    remaining.clone()
                                };
                                if let Err(e) = output::output_text(&output, &output_mode) {
                                    app_clone
                                        .emit("output-error", format!("Output error: {}", e))
                                        .ok();
                                }
                                app_clone
                                    .emit(
                                        "transcription-update",
                                        serde_json::json!({
                                            "text": remaining,
                                            "is_partial": false,
                                        }),
                                    )
                                    .ok();
                            }
                        }
                        Err(e) => {
                            // Still finalize agreement even if Whisper fails
                            let remaining = agreement.finalize();
                            if !remaining.is_empty() {
                                let output = if has_output {
                                    format!(" {}", remaining)
                                } else {
                                    remaining.clone()
                                };
                                let _ = output::output_text(&output, &output_mode);
                                app_clone
                                    .emit(
                                        "transcription-update",
                                        serde_json::json!({
                                            "text": remaining,
                                            "is_partial": false,
                                        }),
                                    )
                                    .ok();
                            }
                            app_clone
                                .emit(
                                    "transcription-error",
                                    format!("Whisper quality pass failed: {}", e),
                                )
                                .ok();
                        }
                    }

                    // Clear tentative display and reset for next utterance
                    app_clone
                        .emit(
                            "transcription-update",
                            serde_json::json!({ "text": "", "is_partial": true }),
                        )
                        .ok();
                    audio_buf.clear();
                    has_output = false;
                } else {
                    // --- During speech: stream pass ---
                    let text = if let Some(ref mut ms) = moonshine {
                        ms.transcribe(&audio_buf).unwrap_or_default()
                    } else {
                        match engine.transcribe(&mut whisper_state, &audio_buf, &language) {
                            Ok(segments) => segments
                                .iter()
                                .map(|s| s.text.as_str())
                                .collect::<Vec<_>>()
                                .join(" ")
                                .trim()
                                .to_string(),
                            Err(e) => {
                                app_clone
                                    .emit(
                                        "transcription-error",
                                        format!("Transcription failed: {}", e),
                                    )
                                    .ok();
                                String::new()
                            }
                        }
                    };

                    if !text.is_empty() {
                        let result = agreement.process(&text);
                        if !result.newly_confirmed.is_empty() {
                            let output = if has_output {
                                format!(" {}", result.newly_confirmed)
                            } else {
                                result.newly_confirmed.clone()
                            };
                            if let Err(e) = output::output_text(&output, &output_mode) {
                                app_clone
                                    .emit("output-error", format!("Output error: {}", e))
                                    .ok();
                            }
                            has_output = true;
                            app_clone
                                .emit(
                                    "transcription-update",
                                    serde_json::json!({
                                        "text": result.newly_confirmed,
                                        "is_partial": false,
                                    }),
                                )
                                .ok();
                        }
                        app_clone
                            .emit(
                                "transcription-update",
                                serde_json::json!({
                                    "text": result.tentative,
                                    "is_partial": true,
                                }),
                            )
                            .ok();
                    }
                }
            }

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

    #[test]
    fn test_stream_engine_config_default_is_whisper_only() {
        let config = Config::default();
        assert_eq!(
            config.stream_engine,
            crate::config::StreamEngineConfig::WhisperOnly
        );
    }
}
