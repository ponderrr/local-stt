//! Static registry of available Whisper GGML models (tiny through large-v3)
//! with HuggingFace download URLs and size metadata.

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhisperModel {
    pub id: String,
    pub display_name: String,
    pub filename: String,
    pub url: String,
    pub size_bytes: u64,
    pub vram_mb: u16,
}

static MODEL_REGISTRY: LazyLock<Vec<WhisperModel>> = LazyLock::new(|| {
    vec![
        WhisperModel {
            id: "tiny".to_string(),
            display_name: "Tiny (~75 MB)".to_string(),
            filename: "ggml-tiny.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
                .to_string(),
            size_bytes: 77_691_713,
            vram_mb: 1000,
        },
        WhisperModel {
            id: "base".to_string(),
            display_name: "Base (~150 MB)".to_string(),
            filename: "ggml-base.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
                .to_string(),
            size_bytes: 147_951_465,
            vram_mb: 1000,
        },
        WhisperModel {
            id: "small".to_string(),
            display_name: "Small (~500 MB)".to_string(),
            filename: "ggml-small.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"
                .to_string(),
            size_bytes: 487_601_967,
            vram_mb: 1500,
        },
        WhisperModel {
            id: "medium".to_string(),
            display_name: "Medium (~1.5 GB)".to_string(),
            filename: "ggml-medium.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin"
                .to_string(),
            size_bytes: 1_533_774_781,
            vram_mb: 3000,
        },
        WhisperModel {
            id: "large-v3".to_string(),
            display_name: "Large V3 (~3 GB)".to_string(),
            filename: "ggml-large-v3.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin"
                .to_string(),
            size_bytes: 3_093_846_125,
            vram_mb: 6000,
        },
        WhisperModel {
            id: "distil-large-v3".to_string(),
            display_name: "Distil Large V3 (~1.5 GB, fast)".to_string(),
            filename: "ggml-distil-large-v3.bin".to_string(),
            url: "https://huggingface.co/distil-whisper/distil-large-v3-ggml/resolve/main/ggml-distil-large-v3.bin"
                .to_string(),
            size_bytes: 1_521_038_733,
            vram_mb: 2000,
        },
        WhisperModel {
            id: "large-v3-turbo".to_string(),
            display_name: "Large V3 Turbo (~1.6 GB, multilingual)".to_string(),
            filename: "ggml-large-v3-turbo.bin".to_string(),
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin"
                .to_string(),
            size_bytes: 1_620_150_822,
            vram_mb: 2500,
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
    fn test_registry_has_seven_models() {
        let registry = get_model_registry();
        assert_eq!(
            registry.len(),
            7,
            "registry should contain exactly 7 models"
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
    fn test_registry_all_filenames_end_with_bin() {
        let registry = get_model_registry();
        for model in registry {
            assert!(
                model.filename.ends_with(".bin"),
                "model {} filename should end with .bin",
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
                model.size_bytes > 50_000_000,
                "model {} should be larger than 50MB, got {} bytes",
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
        // Two calls should return the same data
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
}
