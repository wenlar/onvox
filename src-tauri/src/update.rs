//! Updates like Gezik: a GitHub release's latest.json is checked every 20 hours, the signed
//! installer is downloaded in the background, and it installs only when no voice talk is running.

use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::config::Config;
use crate::state::Phase;

const EVERY: u64 = 20 * 3600;

pub fn due(last_check: u64, now: u64) -> bool {
    now.saturating_sub(last_check) >= EVERY
}

/// Never in the middle of a talk: the installer closes the app.
pub fn can_install(phase: &Phase) -> bool {
    !matches!(phase, Phase::Starting | Phase::Listening)
}

/// The new version, once, on the first start after an update.
pub fn just_updated(cfg: &Config, current: &str) -> Option<String> {
    (!cfg.last_version.is_empty() && cfg.last_version != current).then(|| current.to_string())
}

/// A downloaded update waiting for a quiet moment, and the version this run was updated to.
#[derive(Default)]
pub struct Updates {
    pub ready: Mutex<Option<(Update, Vec<u8>)>>,
    pub just_updated: Mutex<Option<String>>,
}

impl Updates {
    pub fn ready_version(&self) -> Option<String> {
        self.ready.lock().unwrap().as_ref().map(|(u, _)| u.version.clone())
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 10 s after start, then hourly: check when due, download, and offer it in the tray.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(10));
        loop {
            let last = app.state::<crate::Settings>().config.lock().unwrap().last_update_check;
            if app.state::<Updates>().ready_version().is_none() && due(last, now()) {
                if let Ok(Some((update, bytes))) = tauri::async_runtime::block_on(fetch(&app)) {
                    *app.state::<Updates>().ready.lock().unwrap() = Some((update, bytes));
                    crate::tray::refresh(&app);
                }
                app.state::<crate::Settings>().update(|c| c.last_update_check = now());
            }
            std::thread::sleep(Duration::from_secs(3600));
        }
    });
}

async fn fetch(app: &AppHandle) -> tauri_plugin_updater::Result<Option<(Update, Vec<u8>)>> {
    let Some(update) = app.updater()?.check().await? else { return Ok(None) };
    let bytes = update.download(|_, _| {}, || {}).await?;
    Ok(Some((update, bytes)))
}

/// Installs a downloaded update if nothing is talking; the installer then closes and restarts the app.
/// Returns false when there was nothing to install or it had to wait.
pub fn install_if_quiet(app: &AppHandle) -> bool {
    if !can_install(&app.state::<crate::Session>().phase.lock().unwrap()) {
        return false;
    }
    let Some((update, bytes)) = app.state::<Updates>().ready.lock().unwrap().take() else { return false };
    update.install(bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Fault;

    #[test]
    fn due_after_20_hours() {
        assert!(due(1_000, 1_000 + 20 * 3600));
        assert!(due(0, 1_760_000_000));
    }

    #[test]
    fn not_due_before() {
        assert!(!due(1_000, 1_000 + 20 * 3600 - 1));
        // A clock set back must not underflow into "due".
        assert!(!due(5_000, 1_000));
    }

    #[test]
    fn install_blocked_while_active() {
        assert!(!can_install(&Phase::Starting));
        assert!(!can_install(&Phase::Listening));
        assert!(can_install(&Phase::Idle));
        assert!(can_install(&Phase::Error(Fault::LimitReached)));
    }

    #[test]
    fn just_updated_once() {
        let mut cfg = Config { last_version: "0.1.0".into(), ..Config::default() };
        assert_eq!(just_updated(&cfg, "0.2.0"), Some("0.2.0".into()));
        cfg.last_version = "0.2.0".into();
        assert_eq!(just_updated(&cfg, "0.2.0"), None);
        cfg.last_version.clear();
        assert_eq!(just_updated(&cfg, "0.2.0"), None);
    }
}
