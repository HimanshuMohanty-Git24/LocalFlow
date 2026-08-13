//! Listening overlay. Must never steal focus from the dictation target.

use tauri::{Manager, PhysicalPosition, WebviewWindow};

use crate::errors::AppError;

const LABEL: &str = "overlay";

/// Click-through + always on top. Called once at startup.
pub fn prepare(app: &tauri::AppHandle) -> Result<(), AppError> {
    let Some(window) = app.get_webview_window(LABEL) else {
        return Ok(());
    };
    window
        .set_ignore_cursor_events(true)
        .map_err(|e| AppError::message(e.to_string()))?;
    window
        .set_always_on_top(true)
        .map_err(|e| AppError::message(e.to_string()))?;
    position_bottom_center(&window);
    Ok(())
}

/// Shows the overlay without focusing it.
pub fn show(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        let _ = window.set_ignore_cursor_events(true);
        position_bottom_center(&window);
        if let Err(err) = window.show() {
            tracing::debug!(error = %err, "overlay show failed");
        }
    }
}

pub fn hide(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(LABEL) {
        if let Err(err) = window.hide() {
            tracing::debug!(error = %err, "overlay hide failed");
        }
    }
}

fn position_bottom_center(window: &WebviewWindow) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };
    let screen = monitor.size();
    let origin = monitor.position();
    let Ok(size) = window.outer_size() else {
        return;
    };
    let margin = (96.0 * monitor.scale_factor()) as i32;
    let x = origin.x + (screen.width as i32 - size.width as i32) / 2;
    let y = origin.y + screen.height as i32 - size.height as i32 - margin;
    if let Err(err) = window.set_position(PhysicalPosition::new(x, y)) {
        tracing::debug!(error = %err, "overlay position failed");
    }
}
