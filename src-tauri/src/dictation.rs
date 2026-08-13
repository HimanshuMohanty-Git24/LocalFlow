//! Dictation worker. Owns CPAL sessions on one thread (`Stream` is not Send).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

static OVERLAY_GEN: AtomicU64 = AtomicU64::new(0);

use tauri::{AppHandle, Emitter, Manager};

use crate::asr::{SpeechRecognizer, TranscriptionOptions, WhisperBackend};
use crate::audio::capture::{max_capture_seconds, CaptureSession};
use crate::audio::resample::{to_whisper_mono16k, WHISPER_RATE};
use crate::audio::MicTestResult;
use crate::config::Settings;
use crate::errors::AppError;
use crate::config::HotkeyMode;
use crate::hotkey::{
    arm_long_listen, command_for_hotkey, decide_ptt, disarm_long_listen, CaptureCommand,
    HotkeyAction, ListenMode, PttEffect, DOUBLE_TAP_MS,
};
use crate::state::events::FlowEvent;
use crate::state::StateMachine;
use crate::llm::QwenBackend;
use crate::vad::SileroVad;

#[derive(Clone)]
pub struct DictationTx(pub SyncSender<WorkerMsg>);

/// Shown on the dashboard. Never includes transcript text.
#[derive(Debug, Clone)]
pub struct AsrStatus {
    pub label: String,
}

impl Default for AsrStatus {
    fn default() -> Self {
        Self {
            label: "Not loaded".to_string(),
        }
    }
}

pub enum WorkerMsg {
    Hotkey(HotkeyAction),
    DoubleTapExpired(u64),
    UiLong,
    UiShortStart,
    UiShortStop,
    Test {
        mic_id: Option<String>,
        reply: mpsc::Sender<Result<MicTestResult, AppError>>,
    },
}

pub fn spawn_worker(app: AppHandle) -> Result<DictationTx, AppError> {
    let (tx, rx) = mpsc::sync_channel(16);
    let timeout_tx = tx.clone();
    thread::Builder::new()
        .name("localflow-dictation".into())
        .spawn(move || run(app, rx, timeout_tx))
        .map_err(|e| AppError::message(e.to_string()))?;
    Ok(DictationTx(tx))
}

struct Worker {
    app: AppHandle,
    session: Option<CaptureSession>,
    asr: WhisperBackend,
    vad: SileroVad,
    llm: QwenBackend,
    listen: ListenMode,
    started: Instant,
    armed_press: Option<Instant>,
    gen: u64,
    timeout_tx: SyncSender<WorkerMsg>,
}

fn run(app: AppHandle, rx: Receiver<WorkerMsg>, timeout_tx: SyncSender<WorkerMsg>) {
    let mut worker = Worker {
        app,
        session: None,
        asr: WhisperBackend::new(),
        vad: SileroVad::new(),
        llm: QwenBackend::new(),
        listen: ListenMode::Off,
        started: Instant::now(),
        armed_press: None,
        gen: 0,
        timeout_tx,
    };
    for msg in rx {
        match msg {
            WorkerMsg::Hotkey(action) => {
                if let Err(err) = handle_hotkey(&mut worker, action) {
                    tracing::warn!(error = %err, "hotkey action failed");
                }
            }
            WorkerMsg::DoubleTapExpired(gen) => {
                if gen == worker.gen && worker.listen == ListenMode::AwaitingSecondTap {
                    if let Err(err) = handle_hotkey(&mut worker, HotkeyAction::StopNow) {
                        tracing::warn!(error = %err, "double-tap timeout failed");
                    }
                }
            }
            WorkerMsg::UiLong => {
                if let Err(err) = start_long(&mut worker) {
                    tracing::warn!(error = %err, "long listen failed");
                }
            }
            WorkerMsg::UiShortStart => {
                if worker.session.is_none() {
                    worker.listen = ListenMode::Holding;
                    worker.started = Instant::now();
                    if let Err(err) = start_capture(&mut worker, "short") {
                        tracing::warn!(error = %err, "short listen failed");
                    }
                }
            }
            WorkerMsg::UiShortStop => {
                if let Err(err) = finish(&mut worker) {
                    tracing::warn!(error = %err, "short listen stop failed");
                }
            }
            WorkerMsg::Test { mic_id, reply } => {
                let _ = reply.send(run_test(&worker.app, &mut worker.session, mic_id.as_deref()));
            }
        }
    }
}

