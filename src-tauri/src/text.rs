//! Every user-visible string, in Turkish and English. Later parts add keys as they need them.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Tr,
    En,
}

/// Turkish when the Windows UI language is Turkish, English otherwise.
pub fn lang() -> Lang {
    // LANG_TURKISH is 0x1f; the low 10 bits of a LANGID are the primary language.
    if unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() } & 0x3ff == 0x1f {
        Lang::Tr
    } else {
        Lang::En
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    AppName,
    SetupTitle,
    HotkeyTitle,
    StatusIdle,
    StatusLoading,
    StatusStarting,
    StatusListening,
    StatusNotLoggedIn,
    StatusVoiceButtonMissing,
    StatusLimitReached,
    StatusNoResponse,
    MenuNewChat,
    MenuHotkey,
    MenuRelogin,
    MenuCopyOverlay,
    MenuOverlayPortBusy,
    MenuQuit,
    MenuReport,
    MenuUpdate,
    /// "{v}" is replaced with the version.
    UpdateReady,
    /// "{v}" is replaced with the version.
    JustUpdated,
}

pub fn t(key: Key, lang: Lang) -> &'static str {
    match (key, lang) {
        (Key::AppName, _) => "Onvox",
        (Key::SetupTitle, Lang::Tr) => "Onvox kurulumu",
        (Key::SetupTitle, Lang::En) => "Onvox setup",
        (Key::HotkeyTitle, Lang::Tr) => "Kısayol tuşu",
        (Key::HotkeyTitle, Lang::En) => "Hotkey",
        (Key::StatusIdle, Lang::Tr) => "Hazır: konuşmak için kısayola bas",
        (Key::StatusIdle, Lang::En) => "Ready: press the hotkey to talk",
        (Key::StatusLoading, Lang::Tr) => "Sayfa yükleniyor…",
        (Key::StatusLoading, Lang::En) => "Loading the page…",
        (Key::StatusStarting, Lang::Tr) => "Bağlanıyor…",
        (Key::StatusStarting, Lang::En) => "Connecting…",
        (Key::StatusListening, Lang::Tr) => "Konuşuyor: bitirmek için kısayola bas",
        (Key::StatusListening, Lang::En) => "Talking: press the hotkey to stop",
        (Key::StatusNotLoggedIn, Lang::Tr) => "Giriş gerekli: \"Yeniden giriş yap\"a tıkla",
        (Key::StatusNotLoggedIn, Lang::En) => "Login needed: click \"Log in again\"",
        (Key::StatusVoiceButtonMissing, Lang::Tr) => "Sesli mod bulunamadı: site değişmiş olabilir",
        (Key::StatusVoiceButtonMissing, Lang::En) => "Voice mode not found: the site may have changed",
        (Key::StatusLimitReached, Lang::Tr) => "Sesli kullanım sınırına ulaşıldı",
        (Key::StatusLimitReached, Lang::En) => "Voice usage limit reached",
        (Key::StatusNoResponse, Lang::Tr) => "Site yanıt vermedi: tekrar dene",
        (Key::StatusNoResponse, Lang::En) => "The site didn't respond: try again",
        (Key::MenuNewChat, Lang::Tr) => "Yeni sohbet başlat",
        (Key::MenuNewChat, Lang::En) => "Start a new chat",
        (Key::MenuHotkey, Lang::Tr) => "Kısayol tuşunu değiştir…",
        (Key::MenuHotkey, Lang::En) => "Change hotkey…",
        (Key::MenuRelogin, Lang::Tr) => "Yeniden giriş yap",
        (Key::MenuRelogin, Lang::En) => "Log in again",
        (Key::MenuCopyOverlay, Lang::Tr) => "OBS gösterge adresini kopyala",
        (Key::MenuCopyOverlay, Lang::En) => "Copy OBS overlay address",
        (Key::MenuOverlayPortBusy, Lang::Tr) => "OBS göstergesi kapalı: 38517 portu başka bir uygulamada",
        (Key::MenuOverlayPortBusy, Lang::En) => "OBS overlay off: port 38517 is used by another app",
        (Key::MenuQuit, Lang::Tr) => "Çıkış",
        (Key::MenuQuit, Lang::En) => "Quit",
        (Key::MenuReport, Lang::Tr) => "Sorun bildir",
        (Key::MenuReport, Lang::En) => "Report a problem",
        (Key::MenuUpdate, Lang::Tr) => "Güncelle ve yeniden başlat",
        (Key::MenuUpdate, Lang::En) => "Update and restart",
        (Key::UpdateReady, Lang::Tr) => "{v} sürümü hazır",
        (Key::UpdateReady, Lang::En) => "Version {v} is ready",
        (Key::JustUpdated, Lang::Tr) => "Onvox {v} sürümüne güncellendi",
        (Key::JustUpdated, Lang::En) => "Updated to Onvox {v}",
    }
}
