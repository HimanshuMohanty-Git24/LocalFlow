# Testing

## Phase 0

Automated:

```bash
npm run test:rust
```

Manual:

1. `npm run tauri dev`
2. Confirm the dashboard window opens
3. Open Settings, toggle a setting, restart, and confirm it was saved
4. Close the window — the process should remain in the tray
5. Tray → Open LocalFlow should show the window again
6. Tray → Settings should open the Settings page
7. Tray → Quit should exit
8. `$env:LOCALFLOW_LOG='debug'; npm run tauri dev` should print
   `state` / `tray` logs without any transcript text

ASR and Notepad injection are out of scope until later phases.

## Phase 1

Automated:

```bash
npm run test:rust
```

Covers the ring buffer, WAV header, settings, and state machine. Live
capture is not unit-tested (needs a real microphone).

Manual:

1. `npm run tauri dev`
2. Dashboard should show the default microphone name (not "Not configured")
3. Settings → pick a microphone → Save
4. Dashboard → **Test microphone**
5. Speak for 5 seconds
6. **Show WAV in Explorer** and play the file in a local player (Groove,
   VLC, foobar, etc.)
7. Unplug / disable the selected mic, test again — should fall back to the
   default or show an error, never contact a network
8. Logs may include sample rate and device host; they must not dump samples

## Phase 2

Automated: `command_for_hotkey` (push-to-talk vs toggle) and
`RecordingStopped → idle`.

Manual:

1. `npm run tauri dev`
2. Click into Notepad (or any other app) so LocalFlow is not focused
3. Hold **Ctrl+B**, speak, release (or release Ctrl)
4. For a longer take: tap **B** twice while holding Ctrl, speak, then press
   **Space** or **Esc**
5. A small floating overlay should hover near the bottom
   of the screen and must not steal Notepad’s focus
6. Dashboard status should go `listening` then `idle`
7. Overlay should disappear on release (or Space/Esc in long mode)
8. With **Save audio recordings** off, no dictation WAV is created
9. Enable it and repeat; a WAV appears under
   `%LOCALAPPDATA%\LocalFlow\recordings`
10. Play it — length should match the take, including pauses in the middle
11. Typing `b` without Ctrl still works; Ctrl+C still works
12. Settings → Toggle: Ctrl+B starts, Ctrl+B again stops

## Phase 3

Automated: resample (stereo downmix, 48 kHz → 16 kHz length) and model
filename ranking. A live Whisper run needs a local ggml file.

Manual:

1. Copy `ggml-tiny.en.bin` (or `base.en` / `small.en`) into `models/`
   — see [models.md](models.md). Do not let the app download it.
2. `npm run tauri dev`
3. Dashboard Model row says it is ready and will load on first use
4. Hold **Ctrl+B**, speak a short English sentence, release
5. Overlay shows Listening → Processing → the words
6. Dashboard **Last transcript** matches what you said (raw, uncleaned)
7. Logs include `asr finished` with duration; they must not print the text
8. Rename/remove the ggml file and try again — error **model file is missing**,
   app stays up

## Phase 4

Manual:

1. `npm run tauri dev`
2. Open Notepad and click in it so it has the caret
3. Hold **Ctrl+B**, speak a short sentence, release
4. The same words should appear in Notepad at the caret
5. Dashboard still shows **Last transcript**
6. Copy something else first with **Preserve clipboard** on — after
   dictation, Ctrl+V in another place should still be the original copy
7. An elevated (Run as administrator) Notepad may fail; the error should
   say the text is on the clipboard
8. Logs must not print the transcript

## Phase 5

Automated: filler drop, spoken punctuation, new paragraph, trailing period.

Manual:

1. Click Notepad, say **hello um there** — should paste `Hello there.`
2. Say **hello period how are you question mark** — `Hello. How are you?`
3. Dashboard **Inserted** is cleaned; **Raw Whisper** shows only if different
4. Logs must not print either string

## Phase 6

Automated: crop first-to-last speech span; 100 cs at 16 kHz is 1 second.

Manual:

1. Hold Ctrl+B, stay silent, release — dashboard **Inserted** shows
   `(no speech)`, nothing is pasted
2. Hold Ctrl+B, speak, release — text still pastes
3. Logs may include `vad finished` with a segment count, never audio

## Phase 7

Automated: strip `</think>` blocks; reject oversized rewrites.

Manual:

1. Settings → **AI cleanup with local Qwen** on
2. `Qwen3-0.6B-Q8_0.gguf` in `models/`
3. Dictate a slightly messy sentence; paste should be grammatical
4. Toggle Qwen off — paste is rules-only again
5. First run after enable may pause a few seconds while the GGUF loads
6. Logs may include `llm finished` with duration, never the text

