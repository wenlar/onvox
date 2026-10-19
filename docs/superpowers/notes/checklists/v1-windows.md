# v1 Windows checklist

Run on a real Windows machine before a release. Tick each item.

## Setup and login (Task 5)

- [ ] With no `%APPDATA%\app.onvox\config.json`, starting Onvox opens the setup window (Turkish on a Turkish Windows, English otherwise).
- [ ] Choosing ChatGPT closes the setup window and shows chatgpt.com; logging in works.
- [ ] Closing the site window hides it (app keeps running); `config.json` has `"setup_done": true` and `"provider": "chatgpt"`.
- [ ] Restarting Onvox opens no window; the site loads hidden and stays logged in.
- [ ] The site's UI is in English inside Onvox (`--lang=en-US`), even on a Turkish Windows.
- [ ] Same steps with Grok.

## Hotkey, tray, sounds (Task 6)

- [ ] Tray icon is grey with a spinner while the site loads, then normal; the menu's status line says "Loading the page…" meanwhile.
- [ ] Ctrl+Alt+Space during the load: voice starts by itself once the page is ready.
- [ ] Ctrl+Alt+Space: green dot on the icon and the site's own voice chime; again: dot gone, voice ends. No Onvox beep on start or stop.
- [ ] Not logged in / limit / no response: one low beep, red dot, status line names the problem.
- [ ] Tray: switch ChatGPT <-> Grok; the app stays open; a logged-out site shows its window, a logged-in one stays hidden.
- [ ] Cookie banner on a fresh profile is answered by itself (prefers "reject non-essential"); voice then starts.
- [ ] "Change hotkey…": a bare key is refused; Ctrl+Shift+F9 saves, works at once and survives a restart.
- [ ] A hotkey another app already owns is refused with a message; the old one keeps working.
- [ ] Quit while talking: voice ends before the app exits.
- [ ] The X button and minimize on the site window never lose it: "Log in again" brings it back.

## OBS overlay (Task 7)

- [ ] Tray "Copy OBS overlay address" puts `http://127.0.0.1:38517/overlay` on the clipboard.
- [ ] OBS > Sources > Browser, that URL, 300x80: transparent while idle; a pill with a purple dot and "AI" while voice starts (dim dot) and talks (pulsing dot); gone after stop.
- [ ] With another program on port 38517, Onvox still starts and the tray item reads "OBS overlay off: port 38517 is used by another app".
