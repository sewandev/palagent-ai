use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Language {
    #[default]
    Es,
    En,
}

static ES: OnceLock<HashMap<String, String>> = OnceLock::new();
static EN: OnceLock<HashMap<String, String>> = OnceLock::new();
static CURRENT: AtomicU8 = AtomicU8::new(0); // 0 = Es, 1 = En

pub fn detect_system_language() -> Language {
    if let Some(locale) = sys_locale::get_locale() {
        if locale.to_ascii_lowercase().starts_with("es") {
            return Language::Es;
        }
    }
    Language::En
}

pub fn init(lang: Language) {
    let es_src = include_str!("../locales/es.json");
    let en_src = include_str!("../locales/en.json");

    let es_map: HashMap<String, String> =
        serde_json::from_str(es_src).expect("locales/es.json is invalid");
    let en_map: HashMap<String, String> =
        serde_json::from_str(en_src).expect("locales/en.json is invalid");

    let _ = ES.set(es_map);
    let _ = EN.set(en_map);
    set_language(lang);
}

pub fn set_language(lang: Language) {
    CURRENT.store(lang as u8, Ordering::Relaxed);
}

pub fn current_language() -> Language {
    if CURRENT.load(Ordering::Relaxed) == 1 {
        Language::En
    } else {
        Language::Es
    }
}

pub fn t(key: &str) -> String {
    let lang = current_language();
    let primary = match lang {
        Language::Es => ES.get(),
        Language::En => EN.get(),
    };
    if let Some(v) = primary.and_then(|m| m.get(key)) {
        return v.clone();
    }
    if lang != Language::Es {
        if let Some(v) = ES.get().and_then(|m| m.get(key)) {
            return v.clone();
        }
    } else {
        if let Some(v) = EN.get().and_then(|m| m.get(key)) {
            return v.clone();
        }
    }
    key.to_string()
}
