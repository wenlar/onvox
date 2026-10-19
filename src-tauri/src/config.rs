use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[default]
    #[serde(rename = "chatgpt")]
    ChatGpt,
    Grok,
    Claude,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub provider: Provider,
    pub hotkey: String,
    pub setup_done: bool,
    pub last_version: String,
    /// Unix seconds.
    pub last_update_check: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: Provider::ChatGpt,
            hotkey: "Ctrl+Alt+Space".into(),
            setup_done: false,
            last_version: String::new(),
            last_update_check: 0,
        }
    }
}

/// A missing or unreadable file gives the defaults, so a broken file means a fresh setup, never a crash.
pub fn load(path: &Path) -> Config {
    std::fs::read(path).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default()
}

impl Config {
    /// Writes a temp file and renames it over the old one, so a crash mid-write keeps the old file.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_default() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(&dir.path().join("config.json")), Config::default());
    }

    #[test]
    fn corrupt_file_gives_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{not json").unwrap();
        assert_eq!(load(&path), Config::default());
    }

    #[test]
    fn roundtrip_keeps_all_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let c = Config {
            provider: Provider::Grok,
            hotkey: "Ctrl+Shift+F9".into(),
            setup_done: true,
            last_version: "0.1.0".into(),
            last_update_check: 1_760_000_000,
        };
        c.save(&path).unwrap();
        assert_eq!(load(&path), c);
    }

    #[test]
    fn claude_is_stored_as_claude() {
        let c = Config { provider: Provider::Claude, ..Config::default() };
        assert!(serde_json::to_string(&c).unwrap().contains(r#""provider":"claude""#));
    }

    #[test]
    fn unknown_provider_gives_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"provider":"gemini","hotkey":"F9","setup_done":true}"#).unwrap();
        assert_eq!(load(&path), Config::default());
    }

    #[test]
    fn save_leaves_no_tmp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        Config::default().save(&path).unwrap();
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from("config.json")]);
    }
}
