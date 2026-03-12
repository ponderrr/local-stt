//! Static registry of available ASR models (Whisper GGML + Moonshine ONNX)
//! with HuggingFace download URLs and size metadata.

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    #[default]
    WhisperGgml,
    MoonshineOnnx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub id: String,
    pub display_name: String,
    pub filename: String,
    /// Primary download URL (used for single-file Whisper models).
    /// Empty for multi-file models that use `files` instead.
    pub url: String,
    pub size_bytes: u64,
    pub vram_mb: u16,
    #[serde(default)]
    pub model_type: ModelType,
    /// For multi-file models (Moonshine ONNX), list of (filename, url) pairs.
    /// Empty for single-file models (Whisper GGML).
    #[serde(default)]
    pub files: Vec<(String, String)>,
}

const HF_MOONSHINE: &str =
    "https://huggingface.co/UsefulSensors/moonshine/resolve/main/onnx/merged";

static MODEL_REGISTRY: LazyLock<Vec<WhisperModel>> = LazyLock::new(|| {
    vec![
        // --- Whisper GGML models ---
        WhisperModel {
            id: "tiny".to_string(),
            display_name: "Tiny (~75 MB)".to_string(),
            filename: "ggml-tiny.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
                .to_string(),
            size_bytes: 77_691_713,
            vram_mb: 1000,
            model_type: ModelType::WhisperGgml,
            files: vec![],
        },
        WhisperModel {
            id: "base".to_string(),
            display_name: "Base (~150 MB)".to_string(),
            filename: "ggml-base.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
                .to_string(),
            size_bytes: 147_951_465,
            vram_mb: 1000,
            model_type: ModelType::WhisperGgml,
            files: vec![],
        },
        WhisperModel {
            id: "small".to_string(),
            display_name: "Small (~500 MB)".to_string(),
            filename: "ggml-small.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
                .to_string(),
            size_bytes: 487_601_967,
            vram_mb: 1500,
            model_type: ModelType::WhisperGgml,
            files: vec![],
        },
        WhisperModel {
            id: "medium".to_string(),
            display_name: "Medium (~1.5 GB)".to_string(),
            filename: "ggml-medium.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
                .to_string(),
            size_bytes: 1_533_774_781,
            vram_mb: 3000,
            model_type: ModelType::WhisperGgml,
            files: vec![],
        },
        WhisperModel {
            id: "large-v3".to_string(),
            display_name: "Large V3 (~3 GB)".to_string(),
            filename: "ggml-large-v3.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .to_string(),
            size_bytes: 3_093_846_125,
            vram_mb: 6000,
            model_type: ModelType::WhisperGgml,
            files: vec![],
        },
        WhisperModel {
            id: "distil-large-v3".to_string(),
            display_name: "Distil Large V3 (~1.5 GB, fast)".to_string(),
            filename: "ggml-distil-large-v3.bin".to_string(),
            url: "https://huggingface.co/distil-whisper/distil-large-v3-ggml/resolve/main/ggml-distil-large-v3.bin"
                .to_string(),
            size_bytes: 1_521_038_733,
            vram_mb: 2000,
            model_type: ModelType::WhisperGgml,
            files: vec![],
        },
        WhisperModel {
            id: "large-v3-turbo".to_string(),
            display_name: "Large V3 Turbo (~1.6 GB, multilingual)".to_string(),
            filename: "ggml-large-v3-turbo.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
                .to_string(),
            size_bytes: 1_620_150_822,
            vram_mb: 2500,
            model_type: ModelType::WhisperGgml,
            files: vec![],
        },
        // --- Moonshine ONNX models ---
        WhisperModel {
            id: "moonshine-tiny".to_string(),
            display_name: "Moonshine Tiny (~28 MB, CPU, fast)".to_string(),
            filename: "moonshine-tiny".to_string(),
            url: String::new(),
            size_bytes: 29_570_048, // ~28.2 MB total
            vram_mb: 0,            // CPU only
            model_type: ModelType::MoonshineOnnx,
            files: vec![
                (
                    "encoder_model.onnx".to_string(),
                    format!("{}/tiny/quantized/encoder_model.onnx", HF_MOONSHINE),
                ),
                (
                    "decoder_model_merged.onnx".to_string(),
                    format!("{}/tiny/quantized/decoder_model_merged.onnx", HF_MOONSHINE),
                ),
                (
                    "tokenizer.json".to_string(),
                    format!("{}/base/float/tokenizer.json", HF_MOONSHINE),
                ),
            ],
        },
        WhisperModel {
            id: "moonshine-base".to_string(),
            display_name: "Moonshine Base (~63 MB, CPU)".to_string(),
            filename: "moonshine-base".to_string(),
            url: String::new(),
            size_bytes: 66_060_288, // ~63 MB total
            vram_mb: 0,            // CPU only
            model_type: ModelType::MoonshineOnnx,
            files: vec![
                (
                    "encoder_model.onnx".to_string(),
                    format!("{}/base/quantized/encoder_model.onnx", HF_MOONSHINE),
                ),
                (
                    "decoder_model_merged.onnx".to_string(),
                    format!("{}/base/quantized/decoder_model_merged.onnx", HF_MOONSHINE),
                ),
                (
                    "tokenizer.json".to_string(),
                    format!("{}/base/float/tokenizer.json", HF_MOONSHINE),
                ),
            ],
        },
    ]
});

