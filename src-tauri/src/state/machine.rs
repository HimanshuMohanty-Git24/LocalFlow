//! Dictation state machine. ASR is not wired in Phase 0; transitions are still
//! enforced so later phases cannot skip states.

use super::events::FlowEvent;
use crate::errors::AppError;
use serde::Serialize;

/// High-level application dictation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Idle,
    Preparing,
    Listening,
    SpeechDetected,
    Transcribing,
    Normalizing,
    Injecting,
    Error,
}

impl AppState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Preparing => "preparing",
            Self::Listening => "listening",
            Self::SpeechDetected => "speech_detected",
            Self::Transcribing => "transcribing",
            Self::Normalizing => "normalizing",
            Self::Injecting => "injecting",
            Self::Error => "error",
        }
    }
}

/// Dictation lifecycle. One instance should be shared on the application thread
/// that owns the pipeline — not the real-time audio callback.
#[derive(Debug)]
pub struct StateMachine {
    state: AppState,
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: AppState::Idle,
        }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    /// Applies an event. Invalid transitions return `AppError` and leave state unchanged.
    pub fn apply(&mut self, event: FlowEvent) -> Result<AppState, AppError> {
        if matches!(event, FlowEvent::Reset) {
            self.state = AppState::Idle;
            return Ok(self.state);
        }
        if matches!(event, FlowEvent::Error) {
            self.state = AppState::Error;
            return Ok(self.state);
        }

        let next = match (self.state, event) {
            (AppState::Idle, FlowEvent::HotkeyPressed) => AppState::Preparing,
            (AppState::Preparing, FlowEvent::RecordingStarted) => AppState::Listening,
            (AppState::Listening, FlowEvent::SpeechStarted) => AppState::SpeechDetected,
            (AppState::Listening, FlowEvent::HotkeyReleased) => AppState::Transcribing,
            (AppState::SpeechDetected, FlowEvent::HotkeyReleased) => AppState::Transcribing,
            (AppState::Preparing, FlowEvent::RecordingStopped) => AppState::Idle,
            (AppState::Listening, FlowEvent::RecordingStopped) => AppState::Idle,
            (AppState::SpeechDetected, FlowEvent::RecordingStopped) => AppState::Idle,
            (AppState::Transcribing, FlowEvent::RecordingStopped) => AppState::Idle,
            (AppState::SpeechDetected, FlowEvent::SpeechEnded) => AppState::SpeechDetected,
            (AppState::Transcribing, FlowEvent::FinalTranscript) => AppState::Normalizing,
            (AppState::Normalizing, FlowEvent::NormalizationFinished) => AppState::Injecting,
            (AppState::Injecting, FlowEvent::InjectionFinished) => AppState::Idle,
            (AppState::Error, FlowEvent::HotkeyPressed) => AppState::Preparing,
            _ => {
                return Err(AppError::message(format!(
                    "invalid transition: {} + {}",
                    self.state.as_str(),
                    event.name()
                )));
            }
        };

        tracing::debug!(
            from = self.state.as_str(),
            event = event.name(),
            to = next.as_str(),
            "state transition"
        );
        self.state = next;
        Ok(self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path_reaches_idle() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.state(), AppState::Idle);
        sm.apply(FlowEvent::HotkeyPressed).unwrap();
        sm.apply(FlowEvent::RecordingStarted).unwrap();
        sm.apply(FlowEvent::SpeechStarted).unwrap();
        sm.apply(FlowEvent::HotkeyReleased).unwrap();
        sm.apply(FlowEvent::FinalTranscript).unwrap();
        sm.apply(FlowEvent::NormalizationFinished).unwrap();
        sm.apply(FlowEvent::InjectionFinished).unwrap();
        assert_eq!(sm.state(), AppState::Idle);
    }

    #[test]
    fn release_without_speech_still_transcribes() {
        let mut sm = StateMachine::new();
        sm.apply(FlowEvent::HotkeyPressed).unwrap();
        sm.apply(FlowEvent::RecordingStarted).unwrap();
        sm.apply(FlowEvent::HotkeyReleased).unwrap();
        assert_eq!(sm.state(), AppState::Transcribing);
    }

    #[test]
    fn recording_stopped_returns_idle_without_asr() {
        let mut sm = StateMachine::new();
        sm.apply(FlowEvent::HotkeyPressed).unwrap();
        sm.apply(FlowEvent::RecordingStarted).unwrap();
        sm.apply(FlowEvent::RecordingStopped).unwrap();
        assert_eq!(sm.state(), AppState::Idle);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut sm = StateMachine::new();
        let err = sm.apply(FlowEvent::InjectionFinished).unwrap_err();
        assert!(err.to_string().contains("invalid transition"));
        assert_eq!(sm.state(), AppState::Idle);
    }

    #[test]
    fn error_then_reset_returns_to_idle() {
        let mut sm = StateMachine::new();
        sm.apply(FlowEvent::Error).unwrap();
        assert_eq!(sm.state(), AppState::Error);
        sm.apply(FlowEvent::Reset).unwrap();
        assert_eq!(sm.state(), AppState::Idle);
    }
}
