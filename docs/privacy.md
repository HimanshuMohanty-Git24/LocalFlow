# Privacy

LocalFlow is offline-first.

- No accounts
- No telemetry in Phase 0
- No cloud transcription
- No OpenAI APIs
- Audio is not stored unless the user later enables it (default off)
- Text history is off by default
- Closing the window does not keep the microphone open; capture stops
  on hotkey release
- Transcripts are shown in the UI only and are never written to logs

If a future analytics option is added, it must be explicit opt-in and must
never send audio, transcripts, clipboard, window content, or vocabulary.
