# LocalFlow

Completely offline voice dictation for Windows. Speech stays on this PC.
Nothing is sent to a cloud API.

Phase 7 can optionally rewrite the cleaned transcript with local
Qwen3-0.6B (llama.cpp, CPU). No cloud. Place GGUF files in `models/`;
see [docs/models.md](docs/models.md).

## Requirements (development)

- Windows 10/11 x64
- Node.js 20+
- Rust (stable, MSVC toolchain)
- Visual Studio 2022 Build Tools with the C++ workload
- WebView2 (usually already installed on Windows 11)
- A working microphone for capture tests
- CMake (for compiling whisper.cpp)
- LLVM / Clang (bindgen for whisper-rs on Windows)
- A local Whisper ggml file in `models/` (tiny.en is enough to try)

## Run

```bash
npm install
npm run test:rust
npm run tauri dev
```

Optional verbose logs:

```powershell
$env:LOCALFLOW_LOG = "debug"
npm run tauri dev
```

The app lives in the system tray. Closing the window hides it; use **Quit**
in the tray menu to exit.

## Install (Windows)

Download the latest **NSIS installer** (`.exe`) from
[Releases](https://github.com/HimanshuMohanty-Git24/LocalFlow/releases).
It is a current-user install (no admin). Whisper `base.en` is included.

Windows SmartScreen may warn because the installer is not yet code-signed.

After install, see [docs/install.md](docs/install.md) for first use and how
to add optional local Qwen (AI cleanup) into
`%LOCALAPPDATA%\LocalFlow\models`.

## Releases

Push a version tag to build and publish the installer:

```bash
git tag v0.1.0
git push origin v0.1.0
```

`main` only accepts pull requests. Do not push commits directly to `main`.

## Privacy

See [docs/privacy.md](docs/privacy.md). Audio, transcripts, and telemetry are
never sent anywhere.
