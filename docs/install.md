# Install LocalFlow (Windows)

This is for people who downloaded the installer from
[Releases](https://github.com/HimanshuMohanty-Git24/LocalFlow/releases).
You do not need Git, Node, or Rust.

## Install and dictate

1. Download the latest NSIS `.exe` (for example `LocalFlow_0.1.0_x64-setup.exe`).
2. Run it. It installs for your Windows user only — no admin password.
3. If SmartScreen warns that the app is unsigned, choose **More info** → **Run anyway**.
4. Click in any text field, hold **Ctrl+B**, speak, then release.
5. For longer dictation, press **Ctrl+B** twice, speak, then press **Space** or **Esc** to stop.

Whisper speech recognition is already inside the installer. Dictation works
offline without any extra download.

## Optional: add local Qwen (AI cleanup)

The installer does **not** include Qwen (the file is about 610 MB). Without it,
LocalFlow still cleans text with built-in rules. Qwen only adds extra
punctuation and light rewriting, still fully offline — nothing is uploaded.

### 1. Create this folder

Paste this into File Explorer’s address bar and press Enter:

```
%LOCALAPPDATA%\LocalFlow\models
```

That is usually:

`C:\Users\<your-name>\AppData\Local\LocalFlow\models`

If Windows says the folder does not exist, create `LocalFlow\models` under
`AppData\Local`.

### 2. Download the model in a browser

Open this Hugging Face page and download **`Qwen3-0.6B-Q8_0.gguf`**:

https://huggingface.co/Qwen/Qwen3-0.6B-GGUF

Direct file (same model, still a manual download — LocalFlow never fetches it):

https://huggingface.co/Qwen/Qwen3-0.6B-GGUF/resolve/main/Qwen3-0.6B-Q8_0.gguf

Use a Q8 or similar **0.6B** GGUF. The filename must include `qwen` and end
with `.gguf`.

### 3. Put the file in that folder

Copy `Qwen3-0.6B-Q8_0.gguf` into `%LOCALAPPDATA%\LocalFlow\models`.

Do not rename it to something that drops the word `qwen`.

### 4. Restart LocalFlow and turn cleanup on

1. Quit LocalFlow from the tray icon, then open it again.
2. Open **Settings**.
3. Leave **AI cleanup with local Qwen (offline)** checked.
4. Dictate as usual. The first Qwen load can take a short while on CPU.

You need about **8 GB RAM**. If the GGUF is missing, dictation still works;
only the extra AI rewrite is skipped.

## Optional: a larger Whisper model

The installer already includes `ggml-base.en.bin`. To try a more accurate
English model later, put `ggml-small.en.bin` in the same
`%LOCALAPPDATA%\LocalFlow\models` folder and restart. See [models.md](models.md).
