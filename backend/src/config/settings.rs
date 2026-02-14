use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    TypeIntoField,
    Clipboard,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub hotkey: String,
    pub default_model: String,
    pub output_mode: OutputMode,
    pub audio_device: Option<String>,
    pub language: String,
    pub vad_threshold: f32,
    pub chunk_duration_ms: u32,
    pub overlap_ms: u32,
    pub downloaded_models: Vec<String>,
    pub first_run_complete: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            hotkey: "Ctrl+Shift+Space".to_string(),
            default_model: "large-v3".to_string(),
            output_mode: OutputMode::Both,
            audio_device: None,
            language: "auto".to_string(),
            vad_threshold: 0.01,
            chunk_duration_ms: 3000,
            overlap_ms: 500,
            downloaded_models: Vec::new(),
            first_run_complete: false,
        }
    }
}

impl Config {
    pub fn app_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".whispertype")
    }

    pub fn models_dir() -> PathBuf {
        Self::app_dir().join("models")
    }

    pub fn config_path() -> PathBuf {
        Self::app_dir().join("config.json")
    }

    pub fn ensure_dirs() -> Result<(), String> {
        let app_dir = Self::app_dir();
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir).map_err(|e| format!("Failed to create app dir: {}", e))?;
        }

        let models_dir = Self::models_dir();
        if !models_dir.exists() {
            fs::create_dir_all(&models_dir)
                .map_err(|e| format!("Failed to create models dir: {}", e))?;
        }

        let logs_dir = app_dir.join("logs");
        if !logs_dir.exists() {
            fs::create_dir_all(&logs_dir)
                .map_err(|e| format!("Failed to create logs dir: {}", e))?;
        }

        Ok(())
    }

    pub fn load() -> Result<Self, String> {
        Self::ensure_dirs()?;
        let path = Self::config_path();
        if path.exists() {
            let content =
                fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<(), String> {
        Self::ensure_dirs()?;
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(Self::config_path(), content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, 1);
        assert_eq!(config.hotkey, "Ctrl+Shift+Space");
        assert_eq!(config.output_mode, OutputMode::Both);
    }

    #[test]
    fn test_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.hotkey, deserialized.hotkey);
        assert_eq!(config.output_mode, deserialized.output_mode);
    }
}
