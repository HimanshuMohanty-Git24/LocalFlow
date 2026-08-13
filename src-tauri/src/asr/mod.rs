//! Offline speech recognition. Whisper.cpp is the V1 backend.

pub mod engine;
pub mod paths;
pub mod whisper;

pub use engine::{SpeechRecognizer, TranscriptionOptions};
pub use whisper::WhisperBackend;
