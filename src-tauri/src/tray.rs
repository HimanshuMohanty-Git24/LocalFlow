//! System tray. Dictation must keep working if the settings window is closed.

use crate::errors::AppError;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

const MAIN_WINDOW: &str = "main";

/// Shows the settings/dashboard window without stealing it from a future overlay.
pub fn show_main_window(app: &AppHandle) -> Result<(), AppError> {
    let window = app
        .get_webview_window(MAIN_WINDOW)
        .ok_or_else(|| AppError::WindowMissing(MAIN_WINDOW.to_string()))?;
    window
        .show()
        .map_err(|e| AppError::message(e.to_string()))?;
    window
        .unminimize()
        .map_err(|e| AppError::message(e.to_string()))?;
    window
        .set_focus()
        .map_err(|e| AppError::message(e.to_string()))?;
    Ok(())
}

/// Creates the tray icon and menu. Uses the bundled app icon.
pub fn setup(app: &AppHandle) -> Result<(), AppError> {
    let open = MenuItem::with_id(app, "open", "Open LocalFlow", true, None::<&str>)
        .map_err(|e| AppError::message(e.to_string()))?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| AppError::message(e.to_string()))?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| AppError::message(e.to_string()))?;
    let menu = Menu::with_items(app, &[&open, &settings, &quit])
        .map_err(|e| AppError::message(e.to_string()))?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or(AppError::TrayIconMissing)?;

    TrayIconBuilder::with_id("localflow")
        .icon(icon)
        .tooltip("LocalFlow Ready")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                tracing::info!("tray quit");
                app.exit(0);
            }
            "open" => {
                if let Err(err) = show_main_window(app) {
                    tracing::error!(error = %err, "failed to open window");
                }
            }
            "settings" => {
                if let Err(err) = show_main_window(app) {
                    tracing::error!(error = %err, "failed to open settings");
                }
                if let Err(err) = app.emit("navigate", "settings") {
                    tracing::error!(error = %err, "failed to emit navigate");
                }
            }
            id => tracing::debug!(id, "unhandled tray menu id"),
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                if let Err(err) = show_main_window(tray.app_handle()) {
                    tracing::error!(error = %err, "failed to open window from tray");
                }
            }
        })
        .build(app)
        .map_err(|e| AppError::message(e.to_string()))?;

    tracing::info!("tray ready");
    Ok(())
}
