//! Global dictation hotkey. Platform backends live behind `spawn`.

use crate::config::HotkeyMode;

pub mod manager;

#[cfg(windows)]
mod windows;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub static LONG_LISTEN: AtomicBool = AtomicBool::new(false);
static LONG_IGNORE_UNTIL: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn arm_long_listen() {
    LONG_LISTEN.store(true, Ordering::SeqCst);
    LONG_IGNORE_UNTIL.store(now_ms().saturating_add(600), Ordering::SeqCst);
}

pub fn disarm_long_listen() {
    LONG_LISTEN.store(false, Ordering::SeqCst);
    LONG_IGNORE_UNTIL.store(0, Ordering::SeqCst);
}

pub fn long_listen_should_stop() -> bool {
    if !LONG_LISTEN.load(Ordering::SeqCst) {
        return false;
    }
    now_ms() >= LONG_IGNORE_UNTIL.load(Ordering::SeqCst)
}

/// Hold at least this long, then release, to finish a short take.
pub const SHORT_HOLD_MS: u128 = 280;
/// After a quick tap, a second Ctrl+B within this window starts long-listen.
pub const DOUBLE_TAP_MS: u128 = 1500;

/// Edge of the configured dictation key. Hook threads must only send this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    Press,
    Release,
    /// Any key while long-listen is active, or the double-tap wait expired.
    StopNow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListenMode {
    #[default]
    Off,
    Holding,
    AwaitingSecondTap,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PttEffect {
    None,
    Start,
    Stop,
    GoLong,
}

/// Short = hold Ctrl+B. Long = tap Ctrl+B twice, then any key stops.
pub fn decide_ptt(mode: ListenMode, action: HotkeyAction, elapsed_ms: u128) -> (ListenMode, PttEffect) {
    match (mode, action) {
        (ListenMode::Off, HotkeyAction::Press) => (ListenMode::Holding, PttEffect::Start),
        (ListenMode::Off, _) => (ListenMode::Off, PttEffect::None),

        (ListenMode::Holding, HotkeyAction::Press) if elapsed_ms < DOUBLE_TAP_MS => {
            (ListenMode::Long, PttEffect::GoLong)
        }
        (ListenMode::Holding, HotkeyAction::Press) => (ListenMode::Holding, PttEffect::None),
        (ListenMode::Holding, HotkeyAction::Release) if elapsed_ms >= SHORT_HOLD_MS => {
            (ListenMode::Off, PttEffect::Stop)
        }
        (ListenMode::Holding, HotkeyAction::Release) => {
            (ListenMode::AwaitingSecondTap, PttEffect::None)
        }
        (ListenMode::Holding, HotkeyAction::StopNow) => (ListenMode::Off, PttEffect::Stop),

        (ListenMode::AwaitingSecondTap, HotkeyAction::Press) => (ListenMode::Long, PttEffect::GoLong),
        (ListenMode::AwaitingSecondTap, HotkeyAction::Release) => {
            (ListenMode::AwaitingSecondTap, PttEffect::None)
        }
        (ListenMode::AwaitingSecondTap, HotkeyAction::StopNow) => (ListenMode::Off, PttEffect::Stop),

        (ListenMode::Long, HotkeyAction::StopNow) => (ListenMode::Off, PttEffect::Stop),
        (ListenMode::Long, _) => (ListenMode::Long, PttEffect::None),
    }
}

/// What the dictation worker should do. Pure function, no OS calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureCommand {
    Start,
    Stop,
}

/// Chord tracker for Ctrl+B. Pure so the Windows hook stays thin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChordState {
    pub ctrl: bool,
    pub chord: bool,
}

/// Returns (next state, optional edge, whether to swallow the key).
/// Only the B key is swallowed, and only while it is part of Ctrl+B.
pub fn apply_ctrl_b(
    mut state: ChordState,
    is_ctrl: bool,
    is_b: bool,
    down: bool,
    up: bool,
) -> (ChordState, Option<HotkeyAction>, bool) {
    if is_ctrl {
        if down {
            state.ctrl = true;
        }
        if up {
            state.ctrl = false;
            if state.chord {
                state.chord = false;
                return (state, Some(HotkeyAction::Release), false);
            }
        }
        return (state, None, false);
    }
    if is_b {
        if down && state.ctrl {
            if !state.chord {
                state.chord = true;
                return (state, Some(HotkeyAction::Press), true);
            }
            return (state, None, true);
        }
        if up && state.chord {
            state.chord = false;
            return (state, Some(HotkeyAction::Release), true);
        }
    }
    (state, None, false)
}

