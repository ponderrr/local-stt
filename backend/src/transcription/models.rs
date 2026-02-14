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
}
