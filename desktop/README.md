# Windows desktop app

The desktop app opens SuperBot in a native Windows window using Microsoft Edge WebView2. It starts the local server automatically and stops it when the window closes. It requires Windows, the WebView2 Runtime, Rust with the MSVC toolchain, and Visual Studio C++ build tools.

From the repository root in PowerShell:

```powershell
cargo build --release --locked
cargo build --release --locked --manifest-path desktop/Cargo.toml
New-Item -ItemType Directory -Force dist/SuperBot | Out-Null
Copy-Item desktop/target/release/superbot-desktop.exe dist/SuperBot/SuperBot.exe
Copy-Item target/release/botd.exe, target/release/botctl.exe dist/SuperBot/
```

Run `dist/SuperBot/SuperBot.exe`. Keep all three executables together. Data is stored in `%LOCALAPPDATA%\SuperBot`; routines run while the app is open. Install and sign in to a supported agent CLI separately. The default bot uses Claude Code.

The Windows virtual desktop is not implemented; the Screen feature requires Linux display tools. Desktop launch, the UI, server shutdown, and a simulated agent reply have been tested. Real agent runs and cancellation need further verification. The upstream adapters bypass agent permission prompts, so agents can act with the permissions of the current user.
