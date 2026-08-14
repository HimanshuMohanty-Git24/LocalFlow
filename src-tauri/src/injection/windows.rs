//! Clipboard paste, then Unicode SendInput. Runs on the dictation worker.

use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_CONTROL, VK_RETURN, VK_V,
};

const CF_UNICODETEXT: u32 = 13;

use super::to_wide_null;
use crate::errors::AppError;

pub fn inject(text: &str, preserve: bool) -> Result<(), AppError> {
    thread::sleep(Duration::from_millis(80));

    let previous = if preserve { read_unicode().ok() } else { None };

    let paste_ok = set_unicode(text).is_ok() && send_ctrl_v().is_ok();
    if paste_ok {
        thread::sleep(Duration::from_millis(120));
        if preserve {
            match previous {
                Some(old) => {
                    let _ = set_unicode(&old);
                }
                None => {
                    let _ = clear_clipboard();
                }
            }
        }
        tracing::info!("injected via clipboard paste");
        return Ok(());
    }

    if send_unicode(text).is_ok() {
        tracing::info!("injected via sendinput");
        return Ok(());
    }

    let _ = set_unicode(text);
    Err(AppError::InjectionFailed)
}

fn open_clipboard() -> Result<(), AppError> {
    for _ in 0..12 {
        if unsafe { OpenClipboard(None) }.is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err(AppError::InjectionFailed)
}

fn set_unicode(text: &str) -> Result<(), AppError> {
    let wide = to_wide_null(text);
    let bytes = wide.len().saturating_mul(2);
    open_clipboard()?;
    let result = (|| {
        unsafe {
            EmptyClipboard().map_err(|_| AppError::InjectionFailed)?;
            let hmem = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes)
                .map_err(|_| AppError::InjectionFailed)?;
            let ptr = GlobalLock(hmem);
            if ptr.is_null() {
                return Err(AppError::InjectionFailed);
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr.cast::<u16>(), wide.len());
            let _ = GlobalUnlock(hmem);
            SetClipboardData(CF_UNICODETEXT, Some(HANDLE(hmem.0)))
                .map_err(|_| AppError::InjectionFailed)?;
        }
        Ok(())
    })();
    unsafe {
        let _ = CloseClipboard();
    }
    result
}

fn read_unicode() -> Result<String, AppError> {
    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
            return Err(AppError::InjectionFailed);
        }
    }
    open_clipboard()?;
    let text = unsafe {
        let handle = GetClipboardData(CF_UNICODETEXT).map_err(|_| AppError::InjectionFailed)?;
        if handle.is_invalid() {
            let _ = CloseClipboard();
            return Err(AppError::InjectionFailed);
        }
        let ptr = GlobalLock(HGLOBAL(handle.0));
        if ptr.is_null() {
            let _ = CloseClipboard();
            return Err(AppError::InjectionFailed);
        }
        let slice = std::slice::from_raw_parts(ptr.cast::<u16>(), 32_768);
        let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
        let owned = String::from_utf16_lossy(&slice[..end]);
        let _ = GlobalUnlock(HGLOBAL(handle.0));
        let _ = CloseClipboard();
        owned
    };
    Ok(text)
}

fn clear_clipboard() -> Result<(), AppError> {
    open_clipboard()?;
    let ok = unsafe { EmptyClipboard() };
    unsafe {
        let _ = CloseClipboard();
    }
    ok.map_err(|_| AppError::InjectionFailed)
}

fn send_ctrl_v() -> Result<(), AppError> {
    let inputs = [
        key(VK_CONTROL, KEYBD_EVENT_FLAGS(0)),
        key(VK_V, KEYBD_EVENT_FLAGS(0)),
        key(VK_V, KEYEVENTF_KEYUP),
        key(VK_CONTROL, KEYEVENTF_KEYUP),
    ];
    send(&inputs)
}

fn send_unicode(text: &str) -> Result<(), AppError> {
    let mut inputs = Vec::new();
    for unit in text.encode_utf16() {
        if unit == b'\r' as u16 {
            continue;
        }
        if unit == b'\n' as u16 {
            inputs.push(key(VK_RETURN, KEYBD_EVENT_FLAGS(0)));
            inputs.push(key(VK_RETURN, KEYEVENTF_KEYUP));
            continue;
        }
        inputs.push(unicode(unit, false));
        inputs.push(unicode(unit, true));
    }
    if inputs.is_empty() {
        return Ok(());
    }
    send(&inputs)
}

fn key(vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn unicode(scan: u16, up: bool) -> INPUT {
    let flags = if up {
        KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
    } else {
        KEYEVENTF_UNICODE
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(inputs: &[INPUT]) -> Result<(), AppError> {
    let n = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if n as usize != inputs.len() {
        Err(AppError::InjectionFailed)
    } else {
        Ok(())
    }
}
