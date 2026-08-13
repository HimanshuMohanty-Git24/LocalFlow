# Contributing

LocalFlow is built one phase at a time. Do not start a later phase until the
current phase compiles, tests, and can be verified by hand.

## Rules

- Production must work offline. Never add a cloud fallback.
- Do not log transcripts, audio, clipboard contents, or window text.
- Keep OS-specific code behind interfaces.
- Prefer deterministic rules over LLM calls.
- No `unwrap()` on production paths.
- Heavy work must not run in the audio callback (when audio exists).

## Commands

```bash
npm install
npm run test:rust
npm run tauri dev
```
