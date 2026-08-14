//! Strongly typed pipeline events. Payload strings are for in-memory use only
//! and must not be written to logs.

/// Events that drive the dictation state machine.
#[allow(dead_code)] // Phase 0: pipeline not wired to commands yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowEvent {
    HotkeyPressed,
    HotkeyReleased,
    RecordingStarted,
    RecordingStopped,
    SpeechStarted,
    SpeechEnded,
    PartialTranscript,
    FinalTranscript,
    NormalizationStarted,
    NormalizationFinished,
    InjectionFinished,
    Error,
    Reset,
}

impl FlowEvent {
    /// Short label for diagnostics. Never includes transcript text.
    pub fn name(&self) -> &'static str {
        match self {
            Self::HotkeyPressed => "hotkey_pressed",
            Self::HotkeyReleased => "hotkey_released",
            Self::RecordingStarted => "recording_started",
            Self::RecordingStopped => "recording_stopped",
            Self::SpeechStarted => "speech_started",
            Self::SpeechEnded => "speech_ended",
            Self::PartialTranscript => "partial_transcript",
            Self::FinalTranscript => "final_transcript",
            Self::NormalizationStarted => "normalization_started",
            Self::NormalizationFinished => "normalization_finished",
            Self::InjectionFinished => "injection_finished",
            Self::Error => "error",
            Self::Reset => "reset",
        }
    }
}
