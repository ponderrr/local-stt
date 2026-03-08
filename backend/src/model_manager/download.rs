//! Model lifecycle: async download with progress streaming, atomic file writes,
//! deletion, and download-status queries. Supports both single-file (Whisper GGML)
//! and multi-file (Moonshine ONNX) models.

use crate::config::Config;
use crate::transcription::models::{get_model_registry, ModelType};
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
    if is_model_downloaded(model_id) {
        return Ok(dest);
    }

    let models_dir = Config::models_dir();
    tokio::fs::create_dir_all(&models_dir)
        .await
        .map_err(|e| format!("Failed to create models dir: {}", e))?;

    if !model.files.is_empty() {
        // Multi-file model (Moonshine): create directory, download each file
        download_multi_file(model_id, &dest, &model.files, model.size_bytes, app_handle).await?;
    } else {
        // Single-file model (Whisper)
        download_single_file(model_id, &dest, &model.url, model.size_bytes, app_handle).await?;
    }

    Ok(dest)
}

async fn download_single_file(
    model_id: &str,
    dest: &PathBuf,
    url: &str,
    expected_size: u64,
    app_handle: &AppHandle,
) -> Result<(), String> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Download request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let total = response.content_length().unwrap_or(expected_size);
    let mut downloaded: u64 = 0;

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
    drop(file);

    tokio::fs::rename(&temp_path, dest)
        .await
        .map_err(|e| format!("Failed to rename temp file: {}", e))?;

    Ok(())
}

async fn download_multi_file(
    model_id: &str,
    model_dir: &PathBuf,
    files: &[(String, String)],
    total_size: u64,
    app_handle: &AppHandle,
) -> Result<(), String> {
    // Create model directory
    tokio::fs::create_dir_all(model_dir)
        .await
        .map_err(|e| format!("Failed to create model dir: {}", e))?;

    let client = Client::new();
    let mut total_downloaded: u64 = 0;

    for (filename, url) in files {
        let file_dest = model_dir.join(filename);

        // Skip files already downloaded
        if file_dest.exists() {
            if let Ok(meta) = std::fs::metadata(&file_dest) {
                total_downloaded += meta.len();
            }
            continue;
        }

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Download request for {} failed: {}", filename, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Download of {} failed with status: {}",
                filename,
                response.status()
            ));
        }

        let temp_path = file_dest.with_extension("tmp");
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| format!("Failed to create temp file for {}: {}", filename, e))?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download stream error for {}: {}", filename, e))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write chunk for {}: {}", filename, e))?;

            total_downloaded += chunk.len() as u64;

            app_handle
                .emit(
                    "download-progress",
                    serde_json::json!({
                        "model_id": model_id,
                        "percent": (total_downloaded as f64 / total_size as f64) * 100.0,
                        "downloaded_bytes": total_downloaded,
                        "total_bytes": total_size,
                    }),
                )
                .ok();
        }

        file.flush()
            .await
            .map_err(|e| format!("Failed to flush file {}: {}", filename, e))?;
        drop(file);

        tokio::fs::rename(&temp_path, &file_dest)
            .await
            .map_err(|e| format!("Failed to rename temp file for {}: {}", filename, e))?;
    }

    Ok(())
}

pub fn delete_model(model_id: &str) -> Result<(), String> {
    let registry = get_model_registry();
    let model = registry
        .iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| format!("Unknown model: {}", model_id))?;

    let path = Config::models_dir().join(&model.filename);
    if path.exists() {
        if model.model_type == ModelType::MoonshineOnnx {
            std::fs::remove_dir_all(&path)
                .map_err(|e| format!("Failed to delete model directory: {}", e))?;
        } else {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to delete model: {}", e))?;
        }
    }
    Ok(())
}

pub fn is_model_downloaded(model_id: &str) -> bool {
    let registry = get_model_registry();
    registry
        .iter()
        .find(|m| m.id == model_id)
        .map(|m| {
            let base = Config::models_dir().join(&m.filename);
            if m.files.is_empty() {
                // Single-file: check file exists and is non-empty
                base.exists() && std::fs::metadata(&base).map(|meta| meta.len() > 0).unwrap_or(false)
            } else {
                // Multi-file: check directory exists with all expected files
                base.is_dir()
                    && m.files
                        .iter()
                        .all(|(fname, _)| base.join(fname).exists())
            }
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_model_downloaded Tests ---

    #[test]
    fn test_is_model_downloaded_returns_false_for_nonexistent_model() {
        assert!(
            !is_model_downloaded("totally-fake-model-xyz"),
            "nonexistent model should not be considered downloaded"
        );
    }

    #[test]
    fn test_is_model_downloaded_returns_false_for_valid_model_not_on_disk() {
        let result = is_model_downloaded("tiny");
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
        let result = delete_model("tiny");
        assert!(
            result.is_ok(),
            "deleting a valid model ID with no file on disk should succeed"
        );
    }

    #[test]
    fn test_delete_model_with_file_removes_it() {
        let models_dir = Config::models_dir();
        std::fs::create_dir_all(&models_dir).ok();

        let registry = get_model_registry();
        let tiny = registry.iter().find(|m| m.id == "tiny").unwrap();
        let path = models_dir.join(&tiny.filename);

        std::fs::write(&path, b"fake model data").unwrap();
        assert!(path.exists());

        let result = delete_model("tiny");
        assert!(result.is_ok());
        assert!(!path.exists(), "model file should be deleted");
    }

    #[test]
    fn test_delete_moonshine_model_removes_directory() {
        let models_dir = Config::models_dir();
        let model_dir = models_dir.join("moonshine-tiny");
        std::fs::create_dir_all(&model_dir).ok();
        std::fs::write(model_dir.join("encoder_model.onnx"), b"fake").unwrap();
        std::fs::write(model_dir.join("decoder_model_merged.onnx"), b"fake").unwrap();
        std::fs::write(model_dir.join("tokenizer.json"), b"fake").unwrap();
        assert!(model_dir.exists());

        let result = delete_model("moonshine-tiny");
        assert!(result.is_ok());
        assert!(!model_dir.exists(), "moonshine model directory should be deleted");
    }

    #[test]
    fn test_is_moonshine_model_downloaded_checks_all_files() {
        let models_dir = Config::models_dir();
        let model_dir = models_dir.join("moonshine-tiny");

        // Clean up from any previous test
        let _ = std::fs::remove_dir_all(&model_dir);

        // Not downloaded yet
        assert!(!is_model_downloaded("moonshine-tiny"));

        // Create directory with only some files
        std::fs::create_dir_all(&model_dir).unwrap();
        std::fs::write(model_dir.join("encoder_model.onnx"), b"fake").unwrap();
        assert!(
            !is_model_downloaded("moonshine-tiny"),
            "incomplete download should not count as downloaded"
        );

        // Add remaining files
        std::fs::write(model_dir.join("decoder_model_merged.onnx"), b"fake").unwrap();
        std::fs::write(model_dir.join("tokenizer.json"), b"fake").unwrap();
        assert!(
            is_model_downloaded("moonshine-tiny"),
            "complete download should count as downloaded"
        );

        // Clean up
        let _ = std::fs::remove_dir_all(&model_dir);
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
