pub mod config;
pub mod permission;
pub mod provider;
pub mod selectors;
pub mod state;
pub mod text;

use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use config::Provider;
use selectors::Selectors;
use state::PageReport;

/// The settings file, loaded once at start and saved when something changes.
pub struct Settings {
    pub path: std::path::PathBuf,
    pub config: Mutex<config::Config>,
}

impl Settings {
    pub fn update(&self, f: impl FnOnce(&mut config::Config)) {
        let mut c = self.config.lock().unwrap();
        f(&mut c);
        let _ = c.save(&self.path);
    }
}

/// The built-in selectors at first, replaced by the GitHub copy once it arrives.
pub struct Sites(pub Mutex<Selectors>);

#[tauri::command]
fn page_report(report: PageReport, _app: AppHandle) {
    // shortcut: only logged until the hotkey part drives the state machine with it.
    println!("[page] {report:?}");
}

// async: a sync command runs on the main thread, and opening a window there deadlocks on Windows.
#[tauri::command]
async fn choose_provider(p: Provider, app: AppHandle) {
    app.state::<Settings>().update(|c| c.provider = p);
    if let Some(old) = app.get_webview_window(provider::LABEL) {
        let _ = old.destroy();
    }
    if let Some(win) = open_provider(&app) {
        provider::show_for_login(&win);
    }
    if let Some(setup) = app.get_webview_window("setup") {
        let _ = setup.destroy();
    }
}

fn open_provider(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    let p = app.state::<Settings>().config.lock().unwrap().provider;
    let site = app.state::<Sites>().0.lock().unwrap().site(p).cloned()?;
    let win = provider::open(app, &site).ok()?;
    let handle = app.clone();
    // Closing the site window after login hides it and finishes the setup.
    win.on_window_event(move |e| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = e {
            api.prevent_close();
            if let Some(w) = handle.get_webview_window(provider::LABEL) {
                provider::hide(&w);
            }
            handle.state::<Settings>().update(|c| c.setup_done = true);
        }
    });
    Some(win)
}

fn open_setup(app: &AppHandle) -> tauri::Result<()> {
    let lang = text::lang();
    let page = if lang == text::Lang::Tr { "setup.html?lang=tr" } else { "setup.html?lang=en" };
    WebviewWindowBuilder::new(app, "setup", WebviewUrl::App(page.into()))
        .title(text::t(text::Key::SetupTitle, lang))
        .inner_size(480.0, 360.0)
        .resizable(false)
        .center()
        .additional_browser_args(provider::BROWSER_ARGS)
        .build()?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![page_report, choose_provider])
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("config.json");
            let config = config::load(&path);
            let setup_done = config.setup_done;
            app.manage(Settings { path, config: Mutex::new(config) });
            app.manage(Sites(Mutex::new(selectors::embedded())));
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let fresh = selectors::load();
                *handle.state::<Sites>().0.lock().unwrap() = fresh;
            });
            if setup_done {
                open_provider(app.handle());
            } else {
                open_setup(app.handle())?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Onvox");
}
