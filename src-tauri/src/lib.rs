pub mod config;
pub mod state;
pub mod text;

use std::sync::Mutex;

use tauri::Manager;

/// The settings file, loaded once at start and saved when something changes.
pub struct Settings {
    pub path: std::path::PathBuf,
    pub config: Mutex<config::Config>,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("config.json");
            let config = Mutex::new(config::load(&path));
            app.manage(Settings { path, config });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Onvox");
}
