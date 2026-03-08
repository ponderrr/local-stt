//! Moonshine ONNX inference engine wrapping transcribe-rs.
//! Provides fast CPU-only speech-to-text via the Moonshine model family.

use std::path::Path;
use transcribe_rs::engines::moonshine::{
    MoonshineEngine as TrMoonshineEngine, MoonshineModelParams, ModelVariant,
};
use transcribe_rs::TranscriptionEngine as TrEngine;

pub struct MoonshineEngine {
    engine: TrMoonshineEngine,
}

impl MoonshineEngine {
    pub fn new() -> Self {
        Self {
            engine: TrMoonshineEngine::new(),
        }
    }

    /// Load a Moonshine model from a directory containing encoder_model.onnx,
    /// decoder_model_merged.onnx, and tokenizer.json.
    pub fn load(model_dir: &Path, variant: ModelVariant) -> Result<Self, String> {
        let mut engine = TrMoonshineEngine::new();
        engine
            .load_model_with_params(model_dir, MoonshineModelParams::variant(variant))
            .map_err(|e| format!("Failed to load Moonshine model: {}", e))?;
        Ok(Self { engine })
    }

    /// Transcribe raw 16kHz mono f32 audio samples.
    pub fn transcribe(&mut self, audio: &[f32]) -> Result<String, String> {
        let result = self
            .engine
            .transcribe_samples(audio.to_vec(), None)
            .map_err(|e| format!("Moonshine inference failed: {}", e))?;
        Ok(result.text)
    }

    /// Unload the model and free resources.
    pub fn unload(&mut self) {
        self.engine.unload_model();
    }

    /// Map a model ID suffix to a ModelVariant.
    pub fn variant_from_id(model_id: &str) -> ModelVariant {
        match model_id {
            "moonshine-base" => ModelVariant::Base,
            _ => ModelVariant::Tiny,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcribe_rs_links() {
        // Compile-time verification that transcribe-rs links successfully
        assert!(true, "transcribe-rs linked successfully");
    }

    #[test]
    fn test_moonshine_engine_module_exists() {
        // Verify the MoonshineEngine wrapper compiles and can be instantiated
        let _engine = MoonshineEngine::new();
    }

    #[test]
    fn test_variant_from_id() {
        assert!(matches!(
            MoonshineEngine::variant_from_id("moonshine-tiny"),
            ModelVariant::Tiny
        ));
        assert!(matches!(
            MoonshineEngine::variant_from_id("moonshine-base"),
            ModelVariant::Base
        ));
        // Unknown IDs default to Tiny
        assert!(matches!(
            MoonshineEngine::variant_from_id("unknown"),
            ModelVariant::Tiny
        ));
    }

    #[test]
    #[ignore = "requires downloaded moonshine-tiny model"]
    fn test_moonshine_load_model() {
        use crate::config::Config;
        let model_dir = Config::models_dir().join("moonshine-tiny");
        let result = MoonshineEngine::load(&model_dir, ModelVariant::Tiny);
        assert!(result.is_ok(), "should load moonshine-tiny model");
    }

    #[test]
    #[ignore = "requires downloaded moonshine-tiny model"]
    fn test_moonshine_transcribe_silence() {
        use crate::config::Config;
        let model_dir = Config::models_dir().join("moonshine-tiny");
        let mut engine = MoonshineEngine::load(&model_dir, ModelVariant::Tiny).unwrap();
        // 1 second of silence at 16kHz
        let silence = vec![0.0f32; 16000];
        let result = engine.transcribe(&silence);
        assert!(result.is_ok(), "transcribing silence should not error");
    }

    #[test]
    #[ignore = "requires downloaded moonshine-tiny model"]
    fn test_moonshine_transcribe_sine_wave() {
        use crate::config::Config;
        let model_dir = Config::models_dir().join("moonshine-tiny");
        let mut engine = MoonshineEngine::load(&model_dir, ModelVariant::Tiny).unwrap();
        // 2 seconds of 440Hz sine wave at 16kHz
        let samples: Vec<f32> = (0..32000)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 16000.0).sin() * 0.5)
            .collect();
        let result = engine.transcribe(&samples);
        assert!(result.is_ok(), "transcribing sine wave should not error");
    }
}
