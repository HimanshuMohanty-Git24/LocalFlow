# Architecture

LocalFlow is a local desktop program. The UI never runs inference. The core
never contacts a network service.

## Layers

```text
Desktop UI (React + TypeScript + Tauri 2)
        │
        ▼
Rust core (hotkeys, audio, VAD, state, injection, local JSON settings)
        │
        ├── whisper.cpp (ASR)
        └── llama.cpp + Qwen3-0.6B     — Phase 7
                │
                ▼
        Deterministic rule processor
                │
                ▼
        Text injection into the focused app
```

## Phase 7 (current)

Implemented:

- Optional Qwen3-0.6B via llama.cpp (CPU) after rule cleanup
- Settings toggle **AI cleanup with local Qwen**; default on
- Missing GGUF skips the LLM and pastes rule-cleaned text
- Thinking tags stripped; runaway rewrites discarded
- llama.cpp logs disabled so prompts are not printed
- Settings persist in `%LOCALAPPDATA%\LocalFlow\settings.json`
- Optional text history and audio storage are off by default
- Windows NSIS installer bundles Whisper and Silero models

## Phase 6

Implemented:

- Silero VAD (whisper.cpp ggml) after capture, off the audio callback
- Leading/trailing silence trimmed; no-speech skips Whisper and paste
- If the Silero file is missing, full audio is transcribed (no crash)

Not implemented yet: llama.cpp / Qwen, SQLite, vocabulary.

## Phase 5

Implemented:

- Deterministic cleanup before paste: fillers (`um`/`uh`), spoken
  punctuation (`period`, `question mark`), `new line` / `new paragraph`
- Sentence capitalization and a trailing period when missing
- Dashboard shows inserted text vs raw Whisper when they differ
- Qwen / llama.cpp still off

Not implemented yet: VAD, llama.cpp, SQLite, vocabulary.

## Phase 4

Implemented:

- After Whisper, paste into the focused window (clipboard Ctrl+V)
- Unicode `SendInput` if paste fails
- `preserve_clipboard` restores the previous Unicode clipboard
- If both methods fail, text stays on the clipboard and the UI shows an error
- No LLM cleanup yet (normalization is a no-op)

Not implemented yet at that point: VAD, llama.cpp, SQLite.

## Phase 3

Implemented:

- CPU whisper.cpp via `whisper-rs` on the dictation worker thread
- Resample captured PCM to 16 kHz mono
- Local ggml lookup (`models/`, `LOCALFLOW_MODELS_DIR`); never downloaded
- `raw-transcript` event to the dashboard and overlay (not logs)
- Missing model is a recoverable error

Not implemented yet at that point: injection (now Phase 4).

## Phase 2

Implemented:

- Global Ctrl+B push-to-talk on Windows (`WH_KEYBOARD_LL`)
- **Short:** hold Ctrl+B to record, release B or Ctrl to stop
- **Long:** tap Ctrl+B twice (B twice while Ctrl is down); recording continues
  until Space or Esc
- Capture ring is 180 seconds; VAD only skips silence, it does not crop the take
- Toggle mode uses the same chord (press to start, press to stop)
- Only the B key is swallowed during the chord (Ctrl alone still works)
- Floating click-through overlay (never focused) with a hover animation
- Capture start/stop session (up to 180s ring); dictation stays in memory unless
  audio saving is enabled
- OS hook lives in `hotkey/windows.rs`; chord policy in `apply_ctrl_b`

## Phase 1

Implemented:

- CPAL microphone enumeration (WASAPI on Windows)
- Device selection in Settings (`microphone_id`, empty = system default)
- 5-second test capture into a preallocated ring buffer
- 16-bit PCM WAV written to the temp directory
- Fallback: selected mic → default mic → error

Not implemented yet: VAD, Whisper, llama.cpp, SQLite, injection, overlay.

## Phase 0

Implemented:

- Tauri 2 + React + TypeScript settings/dashboard window
- System tray (Open, Settings, Quit)
- Close-to-tray (window hide, process stays running)
- `tracing` logs via `LOCALFLOW_LOG` (no transcript fields)
- Initially in-memory settings (`preserve_clipboard` default on,
  history/audio off); current builds persist them locally
- Dictation `StateMachine` with typed `FlowEvent`s

## Modules

| Path | Role |
| --- | --- |
| `src-tauri/src/logging.rs` | `tracing` subscriber |
| `src-tauri/src/tray.rs` | Tray icon and menu |
| `src-tauri/src/config/` | Settings types, local persistence, startup registration |
| `src-tauri/src/state/` | `AppState` + `FlowEvent` machine |
| `src-tauri/src/errors.rs` | `AppError` |
| `src-tauri/src/audio/device.rs` | Microphone list + resolve |
| `src-tauri/src/audio/capture.rs` | CPAL capture (no inference in callback) |
| `src-tauri/src/audio/ring_buffer.rs` | Preallocated f32 ring |
| `src-tauri/src/audio/wav.rs` | PCM16 WAV writer |
| `src-tauri/src/hotkey/` | Global PTT policy + Windows Ctrl+B hook |
| `src-tauri/src/overlay.rs` | Click-through listening HUD |
| `src-tauri/src/audio/resample.rs` | Downmix + 16 kHz for Whisper |
| `src-tauri/src/asr/` | `SpeechRecognizer` + Whisper backend |
| `src-tauri/src/injection/` | Paste / SendInput into the focused app |
| `src-tauri/src/normalization/` | Deterministic filler + punctuation rules |
| `src-tauri/src/vad/` | Silero VAD + speech crop |
| `src-tauri/src/llm/` | Optional Qwen3-0.6B rewrite |

## Contracts (to be filled in)

```text
SpeechRecognizer          — Phase 3 (WhisperBackend)
VoiceActivityDetector     — Phase 6 (Silero via whisper.cpp)
TextEnhancer              — Phase 5 rules + Phase 7 Qwen
TextInjector              — Phase 4 (clipboard + SendInput)
AppContextProvider
ModelManager
```

## Logging policy

Logs may include timings, device names, model ids, and error categories.
Logs must not include transcripts, audio, clipboard, dictionary entries, or
window contents.