fn handle_hotkey(worker: &mut Worker, action: HotkeyAction) -> Result<(), AppError> {
    let settings = worker.app.state::<Mutex<Settings>>();
    let mode = settings
        .lock()
        .map_err(|_| AppError::LockPoisoned)?
        .hotkey_mode;
    if mode == HotkeyMode::Toggle {
        let capturing = worker.session.is_some();
        return match command_for_hotkey(mode, capturing, action) {
            Some(CaptureCommand::Start) => start_capture(worker, "short"),
            Some(CaptureCommand::Stop) => finish(worker),
            None => Ok(()),
        };
    }

    if action == HotkeyAction::Press {
        let second = worker
            .armed_press
            .map(|t| t.elapsed().as_millis() < DOUBLE_TAP_MS)
            .unwrap_or(false);
        worker.armed_press = Some(Instant::now());
        if second {
            tracing::info!("second Ctrl+B within window; starting long listen");
            return start_long(worker);
        }
    }

    let held = worker
        .session
        .as_ref()
        .map(|s| s.elapsed_ms() as u128)
        .unwrap_or(0);
    let (next, effect) = decide_ptt(worker.listen, action, held);
    worker.listen = next;
    match effect {
        PttEffect::Start => {
            worker.started = Instant::now();
            if worker.session.is_some() {
                return enter_long(worker);
            }
            start_capture(worker, "short")
        }
        PttEffect::GoLong => enter_long(worker),
        PttEffect::Stop => finish(worker),
        PttEffect::None => {
            if worker.listen == ListenMode::AwaitingSecondTap && action == HotkeyAction::Release {
                worker.gen = worker.gen.wrapping_add(1);
                let gen = worker.gen;
                let tx = worker.timeout_tx.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(DOUBLE_TAP_MS as u64));
                    let _ = tx.try_send(WorkerMsg::DoubleTapExpired(gen));
                });
                let _ = worker.app.emit("listen-mode", "awaiting");
            }
            Ok(())
        }
    }
}

fn enter_long(worker: &mut Worker) -> Result<(), AppError> {
    worker.listen = ListenMode::Long;
    worker.gen = worker.gen.wrapping_add(1);
    arm_long_listen();
    let _ = worker.app.emit("listen-mode", "long");
    tracing::info!("long-listen; Space or Esc stops");
    Ok(())
}

fn start_long(worker: &mut Worker) -> Result<(), AppError> {
    if worker.session.is_some() {
        return enter_long(worker);
    }
    worker.listen = ListenMode::Long;
    worker.started = Instant::now();
    start_capture(worker, "long")?;
    enter_long(worker)
}

fn finish(worker: &mut Worker) -> Result<(), AppError> {
    disarm_long_listen();
    worker.listen = ListenMode::Off;
    worker.gen = worker.gen.wrapping_add(1);
    let real_take = worker
        .session
        .as_ref()
        .map(|s| s.elapsed_ms() >= 600)
        .unwrap_or(false);
    if real_take {
        worker.armed_press = None;
    }
    if worker.session.is_none() {
        return Ok(());
    }
    stop_and_transcribe(
        &worker.app,
        &mut worker.session,
        &mut worker.asr,
        &mut worker.vad,
        &mut worker.llm,
    )?;
    Ok(())
}

