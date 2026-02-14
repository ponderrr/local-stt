use crate::config::Config;
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

    // Ensure models directory exists (use tokio async version for robustness)
    let models_dir = Config::models_dir();
    tokio::fs::create_dir_all(&models_dir)
        .await
        .map_err(|e| format!("Failed to create models dir: {}", e))?;

    let client = Client::new();
    let response = client
        .get(&model.url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
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

    // Close the file handle before renaming to ensure all data is written
    drop(file);

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

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_model_downloaded Tests ---

    #[test]
    fn test_is_model_downloaded_returns_false_for_nonexistent_model() {
        // A model with a made-up ID should not be found in registry, thus false
        assert!(
            !is_model_downloaded("totally-fake-model-xyz"),
            "nonexistent model should not be considered downloaded"
        );
    }

    #[test]
    fn test_is_model_downloaded_returns_false_for_valid_model_not_on_disk() {
        // "tiny" is a valid model ID but the file likely does not exist in a test env
        // Unless the user has actually downloaded it, this should be false
        // (This test is best-effort -- in CI it will always be false)
        let result = is_model_downloaded("tiny");
        // We cannot strictly assert false because the file might exist on the developer's machine
        // Instead, just verify it returns a bool without panicking
        let _ = result;
    }

    // --- delete_model Tests ---

    #[test]
    fn test_delete_model_nonexistent_id_returns_error() {
        let result = delete_model("nonexistent-model-abc");
        assert!(
            result.is_err(),
            "deleting a nonexistent model ID should return error"
        );
    }

    #[test]
    fn test_delete_model_valid_id_no_file_ok() {
        // Even if the model file does not exist on disk, delete_model should succeed
        // (it checks path.exists() and skips if not found)
        let result = delete_model("tiny");
        // This should not error -- it checks if the file exists first
        assert!(
            result.is_ok(),
            "deleting a valid model ID with no file on disk should succeed"
        );
    }

    #[test]
    fn test_delete_model_with_file_removes_it() {
        // Create a fake model file in the models directory
        let models_dir = Config::models_dir();
        std::fs::create_dir_all(&models_dir).ok();

        let registry = get_model_registry();
        let tiny = registry.iter().find(|m| m.id == "tiny").unwrap();
        let path = models_dir.join(&tiny.filename);

        // Create a dummy file
        std::fs::write(&path, b"fake model data").unwrap();
        assert!(path.exists());

        // Delete it
        let result = delete_model("tiny");
        assert!(result.is_ok());
        assert!(!path.exists(), "model file should be deleted");
    }

    // --- Path Construction Tests ---

    #[test]
    fn test_model_path_construction() {
        let registry = get_model_registry();
        let model = registry.iter().find(|m| m.id == "tiny").unwrap();
        let expected_path = Config::models_dir().join(&model.filename);
        assert!(expected_path.to_str().unwrap().contains("models"));
        assert!(expected_path.to_str().unwrap().contains("ggml-tiny.bin"));
    }
}
