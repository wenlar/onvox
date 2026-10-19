//! CSS selectors and texts for each site. The copy on GitHub wins, so a site change is fixed without a release.

use std::time::Duration;

use serde::Deserialize;

use crate::config::Provider;

const REMOTE_URL: &str = "https://raw.githubusercontent.com/wenlar/onvox/main/selectors.json";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, serde::Serialize)]
pub struct SiteSelectors {
    pub home_url: String,
    pub new_chat_url: String,
    pub start_voice: Vec<String>,
    pub stop_voice: Vec<String>,
    pub voice_active: Vec<String>,
    pub logged_out: Vec<String>,
    pub limit_texts: Vec<String>,
    /// Cookie banner buttons, tried in order: the voice mode stays off until the banner is answered.
    #[serde(default)]
    pub consent_dismiss: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Selectors {
    pub schema: u32,
    /// `None`: the provider is not offered.
    #[serde(default)]
    pub chatgpt: Option<SiteSelectors>,
    #[serde(default)]
    pub grok: Option<SiteSelectors>,
    #[serde(default)]
    pub claude: Option<SiteSelectors>,
}

impl Selectors {
    pub fn site(&self, p: Provider) -> Option<&SiteSelectors> {
        match p {
            Provider::ChatGpt => self.chatgpt.as_ref(),
            Provider::Grok => self.grok.as_ref(),
            Provider::Claude => self.claude.as_ref(),
        }
    }
}

pub fn parse(json: &str) -> Result<Selectors, String> {
    let s: Selectors = serde_json::from_str(json).map_err(|e| e.to_string())?;
    if s.schema != 1 {
        return Err(format!("unknown schema {}", s.schema));
    }
    for site in [&s.chatgpt, &s.grok, &s.claude].into_iter().flatten() {
        if site.start_voice.is_empty() || site.stop_voice.is_empty() || site.voice_active.is_empty() {
            return Err("a site has no voice selectors".into());
        }
        if ![&site.home_url, &site.new_chat_url].iter().all(|u| u.starts_with("https://")) {
            return Err("a site URL is not https".into());
        }
    }
    Ok(s)
}

pub fn embedded() -> Selectors {
    parse(include_str!("../../selectors.json")).expect("embedded selectors.json is valid")
}

pub fn load() -> Selectors {
    // A debug build tries the local selectors.json, not what main has published.
    if cfg!(debug_assertions) {
        return embedded();
    }
    load_from(REMOTE_URL)
}

fn load_from(url: &str) -> Selectors {
    let fetch = || -> Result<Selectors, String> {
        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build();
        let body = client.and_then(|c| c.get(url).send()?.error_for_status()?.text()).map_err(|e| e.to_string())?;
        parse(&body)
    };
    fetch().unwrap_or_else(|_| embedded())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site_json(start: &str, home: &str) -> String {
        format!(
            r#"{{"home_url":"{home}","new_chat_url":"https://x.test/","start_voice":[{start}],
               "stop_voice":["b"],"voice_active":["b"],"logged_out":[],"limit_texts":[]}}"#
        )
    }

    #[test]
    fn embedded_parses() {
        let s = embedded();
        assert_eq!(s.schema, 1);
        assert!(s.site(Provider::ChatGpt).is_some());
        assert!(s.site(Provider::Grok).is_some());
        assert!(s.site(Provider::Claude).is_some());
    }

    #[test]
    fn wrong_schema_rejected() {
        let json = format!(r#"{{"schema":2,"chatgpt":{}}}"#, site_json(r#""a""#, "https://x.test/"));
        assert!(parse(&json).is_err());
    }

    #[test]
    fn empty_start_voice_rejected() {
        let json = format!(r#"{{"schema":1,"chatgpt":{}}}"#, site_json("", "https://x.test/"));
        assert!(parse(&json).is_err());
    }

    #[test]
    fn non_https_url_rejected() {
        let json = format!(r#"{{"schema":1,"grok":{}}}"#, site_json(r#""a""#, "http://x.test/"));
        assert!(parse(&json).is_err());
    }

    #[test]
    fn missing_site_is_none() {
        let json = format!(r#"{{"schema":1,"chatgpt":{}}}"#, site_json(r#""a""#, "https://x.test/"));
        let s = parse(&json).unwrap();
        assert!(s.site(Provider::ChatGpt).is_some());
        assert!(s.site(Provider::Grok).is_none());
    }

    #[test]
    fn consent_is_optional_and_set_for_grok() {
        let json = format!(r#"{{"schema":1,"chatgpt":{}}}"#, site_json(r#""a""#, "https://x.test/"));
        assert!(parse(&json).unwrap().chatgpt.unwrap().consent_dismiss.is_empty());
        assert!(!embedded().grok.unwrap().consent_dismiss.is_empty());
    }

    #[test]
    fn consent_lists_have_no_generic_buttons() {
        // A bare "Accept"/"Decline" could answer a terms or login dialog for the user.
        let s = embedded();
        for site in [s.chatgpt.unwrap(), s.grok.unwrap(), s.claude.unwrap()] {
            for entry in &site.consent_dismiss {
                assert!(!["text:Accept", "text:Decline"].contains(&entry.as_str()), "{entry}");
            }
        }
    }

    #[test]
    fn remote_failure_falls_back_to_embedded() {
        assert_eq!(load_from("http://127.0.0.1:1/selectors.json"), embedded());
    }
}
