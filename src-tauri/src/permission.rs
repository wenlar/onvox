//! Lets the provider page use the microphone without a prompt; everything else keeps WebView2's default.

use tauri::{Url, WebviewWindow};
pub use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_KIND_CAMERA, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
};

const HOSTS: [&str; 3] = ["chatgpt.com", "grok.com", "claude.ai"];

/// Only the microphone, only over https, only on the three sites and their subdomains.
pub fn is_allowed(uri: &str, kind: COREWEBVIEW2_PERMISSION_KIND) -> bool {
    let Ok(url) = Url::parse(uri) else { return false };
    let host = url.host_str().unwrap_or("");
    kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE
        && url.scheme() == "https"
        && HOSTS.iter().any(|h| host == *h || host.ends_with(&format!(".{h}")))
}

pub fn install(win: &WebviewWindow) -> tauri::Result<()> {
    use webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_PERMISSION_STATE_ALLOW;
    use webview2_com::{take_pwstr, PermissionRequestedEventHandler};
    win.with_webview(|pw| unsafe {
        let Ok(core) = pw.controller().CoreWebView2() else { return };
        let handler = PermissionRequestedEventHandler::create(Box::new(|_, args| {
            if let Some(args) = args {
                let mut kind = COREWEBVIEW2_PERMISSION_KIND::default();
                args.PermissionKind(&mut kind)?;
                let mut uri = windows_core::PWSTR::null();
                args.Uri(&mut uri)?;
                if is_allowed(&take_pwstr(uri), kind) {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)?;
                }
            }
            Ok(())
        }));
        let mut token = 0i64;
        let _ = core.add_PermissionRequested(&handler, &mut token);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mic_allowed_on_chatgpt() {
        assert!(is_allowed("https://chatgpt.com/c/123", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
    }

    #[test]
    fn mic_allowed_on_grok_subdomain() {
        assert!(is_allowed("https://assets.grok.com/x", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
    }

    #[test]
    fn mic_allowed_on_claude() {
        assert!(is_allowed("https://claude.ai/new", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
        assert!(!is_allowed("https://claude.ai.evil.com/", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
    }

    #[test]
    fn mic_denied_on_other_host() {
        assert!(!is_allowed("https://evil.com/", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
        assert!(!is_allowed("https://chatgpt.com.evil.com/", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
        assert!(!is_allowed("https://notchatgpt.com/", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
    }

    #[test]
    fn camera_denied_on_chatgpt() {
        assert!(!is_allowed("https://chatgpt.com/", COREWEBVIEW2_PERMISSION_KIND_CAMERA));
    }

    #[test]
    fn http_denied() {
        assert!(!is_allowed("http://chatgpt.com/", COREWEBVIEW2_PERMISSION_KIND_MICROPHONE));
    }
}