pub fn command_for_hotkey(
    mode: HotkeyMode,
    capturing: bool,
    action: HotkeyAction,
) -> Option<CaptureCommand> {
    match mode {
        HotkeyMode::PushToTalk => match action {
            HotkeyAction::Press if !capturing => Some(CaptureCommand::Start),
            HotkeyAction::Release if capturing => Some(CaptureCommand::Stop),
            HotkeyAction::StopNow if capturing => Some(CaptureCommand::Stop),
            _ => None,
        },
        HotkeyMode::Toggle => match (capturing, action) {
            (false, HotkeyAction::Press) => Some(CaptureCommand::Start),
            (true, HotkeyAction::Press) => Some(CaptureCommand::Stop),
            (true, HotkeyAction::StopNow) => Some(CaptureCommand::Stop),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_to_talk_starts_on_press_stops_on_release() {
        assert_eq!(
            command_for_hotkey(HotkeyMode::PushToTalk, false, HotkeyAction::Press),
            Some(CaptureCommand::Start)
        );
        assert_eq!(
            command_for_hotkey(HotkeyMode::PushToTalk, true, HotkeyAction::Release),
            Some(CaptureCommand::Stop)
        );
        assert_eq!(
            command_for_hotkey(HotkeyMode::PushToTalk, false, HotkeyAction::Release),
            None
        );
        assert_eq!(
            command_for_hotkey(HotkeyMode::PushToTalk, true, HotkeyAction::Press),
            None
        );
    }

    #[test]
    fn toggle_ignores_release() {
        assert_eq!(
            command_for_hotkey(HotkeyMode::Toggle, false, HotkeyAction::Press),
            Some(CaptureCommand::Start)
        );
        assert_eq!(
            command_for_hotkey(HotkeyMode::Toggle, true, HotkeyAction::Press),
            Some(CaptureCommand::Stop)
        );
        assert_eq!(
            command_for_hotkey(HotkeyMode::Toggle, true, HotkeyAction::Release),
            None
        );
    }

    #[test]
    fn ctrl_b_press_and_release() {
        let mut s = ChordState::default();
        let (next, action, swallow) = apply_ctrl_b(s, true, false, true, false);
        s = next;
        assert!(s.ctrl && action.is_none() && !swallow);

        let (next, action, swallow) = apply_ctrl_b(s, false, true, true, false);
        s = next;
        assert_eq!(action, Some(HotkeyAction::Press));
        assert!(swallow && s.chord);

        let (_, action, swallow) = apply_ctrl_b(s, false, true, false, true);
        assert_eq!(action, Some(HotkeyAction::Release));
        assert!(swallow);
    }

    #[test]
    fn b_without_ctrl_is_ignored() {
        let s = ChordState::default();
        let (next, action, swallow) = apply_ctrl_b(s, false, true, true, false);
        assert_eq!(next, s);
        assert!(action.is_none() && !swallow);
    }

    #[test]
    fn ctrl_up_ends_active_chord() {
        let s = ChordState {
            ctrl: true,
            chord: true,
        };
        let (next, action, swallow) = apply_ctrl_b(s, true, false, false, true);
        assert!(!next.ctrl && !next.chord);
        assert_eq!(action, Some(HotkeyAction::Release));
        assert!(!swallow);
    }

    #[test]
    fn short_hold_stops_on_release() {
        let (mode, effect) = decide_ptt(ListenMode::Off, HotkeyAction::Press, 0);
        assert_eq!((mode, effect), (ListenMode::Holding, PttEffect::Start));
        let (mode, effect) = decide_ptt(mode, HotkeyAction::Release, SHORT_HOLD_MS + 10);
        assert_eq!((mode, effect), (ListenMode::Off, PttEffect::Stop));
    }

    #[test]
    fn quick_double_tap_enters_long_listen() {
        let (mode, _) = decide_ptt(ListenMode::Off, HotkeyAction::Press, 0);
        let (mode, effect) = decide_ptt(mode, HotkeyAction::Release, 80);
        assert_eq!((mode, effect), (ListenMode::AwaitingSecondTap, PttEffect::None));
        let (mode, effect) = decide_ptt(mode, HotkeyAction::Press, 200);
        assert_eq!((mode, effect), (ListenMode::Long, PttEffect::GoLong));
        let (mode, effect) = decide_ptt(mode, HotkeyAction::Release, 250);
        assert_eq!((mode, effect), (ListenMode::Long, PttEffect::None));
        let (mode, effect) = decide_ptt(mode, HotkeyAction::StopNow, 4000);
        assert_eq!((mode, effect), (ListenMode::Off, PttEffect::Stop));
    }

    #[test]
    fn missed_second_tap_stops() {
        let (mode, _) = decide_ptt(ListenMode::Off, HotkeyAction::Press, 0);
        let (mode, _) = decide_ptt(mode, HotkeyAction::Release, 50);
        let (mode, effect) = decide_ptt(mode, HotkeyAction::StopNow, DOUBLE_TAP_MS);
        assert_eq!((mode, effect), (ListenMode::Off, PttEffect::Stop));
    }
}
