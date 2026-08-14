//! Windows sign-in startup registration under the current user's Run key.

use crate::errors::AppError;

#[cfg(windows)]
pub fn set_enabled(enabled: bool) -> Result<(), AppError> {
    use windows::core::w;
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
        KEY_SET_VALUE, REG_SZ,
    };

    let command = if enabled {
        Some(startup_command()?)
    } else {
        None
    };
    let mut key = HKEY::default();
    let opened = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            Some(0),
            KEY_SET_VALUE,
            &mut key,
        )
    };
    if opened != ERROR_SUCCESS {
        return Err(AppError::message(
            "could not update Windows startup settings",
        ));
    }

    let result = if let Some(command) = command {
        let wide: Vec<u16> = command.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = unsafe {
            std::slice::from_raw_parts(wide.as_ptr().cast::<u8>(), wide.len() * size_of::<u16>())
        };
        unsafe { RegSetValueExW(key, w!("LocalFlow"), None, REG_SZ, Some(bytes)) }
    } else {
        unsafe { RegDeleteValueW(key, w!("LocalFlow")) }
    };
    unsafe {
        let _ = RegCloseKey(key);
    }

    if result == ERROR_SUCCESS || (!enabled && result == ERROR_FILE_NOT_FOUND) {
        Ok(())
    } else {
        Err(AppError::message(
            "could not update Windows startup settings",
        ))
    }
}

#[cfg(not(windows))]
pub fn set_enabled(_enabled: bool) -> Result<(), AppError> {
    Err(AppError::message(
        "launch at login is available on Windows only",
    ))
}

#[cfg(windows)]
fn startup_command() -> Result<String, AppError> {
    let exe = std::env::current_exe()
        .map_err(|_| AppError::message("could not locate the LocalFlow executable"))?;
    Ok(format!("\"{}\"", exe.display()))
}
