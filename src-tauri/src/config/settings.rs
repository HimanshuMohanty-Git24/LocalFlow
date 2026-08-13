//! User settings. Phase 0 keeps these in memory; SQLite persistence comes later.

use serde::{Deserialize, Serialize};

/// How the dictation hotkey behaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyMode {
    PushToTalk,
    Toggle,
}

/// Local application settings. No network fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_privacy_first() {
        let s = Settings::default();
        assert!(!s.save_text_history);
        assert!(!s.save_audio);
        assert!(s.preserve_clipboard);
        assert!(s.llm_enabled);
        assert_eq!(s.hotkey_mode, HotkeyMode::PushToTalk);
    }

    #[test]
    fn round_trips_json() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }
}
