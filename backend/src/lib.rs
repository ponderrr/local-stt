pub mod audio;
pub mod commands;
pub mod config;
pub mod hotkey;
pub mod model_manager;
pub mod output;
pub mod transcription;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::system::list_audio_devices,
            commands::system::get_gpu_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
