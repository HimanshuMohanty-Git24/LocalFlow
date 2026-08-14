//! Application error types. Production paths return `Result`; they must not panic.

use serde::Serialize;

/// Recoverable application error shown to the UI or logged (never includes transcripts).
#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    #[error("internal lock was poisoned")]
    LockPoisoned,
    #[error("window '{0}' is not available")]
    WindowMissing(String),
    #[error("tray icon is missing")]
    TrayIconMissing,
    #[error("microphone unavailable")]
    MicrophoneUnavailable,
    #[error("selected microphone is not available")]
    MicrophoneNotFound,
    #[error("microphone permission denied or device failed")]
    MicrophoneAccess,
    #[error("unsupported microphone sample format")]
    UnsupportedSampleFormat,
    #[error("already recording")]
    AlreadyRecording,
    #[error("hotkey listener failed")]
    HotkeyUnavailable,
    #[error("model file is missing")]
    ModelMissing,
    #[error("model file is corrupted")]
    ModelCorrupt,
    #[error("ASR initialization failed")]
    AsrInitFailed,
    #[error("transcription failed")]
    TranscriptionFailed,
    #[error("VAD initialization failed")]
    VadInitFailed,
    #[error("LLM initialization failed")]
    LlmInitFailed,
    #[error("could not type into the focused app; text is on the clipboard")]
    InjectionFailed,
    #[error("{0}")]
    Message(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl AppError {
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }
}