fn start_capture(worker: &mut Worker, listen_mode: &str) -> Result<(), AppError> {
    let app = &worker.app;
    let settings = app.state::<Mutex<Settings>>();
    let mic_id = settings
        .lock()
        .map_err(|_| AppError::LockPoisoned)?
        .microphone_id
        .clone();
    apply(app, FlowEvent::HotkeyPressed)?;
    let preferred = if mic_id.is_empty() {
        None
    } else {
        Some(mic_id)
    };
    match CaptureSession::start(preferred.as_deref(), max_capture_seconds()) {
        Ok(started) => {
            worker.session = Some(started);
            apply(app, FlowEvent::RecordingStarted)?;
            set_tray_tooltip(app, "LocalFlow Listening");
            OVERLAY_GEN.fetch_add(1, Ordering::SeqCst);
            crate::overlay::show(app);
            let _ = app.emit("listen-mode", listen_mode);
            let _ = app.emit("transcript-reset", true);
            tracing::info!(mode = listen_mode, "capture listening");
            Ok(())
        }
        Err(err) => {
            worker.listen = ListenMode::Off;
            disarm_long_listen();
            apply(app, FlowEvent::Error)?;
            crate::overlay::hide(app);
            let _ = app.emit("flow-error", err.to_string());
            Err(err)
        }
    }
}

fn stop_and_transcribe(
    app: &AppHandle,
    session: &mut Option<CaptureSession>,
    asr: &mut WhisperBackend,
    vad: &mut SileroVad,
    llm: &mut QwenBackend,
) -> Result<MicTestResult, AppError> {
    let current = session
        .take()
        .ok_or_else(|| AppError::message("capture session missing"))?;
    let audio = match current.stop() {
        Ok(audio) if audio.duration_ms < 80 => {
            cancel_quiet(app);
            return Ok(MicTestResult {
                path: String::new(),
                duration_ms: 0,
                sample_rate: 0,
                channels: 0,
                frames: 0,
            });
        }
        Ok(audio) => audio,
        Err(err) if err.to_string().contains("no audio captured") => {
            cancel_quiet(app);
            return Ok(MicTestResult {
                path: String::new(),
                duration_ms: 0,
                sample_rate: 0,
                channels: 0,
                frames: 0,
            });
        }
        Err(err) => {
            fail_session(app, err.clone());
            return Err(err);
        }
    };

    let wav = match audio.save_wav() {
        Ok(wav) => wav,
        Err(err) => {
            fail_session(app, err.clone());
            return Err(err);
        }
    };
    let _ = app.emit("recording-finished", wav.clone());

    let pcm = to_whisper_mono16k(&audio.samples, audio.sample_rate, audio.channels);
    let speech = match vad.extract_speech(&pcm, WHISPER_RATE) {
        Ok(speech) => speech,
        Err(err) => {
            fail_session(app, err.clone());
            return Err(err);
        }
    };
    let Some(pcm) = speech else {
        tracing::info!("vad: no speech");
        apply(app, FlowEvent::RecordingStopped)?;
        set_tray_tooltip(app, "LocalFlow Ready");
        let _ = app.emit("no-speech", true);
        crate::overlay::hide(app);
        return Ok(wav);
    };

    apply(app, FlowEvent::SpeechStarted)?;
    apply(app, FlowEvent::HotkeyReleased)?;
    set_tray_tooltip(app, "LocalFlow Processing");

    if let Err(err) = asr.load() {
        fail_session(app, err.clone());
        return Err(err);
    }
    set_asr_label(app, asr.model_label());

    match asr.transcribe(&pcm, TranscriptionOptions::default()) {
        Ok(result) => {
            tracing::info!(asr_ms = result.duration_ms, "asr finished");
            let _ = app.emit("raw-transcript", result.text.clone());
            apply(app, FlowEvent::FinalTranscript)?;
            let mut cleaned = crate::normalization::clean(&result.text);
            cleaned = crate::vocabulary::Vocabulary::load().apply(&cleaned);
            if settings_llm(app)? && !cleaned.is_empty() {
                cleaned = llm.enhance(&cleaned);
                cleaned = crate::vocabulary::Vocabulary::load().apply(&cleaned);
            }
            cleaned = crate::normalization::drop_repeated_tail(&cleaned);
            let _ = app.emit("clean-transcript", cleaned.clone());
            apply(app, FlowEvent::NormalizationFinished)?;
            if cleaned.is_empty() {
                apply(app, FlowEvent::Reset)?;
                set_tray_tooltip(app, "LocalFlow Ready");
                schedule_overlay_hide(app.clone());
                return Ok(wav);
            }
            set_tray_tooltip(app, "LocalFlow Inserting");
            let preserve = settings_preserve(app)?;
            match crate::injection::inject(&cleaned, preserve) {
                Ok(()) => {
                    apply(app, FlowEvent::InjectionFinished)?;
                    set_tray_tooltip(app, "LocalFlow Ready");
                    tracing::info!("injection finished");
                    schedule_overlay_hide(app.clone());
                    Ok(wav)
                }
                Err(err) => {
                    fail_session(app, err.clone());
                    Err(err)
                }
            }
        }
        Err(err) => {
            fail_session(app, err.clone());
            Err(err)
        }
    }
}

