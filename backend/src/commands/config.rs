use tauri::State;

use crate::commands::dictation::AppState;
use crate::config::Config;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn update_config(config: Config, state: State<'_, AppState>) -> Result<(), String> {
    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config;
    current.save()?;
    Ok(())
}
