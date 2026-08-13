//! Voice activity detection. Silero via whisper.cpp.

pub mod crop;
pub mod silero;

pub use silero::SileroVad;
