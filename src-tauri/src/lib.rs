//! LocalFlow desktop backend. Phase 7: optional local Qwen cleanup.

mod asr;
mod audio;
mod config;
mod dictation;
mod errors;
mod hotkey;
mod injection;
mod llm;
mod logging;
mod normalization;
mod overlay;
mod state;
mod tray;
mod vad;
mod vocabulary;

pub use config::Settings;
pub use errors::AppError;
pub use state::events::FlowEvent;
pub use state::{AppState, StateMachine};

use std::sync::Mutex;

use serde::Serialize;
use tauri::Manager;

use dictation::{AsrStatus, DictationTx, WorkerMsg};

/// Snapshot shown on the dashboard. No transcripts.
#[derive(Debug, Clone, Serialize)]
pub struct AppStatus {
    pub state: AppState,
    pub microphone: String,
    pub asr_model: String,
    pub llm_enabled: bool,
    pub hotkey: String,
    pub offline: bool,
}

fn lock_poisoned<T>(_: std::sync::PoisonError<T>) -> AppError {
    AppError::LockPoisoned
}

#[tauri::command]
fn get_status(
    settings: tauri::State<'_, Mutex<Settings>>,
    machine: tauri::State<'_, Mutex<StateMachine>>,
    asr: tauri::State<'_, Mutex<AsrStatus>>,
) -> Result<AppStatus, AppError> {
    let settings = settings.lock().map_err(lock_poisoned)?;
    let machine = machine.lock().map_err(lock_poisoned)?;
    let asr = asr.lock().map_err(lock_poisoned)?;
    Ok(AppStatus {
        state: machine.state(),
        microphone: audio::device::status_label(&settings.microphone_id),
        asr_model: asr.label.clone(),
        llm_enabled: settings.llm_enabled,
        hotkey: settings.dictation_hotkey.clone(),
        offline: true,
    })
}

#[tauri::command]
fn get_settings(settings: tauri::State<'_, Mutex<Settings>>) -> Result<Settings, AppError> {
    settings.lock().map(|s| s.clone()).map_err(lock_poisoned)
}

#[tauri::command]
fn update_settings(
    next: Settings,
    settings: tauri::State<'_, Mutex<Settings>>,
) -> Result<Settings, AppError> {
    let mut current = settings.lock().map_err(lock_poisoned)?;
    *current = next;
    tracing::info!(
        hotkey_mode = ?current.hotkey_mode,
        has_microphone_id = !current.microphone_id.is_empty(),
        start_on_login = current.start_on_login,
        preserve_clipboard = current.preserve_clipboard,
        save_text_history = current.save_text_history,
        llm_enabled = current.llm_enabled,
        "settings updated"
    );
    Ok(current.clone())
}

#[tauri::command]
fn list_microphones() -> Result<Vec<audio::Microphone>, AppError> {
    audio::list_microphones()
}

#[tauri::command]
async fn record_microphone_test(app: tauri::AppHandle) -> Result<audio::MicTestResult, AppError> {
    let id = app
        .state::<Mutex<Settings>>()
        .lock()
        .map_err(lock_poisoned)?
        .microphone_id
        .clone();
    let preferred = if id.is_empty() { None } else { Some(id) };
    let tx = app.state::<DictationTx>().0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        tx.send(WorkerMsg::Test {
            mic_id: preferred,
            reply: reply_tx,
        })
        .map_err(|e| AppError::message(e.to_string()))?;
        reply_rx
            .recv()
            .map_err(|e| AppError::message(e.to_string()))?
    })
    .await
    .map_err(|e| AppError::message(e.to_string()))?
}

#[tauri::command]
fn start_long_listen(app: tauri::AppHandle) -> Result<(), AppError> {
    let tx = app.state::<DictationTx>().0.clone();
    tx.send(WorkerMsg::UiLong)
        .map_err(|e| AppError::message(e.to_string()))
}

#[tauri::command]
fn start_short_listen(app: tauri::AppHandle) -> Result<(), AppError> {
    let tx = app.state::<DictationTx>().0.clone();
    tx.send(WorkerMsg::UiShortStart)
        .map_err(|e| AppError::message(e.to_string()))
}

#[tauri::command]
fn stop_short_listen(app: tauri::AppHandle) -> Result<(), AppError> {
    let tx = app.state::<DictationTx>().0.clone();
    tx.send(WorkerMsg::UiShortStop)
        .map_err(|e| AppError::message(e.to_string()))
}

#[tauri::command]
fn reveal_recording(path: String) -> Result<(), AppError> {
    audio::capture::reveal_path(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::init();
    tracing::info!("LocalFlow starting (phase 7)");

    let result = tauri::Builder::default()
        .manage(Mutex::new(Settings::default()))
        .manage(Mutex::new(StateMachine::new()))
        .manage(Mutex::new(AsrStatus::default()))
        .invoke_handler(tauri::generate_handler![
            get_status,
            get_settings,
            update_settings,
            list_microphones,
            record_microphone_test,
            reveal_recording,
            start_long_listen,
            start_short_listen,
            stop_short_listen
        ])
        .setup(|app| {
            if let Err(err) = tray::setup(app.handle()) {
                tracing::error!(error = %err, "tray setup failed");
                return Err(err.into());
            }

            if let Err(err) = overlay::prepare(app.handle()) {
                tracing::warn!(error = %err, "overlay prepare failed");
            }

            let mailbox = dictation::spawn_worker(app.handle().clone())?;
            match hotkey::manager::spawn() {
                Ok((rx, guard)) => {
                    app.manage(Mutex::new(Some(guard)));
                    let tx = mailbox.0.clone();
                    std::thread::Builder::new()
                        .name("localflow-hotkey-fwd".into())
                        .spawn(move || {
                            for action in rx {
                                if tx.send(WorkerMsg::Hotkey(action)).is_err() {
                                    break;
                                }
                            }
                        })
                        .map_err(|e| e.to_string())?;
                }
                Err(err) => {
                    tracing::error!(error = %err, "hotkey listener unavailable");
                }
            }
            app.manage(mailbox);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(err) = window.hide() {
                    tracing::error!(error = %err, "failed to hide window");
                } else {
                    tracing::info!("window hidden; tray still running");
                }
            }
        })
        .run(tauri::generate_context!());

    if let Err(err) = result {
        tracing::error!(error = %err, "tauri exited with error");
        std::process::exit(1);
    }
}
