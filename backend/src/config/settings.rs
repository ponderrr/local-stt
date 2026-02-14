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
    use std::io::Write;

    // --- Default Config Tests ---

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, 1);
        assert_eq!(config.hotkey, "Ctrl+Shift+Space");
        assert_eq!(config.output_mode, OutputMode::Both);
    }

    #[test]
    fn test_default_config_complete_fields() {
        let config = Config::default();
        assert_eq!(config.version, 1);
        assert_eq!(config.hotkey, "Ctrl+Shift+Space");
        assert_eq!(config.default_model, "large-v3");
        assert_eq!(config.output_mode, OutputMode::Both);
        assert!(config.audio_device.is_none());
        assert_eq!(config.language, "auto");
        assert!((config.vad_threshold - 0.01).abs() < 1e-6);
        assert_eq!(config.chunk_duration_ms, 3000);
        assert_eq!(config.overlap_ms, 500);
        assert!(config.downloaded_models.is_empty());
        assert!(!config.first_run_complete);
    }

    // --- Serialization/Deserialization Tests ---

    #[test]
    fn test_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.hotkey, deserialized.hotkey);
        assert_eq!(config.output_mode, deserialized.output_mode);
    }

    #[test]
    fn test_serialization_roundtrip_all_fields() {
        let config = Config {
            version: 2,
            hotkey: "Alt+D".to_string(),
            default_model: "tiny".to_string(),
            output_mode: OutputMode::Clipboard,
            audio_device: Some("USB Mic".to_string()),
            language: "en".to_string(),
            vad_threshold: 0.05,
            chunk_duration_ms: 5000,
            overlap_ms: 1000,
            downloaded_models: vec!["tiny".to_string(), "base".to_string()],
            first_run_complete: true,
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, 2);
        assert_eq!(deserialized.hotkey, "Alt+D");
        assert_eq!(deserialized.default_model, "tiny");
        assert_eq!(deserialized.output_mode, OutputMode::Clipboard);
        assert_eq!(deserialized.audio_device, Some("USB Mic".to_string()));
        assert_eq!(deserialized.language, "en");
        assert!((deserialized.vad_threshold - 0.05).abs() < 1e-6);
        assert_eq!(deserialized.chunk_duration_ms, 5000);
        assert_eq!(deserialized.overlap_ms, 1000);
        assert_eq!(deserialized.downloaded_models, vec!["tiny", "base"]);
        assert!(deserialized.first_run_complete);
    }

    #[test]
    fn test_output_mode_serialization_snake_case() {
        // OutputMode uses #[serde(rename_all = "snake_case")]
        let json_type = serde_json::to_string(&OutputMode::TypeIntoField).unwrap();
        assert_eq!(json_type, "\"type_into_field\"");

        let json_clip = serde_json::to_string(&OutputMode::Clipboard).unwrap();
        assert_eq!(json_clip, "\"clipboard\"");

        let json_both = serde_json::to_string(&OutputMode::Both).unwrap();
        assert_eq!(json_both, "\"both\"");
    }

    #[test]
    fn test_output_mode_deserialization() {
        let type_field: OutputMode = serde_json::from_str("\"type_into_field\"").unwrap();
        assert_eq!(type_field, OutputMode::TypeIntoField);

        let clipboard: OutputMode = serde_json::from_str("\"clipboard\"").unwrap();
        assert_eq!(clipboard, OutputMode::Clipboard);

        let both: OutputMode = serde_json::from_str("\"both\"").unwrap();
        assert_eq!(both, OutputMode::Both);
    }

    #[test]
    fn test_invalid_output_mode_fails() {
        let result: Result<OutputMode, _> = serde_json::from_str("\"invalid_mode\"");
        assert!(result.is_err(), "invalid output mode should fail deserialization");
    }

    #[test]
    fn test_config_with_null_audio_device() {
        let json = r#"{
            "version": 1,
            "hotkey": "Ctrl+Shift+Space",
            "default_model": "large-v3",
            "output_mode": "both",
            "audio_device": null,
            "language": "auto",
            "vad_threshold": 0.01,
            "chunk_duration_ms": 3000,
            "overlap_ms": 500,
            "downloaded_models": [],
            "first_run_complete": false
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.audio_device.is_none());
    }

    #[test]
    fn test_config_with_audio_device() {
        let json = r#"{
            "version": 1,
            "hotkey": "Ctrl+Shift+Space",
            "default_model": "large-v3",
            "output_mode": "both",
            "audio_device": "My USB Mic",
            "language": "auto",
            "vad_threshold": 0.01,
            "chunk_duration_ms": 3000,
            "overlap_ms": 500,
            "downloaded_models": [],
            "first_run_complete": false
        }"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.audio_device, Some("My USB Mic".to_string()));
    }

    #[test]
    fn test_config_deserialization_missing_field_fails() {
        // Missing "version" field
        let json = r#"{
            "hotkey": "Ctrl+Shift+Space",
            "default_model": "large-v3",
            "output_mode": "both",
            "language": "auto"
        }"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err(), "missing required field should fail");
    }

    // --- Path Construction Tests ---

    #[test]
    fn test_app_dir_is_under_home() {
        let app_dir = Config::app_dir();
        let home = dirs::home_dir().expect("home dir");
        assert!(
            app_dir.starts_with(&home),
            "app_dir should be under home directory"
        );
        assert!(
            app_dir.ends_with(".whispertype"),
            "app_dir should end with .whispertype"
        );
    }

    #[test]
    fn test_models_dir_is_under_app_dir() {
        let models_dir = Config::models_dir();
        let app_dir = Config::app_dir();
        assert!(
            models_dir.starts_with(&app_dir),
            "models_dir should be under app_dir"
        );
        assert!(
            models_dir.ends_with("models"),
            "models_dir should end with 'models'"
        );
    }

    #[test]
    fn test_config_path_is_under_app_dir() {
        let config_path = Config::config_path();
        let app_dir = Config::app_dir();
        assert!(
            config_path.starts_with(&app_dir),
            "config_path should be under app_dir"
        );
        assert!(
            config_path.ends_with("config.json"),
            "config_path should end with 'config.json'"
        );
    }

    // --- File System Tests (using temp dir) ---

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let config_path = temp_dir.path().join("config.json");

        let config = Config {
            version: 1,
            hotkey: "Ctrl+Shift+Space".to_string(),
            default_model: "tiny".to_string(),
            output_mode: OutputMode::Clipboard,
            audio_device: Some("Test Mic".to_string()),
            language: "en".to_string(),
            vad_threshold: 0.02,
            chunk_duration_ms: 4000,
            overlap_ms: 750,
            downloaded_models: vec!["tiny".to_string()],
            first_run_complete: true,
        };

        // Save to temp path
        let content = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &content).unwrap();

        // Load from temp path
        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded: Config = serde_json::from_str(&loaded_content).unwrap();

        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.default_model, "tiny");
        assert_eq!(loaded.output_mode, OutputMode::Clipboard);
        assert_eq!(loaded.audio_device, Some("Test Mic".to_string()));
        assert!(loaded.first_run_complete);
    }

    #[test]
    fn test_corrupted_config_file_fails() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let config_path = temp_dir.path().join("config.json");

        // Write invalid JSON
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(b"not valid json {{{").unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<Config, _> = serde_json::from_str(&content);
        assert!(result.is_err(), "corrupted config file should fail to parse");
    }

    #[test]
    fn test_empty_config_file_fails() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let config_path = temp_dir.path().join("config.json");

        fs::write(&config_path, "").unwrap();
        let content = fs::read_to_string(&config_path).unwrap();
        let result: Result<Config, _> = serde_json::from_str(&content);
        assert!(result.is_err(), "empty config file should fail to parse");
    }

    // --- OutputMode Equality ---

    #[test]
    fn test_output_mode_equality() {
        assert_eq!(OutputMode::Both, OutputMode::Both);
        assert_eq!(OutputMode::Clipboard, OutputMode::Clipboard);
        assert_eq!(OutputMode::TypeIntoField, OutputMode::TypeIntoField);
        assert_ne!(OutputMode::Both, OutputMode::Clipboard);
        assert_ne!(OutputMode::Clipboard, OutputMode::TypeIntoField);
    }
}
