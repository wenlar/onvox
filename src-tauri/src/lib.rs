pub mod beep;
pub mod config;
pub mod hotkey;
pub mod overlay;
pub mod permission;
pub mod provider;
pub mod report;
pub mod selectors;
pub mod state;
pub mod text;
pub mod tray;
pub mod update;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::ShortcutState;

use config::Provider;
use selectors::{Selectors, SiteSelectors};
use state::{Effect, Event, PageReport, Phase};

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

/// The voice session. `phase` is shared with the OBS overlay server.
pub struct Session {
    pub phase: Arc<Mutex<Phase>>,
    /// Bumped on every start, so a stale start timer does nothing.
    starts: AtomicU64,
    /// True while a login check started by picking a site waits for the page's answer.
    checking_login: AtomicBool,
    /// False from a page load's start until the page shows its voice button or a login sign.
    pub page_ready: AtomicBool,
    /// False when the overlay port was taken at start; the tray says so.
    pub overlay_ok: AtomicBool,
    /// Bumped on every finished load, so an older load's polling stops.
    loads: AtomicU64,
}

/// Called by the site window as a page starts and finishes loading. The document loading is not
/// the app being usable, so after it the page is asked every 500 ms (for 30 s) until the voice
/// button or a login sign shows.
pub fn page_loaded(app: &AppHandle, finished: bool) {
    let session = app.state::<Session>();
    if !finished {
        session.page_ready.store(false, Ordering::SeqCst);
        tray::set_icon(app, state::Icon::Loading);
        tray::refresh(app);
        return;
    }
    let load = session.loads.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    std::thread::spawn(move || {
        for _ in 0..60 {
            let session = app.state::<Session>();
            if session.page_ready.load(Ordering::SeqCst) || session.loads.load(Ordering::SeqCst) != load {
                return;
            }
            if let (Some(win), Some(site)) = (app.get_webview_window(provider::LABEL), current_site(&app)) {
                provider::check_login(&win, &site);
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        // No answer (site changed or offline): stop showing "loading"; a press then reports the real fault.
        if app.state::<Session>().loads.load(Ordering::SeqCst) == load {
            page_ready(&app);
        }
    });
}

/// The page can take a press now: restore the session's icon and resend a press made during the load.
fn page_ready(app: &AppHandle) {
    let session = app.state::<Session>();
    if session.page_ready.swap(true, Ordering::SeqCst) {
        return;
    }
    let phase = *session.phase.lock().unwrap();
    tray::set_icon(app, tray::icon_for(&phase));
    if phase == Phase::Starting {
        apply_effects(app, vec![Effect::StartVoice]);
    }
    tray::refresh(app);
}

fn current_site(app: &AppHandle) -> Option<SiteSelectors> {
    let p = app.state::<Settings>().config.lock().unwrap().provider;
    app.state::<Sites>().0.lock().unwrap().site(p).cloned()
}

/// Feeds one event through the state machine and carries out what it asks for.
pub fn dispatch(app: &AppHandle, ev: Event) {
    let effects = {
        let session = app.state::<Session>();
        let mut phase = session.phase.lock().unwrap();
        let (next, effects) = state::step(&phase, ev);
        #[cfg(debug_assertions)]
        println!("[state] {:?} + {ev:?} -> {next:?} {effects:?}", *phase);
        *phase = next;
        effects
    };
    apply_effects(app, effects);
    tray::refresh(app);
}

pub fn apply_effects(app: &AppHandle, effects: Vec<Effect>) {
    let win = app.get_webview_window(provider::LABEL);
    let site = current_site(app);
    for effect in effects {
        match effect {
            Effect::Beep(b) => beep::play(b),
            Effect::Icon(i) => tray::set_icon(app, i),
            Effect::StartVoice => {
                if let (Some(win), Some(site)) = (&win, &site) {
                    provider::start_voice(win, site);
                }
                let n = app.state::<Session>().starts.fetch_add(1, Ordering::SeqCst) + 1;
                let app = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(state::START_TIMEOUT);
                    if app.state::<Session>().starts.load(Ordering::SeqCst) == n {
                        dispatch(&app, Event::StartTimeout);
                    }
                });
            }
            Effect::StopVoice => {
                if let (Some(win), Some(site)) = (&win, &site) {
                    provider::stop_voice(win, site);
                }
            }
        }
    }
}

