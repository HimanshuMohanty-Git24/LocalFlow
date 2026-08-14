//! Speech recognition interface. Backends must run off the audio callback thread.

use crate::errors::AppError;

/// Options for a single transcription call.
#[derive(Debug, Clone)]
pub struct TranscriptionOptions {
    pub language: &'static str,
}

impl Default for TranscriptionOptions {
    fn default() -> Self {
        Self { language: "en" }
    }
}

/// Offline ASR result. Never write `text` to logs.
#[derive(Debug, Clone)]
pub struct Transcript {
    pub text: String,
    pub duration_ms: u64,
}

/// Pluggable recognizer. Whisper is first; other engines can implement this later.
pub trait SpeechRecognizer {
    fn load(&mut self) -> Result<(), AppError>;
    #[allow(dead_code)]
    fn unload(&mut self);
    fn transcribe(
        &mut self,
        audio: &[f32],
        options: TranscriptionOptions,
    ) -> Result<Transcript, AppError>;
    #[allow(dead_code)]
    fn supports_streaming(&self) -> bool;
    fn model_label(&self) -> &str;
}
