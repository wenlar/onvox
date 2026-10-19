# v1 Windows checklist

Run on a real Windows machine before a release. Tick each item.

## Setup and login (Task 5)

- [ ] With no `%APPDATA%\app.onvox\config.json`, starting Onvox opens the setup window (Turkish on a Turkish Windows, English otherwise).
- [ ] Choosing ChatGPT closes the setup window and shows chatgpt.com; logging in works.
- [ ] Closing the site window hides it (app keeps running); `config.json` has `"setup_done": true` and `"provider": "chatgpt"`.
- [ ] Restarting Onvox opens no window; the site loads hidden and stays logged in.
- [ ] The site's UI is in English inside Onvox (`--lang=en-US`), even on a Turkish Windows.
- [ ] Same steps with Grok.
