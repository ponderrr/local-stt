use crate::config::settings::Config;
use crate::transcription::models::get_model_registry;
use futures_util::StreamExt;
use reqwest::Client;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

pub async fn download_model(model_id: &str, app_handle: &AppHandle) -> Result<PathBuf, String> {
    let registry = get_model_registry();
    let model = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let dest = Config::models_dir().join(&model.filename);

    // Skip if already downloaded
    if dest.exists() {
        let metadata = std::fs::metadata(&dest).map_err(|e| e.to_string())?;
        if metadata.len() > 0 {
            return Ok(dest);
        }
    }

    // Ensure models directory exists
    Config::ensure_dirs().map_err(|e| format!("Failed to ensure dirs: {}", e))?;

    let client = Client::new();
    let response = client
        .get(&model.url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let total = response.content_length().unwrap_or(model.size_bytes);
    let mut downloaded: u64 = 0;

    // Write to temp file first, rename on completion (atomic write)
    let temp_path = dest.with_extension("bin.tmp");
    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download stream error: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write chunk: {}", e))?;

        downloaded += chunk.len() as u64;

        app_handle
            .emit(
                "download-progress",
                serde_json::json!({
                    "model_id": model_id,
                    "percent": (downloaded as f64 / total as f64) * 100.0,
                    "downloaded_bytes": downloaded,
                    "total_bytes": total,
                }),
            )
            .ok();
    }

    file.flush()
        .await
        .map_err(|e| format!("Failed to flush file: {}", e))?;

    // Atomic rename: temp -> final destination
    tokio::fs::rename(&temp_path, &dest)
        .await
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    Ok(dest)
}

pub fn delete_model(model_id: &str) -> Result<(), String> {
    let registry = get_model_registry();
    let model = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let path = Config::models_dir().join(&model.filename);
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete model: {}", e))?;
    }
    Ok(())
}

pub fn is_model_downloaded(model_id: &str) -> bool {
    let registry = get_model_registry();
    registry
        .iter()
        .find(|m| m.id == model_id)
        .map(|m| Config::models_dir().join(&m.filename).exists())
        .unwrap_or(false)
}
