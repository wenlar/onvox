# Releasing Onvox

The installer is built by `.github/workflows/release.yml` and signed twice: Authenticode by Azure
Artifact Signing (so Windows names Wenlar LLC as the publisher), and with the updater key (so
installed copies accept it as an update).

## Cutting a release

1. Set the version in the root `Cargo.toml` (`[workspace.package] version`) and merge to `main`.
   Run CI first (`gh workflow run ci.yml --ref main`).
2. Tag the merge and push the tag:
   ```sh
   git tag -a v0.1.0 -m "Onvox 0.1.0"
   git push origin v0.1.0
   ```
   The tag must be `v` + the `Cargo.toml` version, or the workflow stops at once.
3. The workflow builds `Onvox_<v>_x64-setup.exe`, its updater signature `.sig`, `latest.json` and
   `SHA256SUMS.txt`, and opens a **draft** GitHub Release with them and generated notes.
4. Download the installer from the draft, install it over the previous version and go through
   [the checklist](superpowers/notes/checklists/v1-windows.md). Edit the notes, then publish.
   **Publishing sends the update to every Onvox** (see below): don't publish an installer you
   have not tried.

A test build without a tag or a release: `gh workflow run release.yml --ref <branch> -f version=0.1.0-rc.1`.
The files are then the run's artifacts (Actions ▸ the run ▸ Artifacts).

## Automatic updates

Onvox reads `https://github.com/wenlar/onvox/releases/latest/download/latest.json` 10 s after it
starts and then every 20 hours, downloads a newer installer in the background, checks its
signature against the public key in `src-tauri/tauri.conf.json`, and installs it when the user
picks "Update and restart" in the tray or quits, never during a voice talk.

- `releases/latest` is the newest **published, non-pre-release** release. Drafts and
  pre-releases (a version with `-`, e.g. `0.1.0-rc.1`) reach no one.
- To withdraw a release, turn it back into a draft. To undo one, publish a higher version: the
  updater never goes to a lower one.
- Keep the file names and `latest.json` as the workflow makes them: they are what installed
  copies look for.

## Keys and settings

| What | Where |
|---|---|
| Updater private key and its password | Repository secrets `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; the maintainer's backup in `%USERPROFILE%\.tauri\onvox.key` and `onvox.key.password`. **Losing it means installed copies can never update again**: they only accept installers signed with it. |
| Updater public key | `plugins.updater.pubkey` in `src-tauri/tauri.conf.json` |
| Azure signing | Repository variables `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_SUBSCRIPTION_ID`, `AZURE_SIGNING_ENDPOINT`, `AZURE_SIGNING_ACCOUNT`, `AZURE_CERT_PROFILE`; the `release` environment |

## Windows signing

The same Azure Artifact Signing account and certificate profile as Gezik (`wenlar-signing`,
`wenlar-public`, Wenlar LLC); see Gezik's `docs/releasing.md` for how they were set up. Onvox has
its own federated credential on the app registration: GitHub Actions, organization `wenlar`,
repository `onvox`, entity **Environment**, name `release`.

The workflow logs in to Azure with that credential, installs Microsoft's signing dlib for `signtool`, and passes
Tauri a `signCommand` through `--config`, so Tauri signs `onvox.exe` and the installer. Without
the repository variables (a fork) the steps are skipped and the build is unsigned. Check a run:
`Get-AuthenticodeSignature Onvox_<v>_x64-setup.exe` says `Valid` with Wenlar LLC (the
workflow's Package step prints it too).

## Checking a download

```powershell
Get-FileHash Onvox_0.1.0_x64-setup.exe -Algorithm SHA256   # compare with SHA256SUMS.txt
```
