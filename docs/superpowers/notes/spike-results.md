# Spike results

Date: 2026-10-19. Spike: `spike/` (throwaway), WebView2 hidden window, Ctrl+Alt+Space.

## ChatGPT (chatgpt.com)

- Works: hotkey toggles voice from the hidden window; AI audio plays; stop works. Confirmed by the user.
- Start: button `aria-label="Start Voice"`. Stop: `aria-label="End Voice"`. Match aria-label exactly;
  substring heuristics hit "Send message", "Voice settings", "Turn off microphone".
- Voice active marker: `End Voice` button present.
- Logged out marker: a visible button with text "Log in" (no Start Voice button when logged out;
  only "Start dictation" and "Send message").
- Microphone: WebView2 PermissionRequested kind=1 (MICROPHONE) for https://chatgpt.com/, auto-allowed, no prompt.
  The page also requests kind=13; denied, no visible effect.
- Login: user logs in once through the shown window; session survived app restarts.
- Starting voice on the home page creates a new chat (`/c/<id>`).
- Window: skip_taskbar window is unreachable once minimized -> tray Show must `unminimize()`;
  X button must hide to tray, not exit (user request).

- OBS: AI audio captured by OBS desktop audio (user confirmed).

- Fullscreen game: hotkey works in-game, game keeps focus, no stutter (user confirmed).


## Grok (grok.com)

- Works: start, AI audio, stop, OBS capture. Confirmed by the user.
- Start: aria-label starts with "Enter voice mode" (full: "Enter voice mode (Ctrl+⇧O)"; match by prefix).
  Stop: `aria-label="Exit voice mode"`. Voice active marker: Exit voice mode present.
- Decoys while active: "Mute microphone"/"Unmute microphone", "Voice settings", "Mute Grok".
- Microphone: PermissionRequested kind=1 for https://grok.com/, auto-allowed, no prompt.
- Starting voice on the home page creates a new chat (`/c/<id>`). Home may redirect to `/?mwbv=true`.
- Grok's own shortcut Ctrl+Shift+O: avoid offering it as a hotkey choice.

## Not observed (builder decides; keep in selectors.json so it can be fixed without a release)

- Usage-limit text on either site (limit never hit).
- Grok logged-out marker (user was already logged in when probed). ChatGPT: "Log in" button.

## Gate

Both providers pass. v1 ships ChatGPT and Grok.
