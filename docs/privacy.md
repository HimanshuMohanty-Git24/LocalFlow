# Privacy

LocalFlow is offline-first.

- No accounts
- No telemetry
- No cloud transcription
- No OpenAI APIs
- Dictation audio stays in memory and is discarded after processing by default
- If **Save audio recordings** is enabled, WAVs are written only to
  `%LOCALAPPDATA%\LocalFlow\recordings`
- Text history is off by default; if enabled, completed dictations are appended
  to `%LOCALAPPDATA%\LocalFlow\history.jsonl`
- Settings are stored locally in `%LOCALAPPDATA%\LocalFlow\settings.json`
- The 5-second microphone test always creates a temporary WAV so the user can
  play and inspect it
- Closing the window does not keep the microphone open; capture stops
  on hotkey release
- Transcripts are shown in the UI only and are never written to logs

If a future analytics option is added, it must be explicit opt-in and must
never send audio, transcripts, clipboard, window content, or vocabulary.