fn settings_preserve(app: &AppHandle) -> Result<bool, AppError> {
    Ok(app
        .state::<Mutex<Settings>>()
        .lock()
        .map_err(|_| AppError::LockPoisoned)?
        .preserve_clipboard)
}

fn settings_llm(app: &AppHandle) -> Result<bool, AppError> {
    Ok(app
        .state::<Mutex<Settings>>()
        .lock()
        .map_err(|_| AppError::LockPoisoned)?
        .llm_enabled)
}

fn cancel_quiet(app: &AppHandle) {
    disarm_long_listen();
    let _ = apply(app, FlowEvent::RecordingStopped);
    let _ = apply(app, FlowEvent::Reset);
    set_tray_tooltip(app, "LocalFlow Ready");
    crate::overlay::hide(app);
    tracing::info!("capture cancelled (empty tap)");
}

fn fail_session(app: &AppHandle, err: AppError) {
    disarm_long_listen();
    let _ = apply(app, FlowEvent::Error);
    let _ = apply(app, FlowEvent::Reset);
    set_tray_tooltip(app, "LocalFlow Ready");
    crate::overlay::hide(app);
    let _ = app.emit("flow-error", err.to_string());
}

fn run_test(
    app: &AppHandle,
    session: &mut Option<CaptureSession>,
    preferred_id: Option<&str>,
) -> Result<MicTestResult, AppError> {
    if session.is_some() {
        return Err(AppError::AlreadyRecording);
    }
    apply(app, FlowEvent::HotkeyPressed)?;
    match CaptureSession::start(preferred_id, 6) {
        Ok(started) => {
            *session = Some(started);
            apply(app, FlowEvent::RecordingStarted)?;
            crate::overlay::show(app);
        }
        Err(err) => {
            apply(app, FlowEvent::Error)?;
            return Err(err);
        }
    }
    thread::sleep(Duration::from_secs(5));
    let current = session
        .take()
        .ok_or_else(|| AppError::message("capture session missing"))?;
    let audio = current.stop()?;
    apply(app, FlowEvent::RecordingStopped)?;
    crate::overlay::hide(app);
    set_tray_tooltip(app, "LocalFlow Ready");
    audio.save_wav()
}

fn apply(app: &AppHandle, event: FlowEvent) -> Result<(), AppError> {
    let machine_state = app.state::<Mutex<StateMachine>>();
    let mut machine = machine_state.lock().map_err(|_| AppError::LockPoisoned)?;
    let state = machine.apply(event)?;
    drop(machine);
    let _ = app.emit("flow-state", state);
    Ok(())
}

fn set_asr_label(app: &AppHandle, label: &str) {
    if let Ok(mut status) = app.state::<Mutex<AsrStatus>>().lock() {
        status.label = label.to_string();
    }
    let _ = app.emit("asr-model", label);
}

fn set_tray_tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("localflow") {
        if let Err(err) = tray.set_tooltip(Some(text)) {
            tracing::debug!(error = %err, "tray tooltip update failed");
        }
    }
}

fn schedule_overlay_hide(app: AppHandle) {
    let gen = OVERLAY_GEN.load(Ordering::SeqCst);
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(6));
        if OVERLAY_GEN.load(Ordering::SeqCst) == gen {
            crate::overlay::hide(&app);
        }
    });
}
