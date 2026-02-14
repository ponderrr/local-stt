use std::sync::{Arc, Mutex};
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
}

fn join_transcription_thread(state: &AppState) {
    let handle = state.transcription_thread.lock().unwrap().take();
    if let Some(h) = handle {
        h.join().ok();
    }
}

/// Core toggle logic, callable from both Tauri commands and the global hotkey handler.
pub fn toggle_dictation_inner(state: &AppState, app: &AppHandle) -> Result<bool, String> {
    if state.pipeline.is_running() {
        state.pipeline.stop();
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

        join_transcription_thread(state);

        let engine = state.engine.clone();
        let app_clone = app.clone();

        let handle = std::thread::spawn(move || {
            while let Ok(chunk) = receiver.recv() {
                match engine.transcribe(&chunk, &language) {
                    Ok(segments) => {
                        for segment in &segments {
                            if let Err(e) = output::output_text(&segment.text, &output_mode) {
                                eprintln!("Output error: {}", e);
                                app_clone.emit("output-error", format!("Failed to output text: {}", e)).ok();
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
                        eprintln!("Transcription error: {}", e);
                        app_clone.emit("transcription-error", format!("Transcription failed: {}", e)).ok();
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
        state.pipeline.stop();
        join_transcription_thread(&state);
        app.emit("dictation-status", "idle").ok();
    }
    Ok(())
}