#[tauri::command]
fn page_report(report: PageReport, app: AppHandle) {
    let session = app.state::<Session>();
    match report {
        PageReport::LoggedIn | PageReport::LoginNeeded => {
            page_ready(&app);
            // Only a check the user started by picking a site opens a window; never pop one on its own.
            if !session.checking_login.swap(false, Ordering::SeqCst) {
                return;
            }
            if report == PageReport::LoginNeeded {
                if let Some(win) = app.get_webview_window(provider::LABEL) {
                    provider::show_for_login(&win);
                }
            } else {
                finish_setup(&app);
            }
        }
        _ => dispatch(&app, Event::Report(report)),
    }
}

/// Marks that the user picked a site, so the load's login answer may open the window.
fn check_login(app: &AppHandle) {
    app.state::<Session>().checking_login.store(true, Ordering::SeqCst);
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(35));
        // No answer (site changed or offline): show it, so the user sees what is wrong.
        if app.state::<Session>().checking_login.swap(false, Ordering::SeqCst) {
            if let Some(win) = app.get_webview_window(provider::LABEL) {
                provider::show_for_login(&win);
            }
        }
    });
}

// async: a sync command runs on the main thread, and opening a window there deadlocks on Windows.
#[tauri::command]
async fn choose_provider(p: Provider, app: AppHandle) {
    switch_provider(&app, p);
    if let Some(setup) = app.get_webview_window("setup") {
        let _ = setup.destroy();
    }
}

#[tauri::command]
async fn set_hotkey(combo: String, app: AppHandle) -> Result<(), String> {
    hotkey::register(&app, &combo)?;
    app.state::<Settings>().update(|c| c.hotkey = combo);
    if let Some(w) = app.get_webview_window("hotkey") {
        let _ = w.destroy();
    }
    Ok(())
}

/// Loads the new site hidden and shows it only if it needs a login; any talk on the old one ends.
fn switch_provider(app: &AppHandle, p: Provider) {
    dispatch(app, Event::NewChat);
    app.state::<Settings>().update(|c| c.provider = p);
    // Reuse the window: destroying it and building a new one with the same label races on Windows.
    let ready = match (app.get_webview_window(provider::LABEL), current_site(app)) {
        (Some(win), Some(site)) => {
            provider::new_chat(&win, &site);
            true
        }
        _ => open_provider(app).is_some(),
    };
    if ready {
        check_login(app);
    }
    tray::refresh(app);
}

fn open_provider(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    let win = provider::open(app, &current_site(app)?).ok()?;
    let handle = app.clone();
    // Closing the site window after login hides it and finishes the setup.
    win.on_window_event(move |e| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = e {
            api.prevent_close();
            if let Some(w) = handle.get_webview_window(provider::LABEL) {
                provider::hide(&w);
            }
            finish_setup(&handle);
        }
    });
    Some(win)
}

/// The first time the user is set up, Onvox starts with Windows from then on.
fn finish_setup(app: &AppHandle) {
    let first = !app.state::<Settings>().config.lock().unwrap().setup_done;
    if first {
        app.state::<Settings>().update(|c| c.setup_done = true);
        use tauri_plugin_autostart::ManagerExt;
        let _ = app.autolaunch().enable();
    }
}

/// A local page from `ui/`, told the Windows language through the query.
fn open_page(app: &AppHandle, label: &str, title: text::Key, size: (f64, f64)) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(label) {
        return w.set_focus();
    }
    let lang = text::lang();
    let page = format!("{label}.html?lang={}", if lang == text::Lang::Tr { "tr" } else { "en" });
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(page.into()))
        .title(text::t(title, lang))
        .inner_size(size.0, size.1)
        .resizable(false)
        .center()
        .additional_browser_args(provider::BROWSER_ARGS)
        .build()?;
    Ok(())
}

