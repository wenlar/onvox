use std::time::Duration;

use serde::Deserialize;

/// How long the page gets to report that voice mode started before it counts as not responding.
pub const START_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fault {
    NotLoggedIn,
    VoiceButtonMissing,
    LimitReached,
    NoResponse,
}

/// What the page script tells the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", content = "fault", rename_all = "snake_case")]
pub enum PageReport {
    VoiceStarted,
    VoiceEnded,
    Failed(Fault),
    /// Answers to a login check after the user picks a site; they never touch the voice session.
    LoggedIn,
    LoginNeeded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Idle,
    Starting,
    Listening,
    Error(Fault),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Hotkey,
    Report(PageReport),
    StartTimeout,
    NewChat,
    /// The page reloaded or its renderer died; any voice in it is gone.
    PageLost,
}

/// Only errors beep: the sites already chime when voice starts and stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Beep {
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// The site page is loading; set by the app, never by `step`.
    Loading,
    Idle,
    Active,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Beep(Beep),
    Icon(Icon),
    StartVoice,
    StopVoice,
}

/// The whole session logic: no I/O here, the caller carries out the effects in order.
pub fn step(phase: &Phase, ev: Event) -> (Phase, Vec<Effect>) {
    use Effect::{StartVoice, StopVoice};
    use Phase::*;
    let stopped = || vec![StopVoice, Effect::Icon(Icon::Idle)];
    let failed = |f| (Error(f), vec![StopVoice, Effect::Beep(Beep::Error), Effect::Icon(Icon::Error)]);
    match (phase, ev) {
        (Idle | Error(_), Event::Hotkey) => (Starting, vec![Effect::Icon(Icon::Active), StartVoice]),
        (Starting, Event::Report(PageReport::VoiceStarted)) => (Listening, vec![]),
        (Starting | Listening, Event::Hotkey | Event::NewChat) => (Idle, stopped()),
        (Starting | Listening, Event::PageLost) => (Idle, vec![Effect::Icon(Icon::Idle)]),
        (Listening, Event::Report(PageReport::VoiceEnded)) => (Idle, vec![Effect::Icon(Icon::Idle)]),
        (Starting | Listening, Event::Report(PageReport::Failed(f))) => failed(f),
        (Starting, Event::StartTimeout) => failed(Fault::NoResponse),
        // Voice came up after the user already cancelled: turn it off again.
        (Idle | Error(_), Event::Report(PageReport::VoiceStarted)) => (*phase, vec![StopVoice]),
        _ => (*phase, vec![]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Effect::{StartVoice, StopVoice};

    #[test]
    fn hotkey_from_idle_starts() {
        assert_eq!(step(&Phase::Idle, Event::Hotkey), (Phase::Starting, vec![Effect::Icon(Icon::Active), StartVoice]));
    }

    #[test]
    fn voice_started_goes_listening() {
        assert_eq!(step(&Phase::Starting, Event::Report(PageReport::VoiceStarted)), (Phase::Listening, vec![]));
    }

    #[test]
    fn hotkey_while_listening_stops() {
        assert_eq!(step(&Phase::Listening, Event::Hotkey), (Phase::Idle, vec![StopVoice, Effect::Icon(Icon::Idle)]));
    }

    #[test]
    fn voice_ended_while_listening_goes_idle() {
        assert_eq!(
            step(&Phase::Listening, Event::Report(PageReport::VoiceEnded)),
            (Phase::Idle, vec![Effect::Icon(Icon::Idle)])
        );
    }

    #[test]
    fn late_voice_started_after_cancel_stops_again() {
        assert_eq!(step(&Phase::Idle, Event::Report(PageReport::VoiceStarted)), (Phase::Idle, vec![StopVoice]));
    }

    #[test]
    fn not_logged_in_report_is_error_without_window() {
        let (phase, effects) = step(&Phase::Starting, Event::Report(PageReport::Failed(Fault::NotLoggedIn)));
        assert_eq!(phase, Phase::Error(Fault::NotLoggedIn));
        // Only sound and icon: the game must keep focus, so no window opens on its own.
        assert_eq!(effects, vec![StopVoice, Effect::Beep(Beep::Error), Effect::Icon(Icon::Error)]);
    }

    #[test]
    fn timeout_while_starting_is_no_response() {
        assert_eq!(
            step(&Phase::Starting, Event::StartTimeout),
            (Phase::Error(Fault::NoResponse), vec![StopVoice, Effect::Beep(Beep::Error), Effect::Icon(Icon::Error)])
        );
    }

    #[test]
    fn timeout_after_listening_is_ignored() {
        assert_eq!(step(&Phase::Listening, Event::StartTimeout), (Phase::Listening, vec![]));
    }

    #[test]
    fn hotkey_from_error_retries() {
        assert_eq!(
            step(&Phase::Error(Fault::LimitReached), Event::Hotkey),
            (Phase::Starting, vec![Effect::Icon(Icon::Active), StartVoice])
        );
    }

    #[test]
    fn page_lost_while_talking_goes_idle() {
        // A reload or a renderer crash took the voice with it: nothing left to stop.
        for phase in [Phase::Starting, Phase::Listening] {
            assert_eq!(step(&phase, Event::PageLost), (Phase::Idle, vec![Effect::Icon(Icon::Idle)]));
        }
        assert_eq!(step(&Phase::Idle, Event::PageLost), (Phase::Idle, vec![]));
    }

    #[test]
    fn login_reports_leave_phase_alone() {
        for report in [PageReport::LoggedIn, PageReport::LoginNeeded] {
            assert_eq!(step(&Phase::Listening, Event::Report(report)), (Phase::Listening, vec![]));
            assert_eq!(step(&Phase::Idle, Event::Report(report)), (Phase::Idle, vec![]));
        }
    }

    #[test]
    fn report_json_shapes() {
        let p = |s: &str| serde_json::from_str::<PageReport>(s).unwrap();
        assert_eq!(p(r#"{"kind":"logged_in"}"#), PageReport::LoggedIn);
        assert_eq!(p(r#"{"kind":"login_needed"}"#), PageReport::LoginNeeded);
        let p = |s: &str| serde_json::from_str::<PageReport>(s).unwrap();
        assert_eq!(p(r#"{"kind":"voice_started"}"#), PageReport::VoiceStarted);
        assert_eq!(p(r#"{"kind":"voice_ended"}"#), PageReport::VoiceEnded);
        assert_eq!(p(r#"{"kind":"failed","fault":"not_logged_in"}"#), PageReport::Failed(Fault::NotLoggedIn));
        assert_eq!(
            p(r#"{"kind":"failed","fault":"voice_button_missing"}"#),
            PageReport::Failed(Fault::VoiceButtonMissing)
        );
    }
}
