//! Persistent, local-only user settings and optional dictation storage.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::errors::AppError;

const APP_DIR_NAME: &str = "LocalFlow";
const SETTINGS_FILE: &str = "settings.json";
const HISTORY_FILE: &str = "history.jsonl";

/// How the dictation hotkey behaves.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyMode {
    #[default]
    PushToTalk,
    Toggle,
}

/// Local application settings. No network fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub dictation_hotkey: String,
    pub hotkey_mode: HotkeyMode,
    /// Empty string means the system default input device.
    pub microphone_id: String,
    pub start_on_login: bool,
    pub preserve_clipboard: bool,
    pub save_text_history: bool,
    pub save_audio: bool,
    /// Local Qwen cleanup after deterministic rules. No network.
    pub llm_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dictation_hotkey: "Ctrl+B".to_string(),
            hotkey_mode: HotkeyMode::PushToTalk,
            microphone_id: String::new(),
            start_on_login: false,
            preserve_clipboard: true,
            save_text_history: false,
            save_audio: false,
            llm_enabled: true,
        }
    }
}

pub fn app_data_dir() -> Result<PathBuf, AppError> {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::message("Windows local app-data folder is unavailable"))?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn ensure_app_data_dir() -> Result<PathBuf, AppError> {
    let dir = app_data_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|_| AppError::message("could not create the LocalFlow data folder"))?;
    Ok(dir)
}

pub fn load() -> Result<Settings, AppError> {
    load_from(&app_data_dir()?.join(SETTINGS_FILE))
}

pub fn save(settings: &Settings) -> Result<(), AppError> {
    let path = ensure_app_data_dir()?.join(SETTINGS_FILE);
    save_to(&path, settings)
}

pub fn recording_path() -> Result<PathBuf, AppError> {
    let dir = ensure_app_data_dir()?.join("recordings");
    fs::create_dir_all(&dir)
        .map_err(|_| AppError::message("could not create the recordings folder"))?;
    let stamp = unix_millis();
    Ok(dir.join(format!("localflow-{stamp}-{}.wav", std::process::id())))
}

pub fn append_history(text: &str) -> Result<(), AppError> {
    if text.is_empty() {
        return Ok(());
    }
    let path = ensure_app_data_dir()?.join(HISTORY_FILE);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| AppError::message("could not save text history"))?;
    let record = serde_json::json!({
        "created_at_unix_ms": unix_millis(),
        "text": text,
    });
    serde_json::to_writer(&mut file, &record)
        .map_err(|_| AppError::message("could not save text history"))?;
    writeln!(file).map_err(|_| AppError::message("could not save text history"))
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0)
}

fn load_from(path: &Path) -> Result<Settings, AppError> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    let json = fs::read_to_string(path)
        .map_err(|_| AppError::message("could not read LocalFlow settings"))?;
    serde_json::from_str(&json)
        .map_err(|_| AppError::message("LocalFlow settings are not valid JSON"))
}

fn save_to(path: &Path, settings: &Settings) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| AppError::message("could not create the settings folder"))?;
    }
    let json = serde_json::to_vec_pretty(settings)
        .map_err(|_| AppError::message("could not serialize LocalFlow settings"))?;
    fs::write(path, json).map_err(|_| AppError::message("could not save LocalFlow settings"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_privacy_first() {
        let settings = Settings::default();
        assert!(!settings.save_text_history);
        assert!(!settings.save_audio);
        assert!(settings.preserve_clipboard);
        assert!(settings.llm_enabled);
        assert_eq!(settings.hotkey_mode, HotkeyMode::PushToTalk);
    }

    #[test]
    fn round_trips_json() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, back);
    }

    #[test]
    fn missing_fields_receive_private_defaults() {
        let settings: Settings = serde_json::from_str(r#"{"dictation_hotkey":"Ctrl+B"}"#).unwrap();
        assert!(!settings.save_audio);
        assert!(!settings.save_text_history);
        assert!(settings.preserve_clipboard);
    }

    #[test]
    fn settings_persist_to_disk() {
        let dir = std::env::temp_dir().join(format!(
            "localflow-settings-test-{}-{}",
            std::process::id(),
            unix_millis()
        ));
        let path = dir.join(SETTINGS_FILE);
        let expected = Settings {
            save_audio: true,
            ..Settings::default()
        };
        save_to(&path, &expected).unwrap();
        assert_eq!(load_from(&path).unwrap(), expected);
        let _ = fs::remove_dir_all(dir);
    }
}
