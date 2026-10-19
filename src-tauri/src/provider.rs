//! The hidden window that runs the site's own voice mode.

use tauri::utils::config::BackgroundThrottlingPolicy;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::selectors::SiteSelectors;

pub const LABEL: &str = "provider";

/// Every window shares one WebView2 environment, so they all need the same arguments.
/// English UI: the selectors are the sites' English labels.
pub const BROWSER_ARGS: &str =
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --lang=en-US --accept-lang=en-US";

/// Opens a new chat each time the app starts.
pub fn open(app: &AppHandle, site: &SiteSelectors) -> tauri::Result<WebviewWindow> {
    let url = site.new_chat_url.parse().map_err(tauri::Error::InvalidUrl)?;
    let win = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::External(url))
        .title("Onvox")
        .inner_size(1000.0, 800.0)
        .visible(false)
        .focused(false)
        .skip_taskbar(true)
        .background_throttling(BackgroundThrottlingPolicy::Disabled)
        .additional_browser_args(BROWSER_ARGS)
        .initialization_script(driver_script(&app.state::<crate::Session>().token))
        .on_page_load(|win, payload| {
            crate::page_loaded(win.app_handle(), payload.event() == tauri::webview::PageLoadEvent::Finished)
        })
        .build()?;
    crate::permission::install(&win)?;
    Ok(win)
}

fn call(win: &WebviewWindow, method: &str, site: &SiteSelectors) {
    // The selectors go in as a JSON value; their text is never run as code.
    let json = serde_json::to_string(site).unwrap_or_default();
    let _ = win.eval(format!("window.__onvox && window.__onvox.{method}({json})"));
}

pub fn check_login(win: &WebviewWindow, site: &SiteSelectors) {
    call(win, "checkLogin", site);
}

pub fn start_voice(win: &WebviewWindow, site: &SiteSelectors) {
    call(win, "start", site);
}

pub fn stop_voice(win: &WebviewWindow, site: &SiteSelectors) {
    call(win, "stop", site);
}

pub fn new_chat(win: &WebviewWindow, site: &SiteSelectors) {
    if let Ok(url) = site.new_chat_url.parse() {
        let _ = win.navigate(url);
    }
}

pub fn show_for_login(win: &WebviewWindow) {
    // A skip_taskbar window cannot be brought back once minimized, except from here.
    let _ = win.unminimize();
    let _ = win.show();
    let _ = win.set_focus();
}

pub fn hide(win: &WebviewWindow) {
    let _ = win.hide();
}

/// driver.js with this run's report token in it.
pub fn driver_script(token: &str) -> String {
    include_str!("driver.js").replace("__ONVOX_TOKEN__", token)
}

/// 128 random bits from the standard library's per-process hash keys.
// shortcut: RandomState is not a CSPRNG, enough to keep page scripts from guessing; use getrandom if it must resist more.
pub fn new_token() -> String {
    use std::hash::{BuildHasher, Hasher};
    let part = || std::collections::hash_map::RandomState::new().build_hasher().finish();
    format!("{:016x}{:016x}", part(), part())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_carries_the_token() {
        let s = driver_script("abc123");
        assert!(s.contains(r#"const TOKEN = "abc123";"#), "{s}");
        assert!(!s.contains("__ONVOX_TOKEN__"));
    }

    #[test]
    fn tokens_differ() {
        assert_ne!(new_token(), new_token());
        assert_eq!(new_token().len(), 32);
    }
}
