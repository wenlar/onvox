# Onvox: rules for coding agents

- Commit messages and PR texts are in English.
- Never add `Co-Authored-By: Claude …` or any other AI attribution line to commits or PRs.
- Chat with the maintainer in Turkish.
- CI (fmt, clippy, tests on Windows) runs only when started by hand: run it (`gh workflow run ci.yml --ref <branch>`) before a big step's last PR merges, not on every PR.
- Test items for a step go to `docs/superpowers/notes/checklists/<step>-windows.md`; never append to another step's checklist.
- Site selectors live in `selectors.json` (repo root), not in code, so a site change can be fixed without a new release.
