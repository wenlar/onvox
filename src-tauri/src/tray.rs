//! The tray icon and its menu: the app's whole visible face.

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Manager};

use crate::config::Provider;
use crate::state::{Fault, Icon, Phase};
use crate::text::{lang, t, Key, Lang};

const ID: &str = "main";

pub fn status_text(phase: &Phase, lang: Lang) -> &'static str {
    let key = match phase {
        Phase::Idle => Key::StatusIdle,
        Phase::Starting => Key::StatusStarting,
        Phase::Listening => Key::StatusListening,
        Phase::Error(Fault::NotLoggedIn) => Key::StatusNotLoggedIn,
        Phase::Error(Fault::VoiceButtonMissing) => Key::StatusVoiceButtonMissing,
        Phase::Error(Fault::LimitReached) => Key::StatusLimitReached,
        Phase::Error(Fault::NoResponse) => Key::StatusNoResponse,
    };
    t(key, lang)
}

/// The icon the session should show once the page is ready again.
pub fn icon_for(phase: &Phase) -> Icon {
    match phase {
        Phase::Idle => Icon::Idle,
        Phase::Starting | Phase::Listening => Icon::Active,
        Phase::Error(_) => Icon::Error,
    }
}

fn image(icon: Icon) -> Image<'static> {
    let bytes: &'static [u8] = match icon {
        Icon::Loading => include_bytes!("../icons/tray-loading.png"),
        Icon::Idle => include_bytes!("../icons/tray-idle.png"),
        Icon::Active => include_bytes!("../icons/tray-active.png"),
        Icon::Error => include_bytes!("../icons/tray-error.png"),
    };
    Image::from_bytes(bytes).expect("tray icons are valid PNGs")
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let l = lang();
    let phase = *app.state::<crate::Session>().phase.lock().unwrap();
    let provider = app.state::<crate::Settings>().config.lock().unwrap().provider;
    let sites = app.state::<crate::Sites>().0.lock().unwrap().clone();
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(app, "status", current_status(app, &phase, l), false, None::<&str>)?)?;
    let updates = app.state::<crate::update::Updates>();
    // Copied out: building menu items waits for the main thread, so no lock may be held across it.
    let just_updated = updates.just_updated.lock().unwrap().clone();
    if let Some(v) = just_updated {
        menu.append(&MenuItem::with_id(
            app,
            "updated",
            t(Key::JustUpdated, l).replace("{v}", &v),
            false,
            None::<&str>,
        )?)?;
    }
    let ready = updates.ready_version();
    if let Some(v) = &ready {
        menu.append(&MenuItem::with_id(app, "ready", t(Key::UpdateReady, l).replace("{v}", v), false, None::<&str>)?)?;
        menu.append(&MenuItem::with_id(app, "update", t(Key::MenuUpdate, l), true, None::<&str>)?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    for (p, id, name) in [
        (Provider::ChatGpt, "p:chatgpt", "ChatGPT"),
        (Provider::Grok, "p:grok", "Grok"),
        (Provider::Claude, "p:claude", "Claude"),
    ] {
        if sites.site(p).is_some() {
            menu.append(&CheckMenuItem::with_id(app, id, name, true, p == provider, None::<&str>)?)?;
        }
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "new_chat", t(Key::MenuNewChat, l), true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "hotkey", t(Key::MenuHotkey, l), true, None::<&str>)?)?;
    let overlay_ok = app.state::<crate::Session>().overlay_ok.load(std::sync::atomic::Ordering::SeqCst);
    let obs = if overlay_ok { Key::MenuCopyOverlay } else { Key::MenuOverlayPortBusy };
    menu.append(&MenuItem::with_id(app, "obs", t(obs, l), overlay_ok, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "relogin", t(Key::MenuRelogin, l), true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "report", t(Key::MenuReport, l), true, None::<&str>)?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", t(Key::MenuQuit, l), true, None::<&str>)?)?;
    Ok(menu)
}

pub fn build(app: &AppHandle) -> tauri::Result<TrayIcon> {
    TrayIconBuilder::with_id(ID)
        .icon(image(Icon::Idle))
        .tooltip("Onvox")
        .menu(&menu(app)?)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, e| crate::on_menu(app, e.id.as_ref()))
        .build(app)
}

pub fn set_icon(app: &AppHandle, icon: Icon) {
    if let Some(tray) = app.tray_by_id(ID) {
        let _ = tray.set_icon(Some(image(icon)));
    }
}

/// Rebuilds the menu so the status line and the checked provider are current.
pub fn refresh(app: &AppHandle) {
    let phase = *app.state::<crate::Session>().phase.lock().unwrap();
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(ID), menu(app)) {
        let _ = tray.set_menu(Some(menu));
        let _ = tray.set_tooltip(Some(format!("Onvox: {}", current_status(app, &phase, lang()))));
    }
}

/// While the site loads, say so; a hotkey press then waits for it.
fn current_status(app: &AppHandle, phase: &Phase, l: Lang) -> &'static str {
    let loading = !app.state::<crate::Session>().page_ready.load(std::sync::atomic::Ordering::SeqCst);
    if loading && *phase == Phase::Idle {
        t(Key::StatusLoading, l)
    } else {
        status_text(phase, l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Fault;

    #[test]
    fn status_text_covers_every_phase() {
        let faults = [Fault::NotLoggedIn, Fault::VoiceButtonMissing, Fault::LimitReached, Fault::NoResponse];
        let phases = [Phase::Idle, Phase::Starting, Phase::Listening].into_iter().chain(faults.map(Phase::Error));
        for phase in phases {
            for lang in [Lang::Tr, Lang::En] {
                assert!(!status_text(&phase, lang).is_empty(), "{phase:?} {lang:?}");
            }
        }
        // Each fault says something different, so the streamer knows what to fix.
        let texts: std::collections::HashSet<_> = faults.map(|f| status_text(&Phase::Error(f), Lang::En)).into();
        assert_eq!(texts.len(), 4);
    }

    #[test]
    fn icon_after_load_matches_phase() {
        assert_eq!(icon_for(&Phase::Idle), Icon::Idle);
        assert_eq!(icon_for(&Phase::Starting), Icon::Active);
        assert_eq!(icon_for(&Phase::Listening), Icon::Active);
        assert_eq!(icon_for(&Phase::Error(Fault::NoResponse)), Icon::Error);
    }
}
