//! Voice activity detection. Silero via whisper.cpp.

#[cfg(test)]
mod crop;
pub mod silero;

pub use silero::SileroVad;