pub fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "p:chatgpt" => switch_provider(app, Provider::ChatGpt),
        "p:grok" => switch_provider(app, Provider::Grok),
        "new_chat" => {
            dispatch(app, Event::NewChat);
            if let (Some(win), Some(site)) = (app.get_webview_window(provider::LABEL), current_site(app)) {
                provider::new_chat(&win, &site);
            }
        }
        "hotkey" => {
            let _ = open_page(app, "hotkey", text::Key::HotkeyTitle, (420.0, 260.0));
        }
        "obs" => {
            use tauri_plugin_clipboard_manager::ClipboardExt;
            let _ = app.clipboard().write_text(overlay::OVERLAY_URL);
        }
        "update" => {
            // Only offered when ready; refused mid-talk, in which case the error beep says so.
            if !update::install_if_quiet(app) {
                beep::play(state::Beep::Error);
            }
        }
        "report" => {
            use tauri_plugin_opener::OpenerExt;
            let p = app.state::<Settings>().config.lock().unwrap().provider;
            let phase = *app.state::<Session>().phase.lock().unwrap();
            let version = app.package_info().version.to_string();
            let _ = app.opener().open_url(report::issue_url(&version, p, &phase), None::<&str>);
        }
        "relogin" => match app.get_webview_window(provider::LABEL) {
            Some(win) => provider::show_for_login(&win),
            None => {
                if let Some(win) = open_provider(app) {
                    provider::show_for_login(&win);
                }
            }
        },
        "quit" => {
            let talking = matches!(*app.state::<Session>().phase.lock().unwrap(), Phase::Starting | Phase::Listening);
            if talking {
                dispatch(app, Event::Hotkey);
            }
            // Give the page a moment to hang up before the window goes.
            let app = app.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(if talking { 500 } else { 0 }));
                app.exit(0);
            });
        }
        _ => {}
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _, ev| {
                    if ev.state == ShortcutState::Pressed {
                        dispatch(app, Event::Hotkey);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![page_report, choose_provider, set_hotkey])
        .setup(|app| {
            let path = app.path().app_config_dir()?.join("config.json");
            let config = config::load(&path);
            let (setup_done, combo) = (config.setup_done, config.hotkey.clone());
            let version = app.package_info().version.to_string();
            let updates = update::Updates::default();
            *updates.just_updated.lock().unwrap() = update::just_updated(&config, &version);
            app.manage(updates);
            app.manage(Settings { path, config: Mutex::new(config) });
            app.state::<Settings>().update(|c| c.last_version = version);
            app.manage(Sites(Mutex::new(selectors::embedded())));
            app.manage(Session {
                phase: Arc::default(),
                starts: AtomicU64::new(0),
                checking_login: AtomicBool::new(false),
                page_ready: AtomicBool::new(false),
                loads: AtomicU64::new(0),
                overlay_ok: AtomicBool::new(false),
            });
            let phase = app.state::<Session>().phase.clone();
            let overlay_ok = overlay::serve(overlay::OVERLAY_PORT, phase).is_ok();
            app.state::<Session>().overlay_ok.store(overlay_ok, Ordering::SeqCst);
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let fresh = selectors::load();
                *handle.state::<Sites>().0.lock().unwrap() = fresh;
                tray::refresh(&handle);
            });
            tray::build(app.handle())?;
            update::spawn(app.handle());
            let hotkey_ok = hotkey::register(app.handle(), &combo).is_ok();
            if !setup_done {
                open_page(app.handle(), "setup", text::Key::SetupTitle, (480.0, 360.0))?;
            } else {
                open_provider(app.handle());
                // Another app owns the hotkey: ask for a new one now, at start, rather than fail silently in-game.
                if !hotkey_ok {
                    open_page(app.handle(), "hotkey", text::Key::HotkeyTitle, (420.0, 260.0))?;
                }
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Onvox")
        .run(|app, event| match event {
            // A tray app: no window left (e.g. while the site window is swapped) is not a reason to quit.
            // Only the tray's Quit exits, and it passes an exit code.
            tauri::RunEvent::ExitRequested { code: None, api, .. } => api.prevent_exit(),
            // Quitting is a quiet moment: put a downloaded update in now.
            tauri::RunEvent::ExitRequested { code: Some(_), .. } => {
                update::install_if_quiet(app);
            }
            _ => {}
        });
}
