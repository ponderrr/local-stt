//! WhisperType — local AI-powered speech-to-text desktop app built on Tauri v2.

pub mod audio;
pub mod commands;
pub mod config;
pub mod model_manager;
pub mod output;
pub mod transcription;

use std::sync::{Arc, Mutex};

use commands::dictation::AppState;
use config::Config;
use transcription::engine::TranscriptionEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Config::load().unwrap_or_default();

    let app_state = AppState {
        engine: Arc::new(TranscriptionEngine::new()),
        pipeline: audio::AudioPipeline::new(),
        config: Mutex::new(config),
        transcription_thread: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    use tauri::Manager;
                    use tauri_plugin_global_shortcut::ShortcutState;

                    if event.state() == ShortcutState::Pressed {
                        let state = app.state::<AppState>();
                        let _ = commands::dictation::toggle_dictation_inner(&state, app);
                    }
                })
                .build(),
        )
        .manage(app_state)
        .setup(|app| {
            use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space);
            app.global_shortcut().register(shortcut)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::dictation::toggle_dictation,
            commands::dictation::start_dictation,
            commands::dictation::stop_dictation,
            commands::models::list_models,
            commands::models::download_model,
            commands::models::delete_model,
            commands::models::load_model,
            commands::models::get_active_model,
            commands::config::get_config,
            commands::config::update_config,
            commands::system::list_audio_devices,
            commands::system::get_gpu_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
