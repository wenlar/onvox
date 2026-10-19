//! The global hotkey: RegisterHotKey through the plugin, no keyboard hook, so games and anti-cheat see nothing.

use std::sync::Mutex;

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// The shortcut registered now, so a failed change can put it back.
static CURRENT: Mutex<Option<Shortcut>> = Mutex::new(None);

/// A bare key would fire while typing in chat or playing, so a modifier is required.
pub fn validate(s: &str) -> Result<Shortcut, String> {
    let sc: Shortcut = s.parse().map_err(|e| format!("{e}"))?;
    if sc.mods.is_empty() {
        return Err("needs Ctrl, Alt, Shift or Win".into());
    }
    Ok(sc)
}

/// Swaps in the new shortcut; if Windows refuses it (another app owns it), the old one stays.
pub fn register(app: &AppHandle, s: &str) -> Result<(), String> {
    let new = validate(s)?;
    let gs = app.global_shortcut();
    let mut current = CURRENT.lock().unwrap();
    if let Some(old) = *current {
        let _ = gs.unregister(old);
    }
    match gs.register(new) {
        Ok(()) => {
            *current = Some(new);
            Ok(())
        }
        Err(e) => {
            if let Some(old) = *current {
                let _ = gs.register(old);
            }
            Err(e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hotkey_needs_modifier() {
        assert!(validate("Space").is_err());
    }

    #[test]
    fn default_hotkey_valid() {
        assert!(validate(&crate::config::Config::default().hotkey).is_ok());
    }

    #[test]
    fn garbage_hotkey_rejected() {
        assert!(validate("Ctrl+Banana").is_err());
        assert!(validate("").is_err());
    }
}
