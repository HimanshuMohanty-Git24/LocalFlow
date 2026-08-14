//! Local-only configuration, persistence, and optional saved dictation data.

pub mod autostart;
pub mod settings;

pub use settings::{
    append_history, ensure_app_data_dir, load, recording_path, save, HotkeyMode, Settings,
};
