//! Starts the platform hotkey source. Windows: Ctrl+B low-level hook.

use std::sync::mpsc::{self, Receiver};

use super::HotkeyAction;
use crate::errors::AppError;

/// Handle that unhooks and joins the listener thread on drop.
pub struct HotkeyGuard {
    inner: Option<PlatformGuard>,
}

struct PlatformGuard {
    #[cfg(windows)]
    _hook: super::windows::WindowsHook,
}

/// Spawns a background listener. Events are Ctrl+B press/release on Windows.
pub fn spawn() -> Result<(Receiver<HotkeyAction>, HotkeyGuard), AppError> {
    let (tx, rx) = mpsc::sync_channel(8);
    #[cfg(windows)]
    {
        let hook = super::windows::install(tx)?;
        tracing::info!("hotkey listener ready (Ctrl+B)");
        Ok((
            rx,
            HotkeyGuard {
                inner: Some(PlatformGuard { _hook: hook }),
            },
        ))
    }
    #[cfg(not(windows))]
    {
        let _ = tx;
        Err(AppError::HotkeyUnavailable)
    }
}

impl Drop for HotkeyGuard {
    fn drop(&mut self) {
        self.inner.take();
    }
}
