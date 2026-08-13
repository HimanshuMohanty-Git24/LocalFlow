//! Windows push-to-talk: WH_KEYBOARD_LL for Ctrl+B. Must stay off the audio thread.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::SyncSender;
use std::sync::{Mutex, OnceLock};
use std::thread::{self, JoinHandle};

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    VK_B, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RCONTROL, VK_RMENU,
    VK_RSHIFT, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::{
    apply_ctrl_b, long_listen_should_stop, ChordState, HotkeyAction,
};
use crate::errors::AppError;

static SENDER: OnceLock<Mutex<Option<SyncSender<HotkeyAction>>>> = OnceLock::new();
static CHORD: Mutex<ChordState> = Mutex::new(ChordState {
    ctrl: false,
    chord: false,
});
static HOOK_THREAD: AtomicU32 = AtomicU32::new(0);

pub struct WindowsHook {
    thread: Option<JoinHandle<()>>,
}

pub fn install(tx: SyncSender<HotkeyAction>) -> Result<WindowsHook, AppError> {
    let slot = SENDER.get_or_init(|| Mutex::new(None));
    *slot.lock().map_err(|_| AppError::LockPoisoned)? = Some(tx);

    let thread = thread::Builder::new()
        .name("localflow-hotkey".into())
        .spawn(hook_thread)
        .map_err(|e| AppError::message(e.to_string()))?;

    Ok(WindowsHook {
        thread: Some(thread),
    })
}

fn hook_thread() {
    HOOK_THREAD.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
    let hook = unsafe {
        SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            None,
            0,
        )
    };
    let Ok(hook) = hook else {
        tracing::error!("SetWindowsHookExW failed");
        return;
    };

    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let _ = UnhookWindowsHookEx(hook);
    }
    if let Ok(mut state) = CHORD.lock() {
        *state = ChordState::default();
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code == HC_ACTION as i32 && lparam.0 != 0 {
        let kb = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let vk = kb.vkCode;
        let up = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;
        let down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
        let injected = kb.flags.0 & 0x10 != 0;
        if down && !injected && long_listen_should_stop() && !is_hold_key(vk) {
            send(HotkeyAction::StopNow);
            return LRESULT(1);
        }
        let is_ctrl = vk == u32::from(VK_CONTROL.0)
            || vk == u32::from(VK_LCONTROL.0)
            || vk == u32::from(VK_RCONTROL.0);
        let is_b = vk == u32::from(VK_B.0);
        if is_ctrl || is_b {
            if let Ok(mut state) = CHORD.try_lock() {
                let (next, action, swallow) = apply_ctrl_b(*state, is_ctrl, is_b, down, up);
                *state = next;
                if let Some(action) = action {
                    send(action);
                }
                if swallow {
                    return LRESULT(1);
                }
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn is_hold_key(vk: u32) -> bool {
    vk == u32::from(VK_CONTROL.0)
        || vk == u32::from(VK_LCONTROL.0)
        || vk == u32::from(VK_RCONTROL.0)
        || vk == u32::from(VK_SHIFT.0)
        || vk == u32::from(VK_LSHIFT.0)
        || vk == u32::from(VK_RSHIFT.0)
        || vk == u32::from(VK_MENU.0)
        || vk == u32::from(VK_LMENU.0)
        || vk == u32::from(VK_RMENU.0)
        || vk == u32::from(VK_LWIN.0)
        || vk == u32::from(VK_RWIN.0)
        || vk == u32::from(VK_B.0)
}

fn send(action: HotkeyAction) {
    if let Some(slot) = SENDER.get() {
        if let Ok(guard) = slot.lock() {
            if let Some(tx) = guard.as_ref() {
                let _ = tx.try_send(action);
            }
        }
    }
}

impl Drop for WindowsHook {
    fn drop(&mut self) {
        let tid = HOOK_THREAD.load(Ordering::SeqCst);
        if tid != 0 {
            unsafe {
                let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        if let Some(slot) = SENDER.get() {
            if let Ok(mut guard) = slot.lock() {
                *guard = None;
            }
        }
    }
}
