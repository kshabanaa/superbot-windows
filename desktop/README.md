# SuperBot Windows

Download `SuperBot-Windows.exe` from the repository's Releases page and double-click it. It opens in a Windows window and starts its own server. On first launch it places its bundled backend files in `%LOCALAPPDATA%\SuperBot\bin`. Chats and workspace files are saved in `%LOCALAPPDATA%\SuperBot`.

Microsoft Edge WebView2 Runtime and a supported, signed-in agent CLI (Claude Code, Codex, or Pi) are required. The default bot uses Claude Code. Routines run while the app is open. The Linux virtual screen is not available on Windows. The upstream adapters bypass agent permission prompts, so agents can act with your Windows account permissions.

To build from source on Windows, install the Rust MSVC toolchain and Visual Studio C++ build tools. In PowerShell from the repository root:

```powershell
cargo build --release --locked
cargo build --release --locked --manifest-path desktop/Cargo.toml
Copy-Item desktop/target/release/superbot-desktop.exe SuperBot-Windows.exe
```

Build the server first: the desktop build embeds its two executable files. Real agent runs and cancellation need further verification.
