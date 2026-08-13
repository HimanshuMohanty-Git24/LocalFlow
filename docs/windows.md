# Windows

Windows 10/11 x64 is the first supported platform.

## Development

Install:

1. [Rust](https://rustup.rs/) with the `stable-x86_64-pc-windows-msvc` toolchain
2. Visual Studio 2022 Build Tools, workload **Desktop development with C++**
3. Node.js 20+
4. WebView2 Runtime (included on most Windows 11 systems)

## Runtime notes

- Microphone: WASAPI through CPAL
- Text insertion: clipboard paste, then `SendInput` fallback
- A normal-integrity process cannot inject into a higher-integrity app
- If insertion fails, copy text to the clipboard and tell the user
