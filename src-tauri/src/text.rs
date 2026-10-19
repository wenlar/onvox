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
}

pub fn t(key: Key, lang: Lang) -> &'static str {
    match (key, lang) {
        (Key::AppName, _) => "Onvox",
        (Key::SetupTitle, Lang::Tr) => "Onvox kurulumu",
        (Key::SetupTitle, Lang::En) => "Onvox setup",
    }
}
