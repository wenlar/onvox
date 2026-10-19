# Onvox

Talk to ChatGPT or Grok with one hotkey while you stream. Onvox sits in the Windows tray, opens the site's own voice
mode in a hidden window with your own subscription, and never takes focus from your game. Your viewers hear the AI
through your usual desktop audio in OBS.

Free and open source. Website: [onvox.app](https://onvox.app).

## Build

Needs Rust and the Tauri CLI (`cargo install tauri-cli --version "^2"`).

```
cargo test
cargo tauri build
```
