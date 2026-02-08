use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

use crate::audio::AudioPipeline;
use crate::config::settings::Config;
use crate::output;
use crate::transcription::engine::TranscriptionEngine;

pub struct AppState {
    pub engine: Arc<TranscriptionEngine>,
    pub pipeline: AudioPipeline,
    pub config: Mutex<Config>,
}

/// Core toggle logic, callable from both Tauri commands and the global hotkey handler.
pub fn toggle_dictation_inner(state: &AppState, app: &AppHandle) -> Result<bool, String> {
    if state.pipeline.is_running() {
        state.pipeline.stop();
        app.emit("dictation-status", "idle").ok();
        Ok(false)
    } else {
        if !state.engine.is_loaded() {
            return Err(
                "No model loaded. Please load a model before starting dictation.".to_string(),
            );
        }

        let config = state.config.lock().map_err(|e| e.to_string())?;
        let receiver = state.pipeline.start(
            config.audio_device.clone(),
            config.vad_threshold,
            config.chunk_duration_ms,
            config.overlap_ms,
        )?;
        let language = config.language.clone();
        let output_mode = config.output_mode.clone();
        drop(config);

        app.emit("dictation-status", "listening").ok();

        let engine = state.engine.clone();
        let app_clone = app.clone();

        std::thread::spawn(move || {
            while let Ok(chunk) = receiver.recv() {
                match engine.transcribe(&chunk, &language) {
                    Ok(segments) => {
                        for segment in &segments {
                            output::output_text(&segment.text, &output_mode).ok();
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
                        eprintln!("Transcription error: {}", e);
                    }
                }
            }
            app_clone.emit("dictation-status", "idle").ok();
        });

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
        state.pipeline.stop();
        app.emit("dictation-status", "idle").ok();
    }
    Ok(())
}
