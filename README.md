# Onvox

Talk to ChatGPT or Grok with one hotkey while you stream. Onvox sits in the Windows tray, opens the site's own voice
mode in a hidden window with your own subscription, and never takes focus from your game. Your viewers hear the AI
through your usual desktop audio in OBS.

Free and open source. Website: [onvox.app](https://onvox.app).

## Use

1. Download `Onvox_<version>_x64-setup.exe` from [Releases](https://github.com/wenlar/onvox/releases/latest) and install it.
2. Pick ChatGPT or Grok, log in in the window that opens, then close it. Onvox stays in the tray and starts with Windows.
3. Press **Ctrl+Alt+Space** to talk, again to stop. The tray icon shows a green dot while it talks.

The tray menu switches the site, starts a new chat, changes the hotkey, copies the OBS overlay address
(add it as a Browser source to show when the AI talks) and reports a problem.

## Build

Needs Rust and the Tauri CLI (`cargo install tauri-cli --version "^2"`).

```
cargo test
cargo tauri build
```