pub fn get_model_registry() -> &'static [WhisperModel] {
    &MODEL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_contains_models() {
        let registry = get_model_registry();
        assert!(!registry.is_empty());
        assert!(registry.iter().any(|m| m.id == "large-v3"));
        assert!(registry.iter().any(|m| m.id == "tiny"));
    }

    #[test]
    fn test_registry_has_nine_models() {
        let registry = get_model_registry();
        assert_eq!(
            registry.len(),
            9,
            "registry should contain exactly 9 models (7 Whisper + 2 Moonshine)"
        );
    }

    #[test]
    fn test_registry_ids_are_unique() {
        let registry = get_model_registry();
        let mut ids: Vec<&str> = registry.iter().map(|m| m.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), registry.len(), "all model IDs should be unique");
    }

    #[test]
    fn test_registry_filenames_are_unique() {
        let registry = get_model_registry();
        let mut filenames: Vec<&str> = registry.iter().map(|m| m.filename.as_str()).collect();
        filenames.sort();
        filenames.dedup();
        assert_eq!(
            filenames.len(),
            registry.len(),
            "all filenames should be unique"
        );
    }

    #[test]
    fn test_registry_all_models_have_valid_urls() {
        let registry = get_model_registry();
        for model in registry {
            match model.model_type {
                ModelType::WhisperGgml => {
                    assert!(
                        model.url.starts_with("https://"),
                        "model {} URL should start with https://",
                        model.id
                    );
                    assert!(
                        model.url.contains("huggingface.co"),
                        "model {} URL should point to huggingface",
                        model.id
                    );
                }
                ModelType::MoonshineOnnx => {
                    assert!(
                        !model.files.is_empty(),
                        "Moonshine model {} should have files",
                        model.id
                    );
                    for (fname, url) in &model.files {
                        assert!(
                            url.starts_with("https://") && url.contains("huggingface.co"),
                            "model {} file {} URL should point to huggingface",
                            model.id,
                            fname
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_registry_all_models_have_nonzero_size() {
        let registry = get_model_registry();
        for model in registry {
            assert!(
                model.size_bytes > 0,
                "model {} should have nonzero size_bytes",
                model.id
            );
        }
    }

    #[test]
    fn test_registry_all_models_have_display_names() {
        let registry = get_model_registry();
        for model in registry {
            assert!(
                !model.display_name.is_empty(),
                "model {} should have a display name",
                model.id
            );
        }
    }

    #[test]
    fn test_registry_whisper_filenames_end_with_bin() {
        let registry = get_model_registry();
        for model in registry
            .iter()
            .filter(|m| m.model_type == ModelType::WhisperGgml)
        {
            assert!(
                model.filename.ends_with(".bin"),
                "Whisper model {} filename should end with .bin",
                model.id
            );
        }
    }

    #[test]
    fn test_registry_contains_all_expected_models() {
        let registry = get_model_registry();
        let expected_ids = [
            "tiny",
            "base",
            "small",
            "medium",
            "large-v3",
            "distil-large-v3",
            "large-v3-turbo",
            "moonshine-tiny",
            "moonshine-base",
        ];
        for id in &expected_ids {
            assert!(
                registry.iter().any(|m| m.id == *id),
                "registry should contain model with id '{}'",
                id
            );
        }
    }

    #[test]
    fn test_registry_all_models_have_reasonable_sizes() {
        let registry = get_model_registry();
        for model in registry {
            assert!(
                model.size_bytes > 1_000_000,
                "model {} should be larger than 1MB, got {} bytes",
                model.id,
                model.size_bytes
            );
            assert!(
                model.size_bytes < 10_000_000_000,
                "model {} should be smaller than 10GB, got {} bytes",
                model.id,
                model.size_bytes
            );
        }
    }

    #[test]
    fn test_registry_distil_and_turbo_models() {
        let registry = get_model_registry();

        let distil = registry.iter().find(|m| m.id == "distil-large-v3");
        assert!(distil.is_some(), "should find distil-large-v3");
        let distil = distil.unwrap();
        assert_eq!(distil.filename, "ggml-distil-large-v3.bin");
        assert!(distil.url.contains("distil-whisper"));
        assert!(distil.size_bytes > 1_500_000_000, "distil should be ~1.5GB");

        let turbo = registry.iter().find(|m| m.id == "large-v3-turbo");
        assert!(turbo.is_some(), "should find large-v3-turbo");
        let turbo = turbo.unwrap();
        assert_eq!(turbo.filename, "ggml-large-v3-turbo.bin");
        assert!(turbo.url.contains("ggerganov"));
        assert!(turbo.size_bytes > 1_600_000_000, "turbo should be ~1.6GB");
    }

    #[test]
    fn test_registry_is_static_and_consistent() {
        let reg1 = get_model_registry();
        let reg2 = get_model_registry();
        assert_eq!(reg1.len(), reg2.len());
        for (m1, m2) in reg1.iter().zip(reg2.iter()) {
            assert_eq!(m1.id, m2.id);
            assert_eq!(m1.size_bytes, m2.size_bytes);
        }
    }

    #[test]
    fn test_model_find_by_id() {
        let registry = get_model_registry();
        let tiny = registry.iter().find(|m| m.id == "tiny");
        assert!(tiny.is_some(), "should find 'tiny' model");
        let tiny = tiny.unwrap();
        assert_eq!(tiny.filename, "ggml-tiny.bin");
    }

    #[test]
    fn test_model_find_nonexistent_returns_none() {
        let registry = get_model_registry();
        let result = registry.iter().find(|m| m.id == "nonexistent-model");
        assert!(result.is_none(), "nonexistent model should not be found");
    }

    #[test]
    fn test_moonshine_models() {
        let registry = get_model_registry();

        let tiny = registry.iter().find(|m| m.id == "moonshine-tiny").unwrap();
        assert_eq!(tiny.model_type, ModelType::MoonshineOnnx);
        assert_eq!(tiny.vram_mb, 0, "Moonshine is CPU-only");
        assert_eq!(tiny.files.len(), 3, "moonshine-tiny needs 3 files");
        assert!(tiny.files.iter().any(|(f, _)| f == "encoder_model.onnx"));
        assert!(tiny
            .files
            .iter()
            .any(|(f, _)| f == "decoder_model_merged.onnx"));
        assert!(tiny.files.iter().any(|(f, _)| f == "tokenizer.json"));

        let base = registry.iter().find(|m| m.id == "moonshine-base").unwrap();
        assert_eq!(base.model_type, ModelType::MoonshineOnnx);
        assert_eq!(base.files.len(), 3, "moonshine-base needs 3 files");
        assert!(
            base.size_bytes > tiny.size_bytes,
            "base should be larger than tiny"
        );
    }

    #[test]
    fn test_model_type_default_is_whisper() {
        assert_eq!(ModelType::default(), ModelType::WhisperGgml);
    }
}
