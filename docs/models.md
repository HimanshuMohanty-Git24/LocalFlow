# Whisper models (manual install)

LocalFlow never downloads models. Place a Whisper.cpp ggml file in
`models/` yourself, then restart the app.

## Recommended for an 8 GB Windows PC

1. `ggml-small.en.bin` (best of the three for English)
2. `ggml-base.en.bin`
3. `ggml-tiny.en.bin` (fastest, least accurate)

Quantized files such as `ggml-base.en-q5_1.bin` also work.

## Where to get them

Download from Hugging Face (`ggerganov/whisper.cpp`) in a browser, then
copy the file into this repo's `models/` folder. Example names:

- `ggml-tiny.en.bin`
- `ggml-base.en.bin`
- `ggml-small.en.bin`

Do not use the OpenAI cloud API. Do not point LocalFlow at a URL.

The Windows installer from GitHub Releases already includes `ggml-base.en.bin`,
Silero VAD, and `vocabulary.txt`. You only need this page for development or
to add a larger Whisper file / optional Qwen GGUF.

## Search order

The app looks in:

1. `LOCALFLOW_MODELS_DIR` if set
2. `models/` next to the working directory
3. `models/` next to the executable (installer layout)
4. `%LOCALAPPDATA%\LocalFlow\models` (optional extra models such as Qwen)

It picks `small.en`, then `base.en`, then `tiny.en`, then any other
`ggml-*.bin` / `ggml-*.gguf`.

## Vocabulary (names and places)

Edit `models/vocabulary.txt`. Add your name, city, and product terms.
Whisper is primed with that list, and close misspellings (Orysa → Odisha)
are corrected after recognition.

`tiny.en` will still mangle Indian names. Prefer `ggml-base.en.bin` or
`ggml-small.en.bin` in the same folder.

If nothing is found, dictation shows **model file is missing** and the
app stays running.

## Silero VAD

Place `ggml-silero-v5.1.2.bin` in the same `models/` folder (Hugging Face
`ggml-org/whisper-vad`). LocalFlow will not fetch it.

If that file is missing, dictation still runs on the full capture.

## Qwen (optional)

Place `Qwen3-0.6B-Q8_0.gguf` in `models/` (Hugging Face `Qwen/Qwen3-0.6B-GGUF`).
Turn **AI cleanup with local Qwen** on in Settings. If the file is missing,
rule cleanup still runs.
