//! "Report a problem": opens a prefilled GitHub issue. The user sees and sends it themselves.

use crate::config::Provider;
use crate::state::Phase;

/// Only the version, the site and the state go in: nothing about the user or their machine.
pub fn issue_url(version: &str, provider: Provider, phase: &Phase) -> String {
    let site = serde_json::to_value(provider).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
    let state = match phase {
        Phase::Idle => "idle".to_string(),
        Phase::Starting => "starting".to_string(),
        Phase::Listening => "listening".to_string(),
        Phase::Error(f) => {
            let fault = serde_json::to_value(f).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_default();
            format!("error: {fault}")
        }
    };
    let body = format!("What happened?\n\n\n- Onvox version: {version}\n- Site: {site}\n- State: {state}\n");
    let title = format!("Problem with {site}");
    tauri::Url::parse_with_params("https://github.com/wenlar/onvox/issues/new", [("title", title), ("body", body)])
        .map(String::from)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Fault;

    #[test]
    fn issue_url_encodes_and_has_no_extra_data() {
        let url = issue_url("0.1.0", Provider::Grok, &Phase::Error(Fault::VoiceButtonMissing));
        assert!(url.starts_with("https://github.com/wenlar/onvox/issues/new?title="), "{url}");
        assert!(!url.contains(' '), "{url}");
        let decoded = tauri::Url::parse(&url).unwrap();
        let body: String = decoded.query_pairs().find(|(k, _)| k == "body").unwrap().1.into();
        assert!(body.contains("0.1.0") && body.contains("grok") && body.contains("voice_button_missing"), "{body}");
        // Only version, site and state: nothing about the user or their machine.
        let keys: Vec<String> = decoded.query_pairs().map(|(k, _)| k.into_owned()).collect();
        assert_eq!(keys, ["title", "body"]);
        assert_eq!(body.lines().filter(|l| l.starts_with("- ")).count(), 3, "{body}");
    }
}
