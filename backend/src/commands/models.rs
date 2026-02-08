use tauri::{AppHandle, Emitter, Manager, State};

use crate::commands::dictation::AppState;
use crate::config::Config;
use crate::model_manager;
use crate::transcription::{get_model_registry, WhisperModel};

#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub vram_mb: u16,
    pub downloaded: bool,
}

impl From<&WhisperModel> for ModelInfo {
    fn from(m: &WhisperModel) -> Self {
        Self {
            id: m.id.clone(),
            display_name: m.display_name.clone(),
            filename: m.filename.clone(),
            size_bytes: m.size_bytes,
            vram_mb: m.vram_mb,
            downloaded: model_manager::is_model_downloaded(&m.id),
        }
    }
}

#[tauri::command]
pub fn list_models() -> Vec<ModelInfo> {
    get_model_registry().iter().map(ModelInfo::from).collect()
}

#[tauri::command]
pub async fn download_model(model_id: String, app: AppHandle) -> Result<(), String> {
    model_manager::download_model(&model_id, &app).await?;

    let state = app.state::<AppState>();
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if !config.downloaded_models.contains(&model_id) {
        config.downloaded_models.push(model_id);
        config.save()?;
    }

    Ok(())
}

#[tauri::command]
pub fn delete_model(model_id: String, state: State<'_, AppState>) -> Result<(), String> {
    if state.engine.get_active_model().as_deref() == Some(model_id.as_str()) {
        state.engine.unload_model()?;
    }
    model_manager::delete_model(&model_id)?;

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.downloaded_models.retain(|m| m != &model_id);
    config.save()?;

    Ok(())
}

#[tauri::command]
pub async fn load_model(
    model_id: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    app.emit("dictation-status", "loading").ok();

    let registry = get_model_registry();
    let model = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let model_path = Config::models_dir().join(&model.filename);
    if !model_path.exists() {
        return Err(format!("Model not downloaded: {}", model_id));
    }

    let engine = state.engine.clone();
    let mid = model_id.clone();

    tokio::task::spawn_blocking(move || engine.load_model(&model_path, &mid))
        .await
        .map_err(|e| e.to_string())??;

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.default_model = model_id;
    config.save()?;

    app.emit("dictation-status", "idle").ok();
    Ok(())
}

#[tauri::command]
pub fn get_active_model(state: State<'_, AppState>) -> Option<String> {
    state.engine.get_active_model()
}
