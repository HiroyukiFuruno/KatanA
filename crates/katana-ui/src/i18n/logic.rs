use super::*;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{LazyLock, OnceLock};

impl I18nOps {
    const FALLBACK_LANGUAGE: &'static str = "en";

    pub fn supported_languages() -> &'static [(String, String)] {
        static LANGS: OnceLock<Vec<(String, String)>> = OnceLock::new();
        LANGS.get_or_init(|| {
            let json = include_str!("../../locales/languages.json");
            let entries: Vec<LanguageEntry> =
                serde_json::from_str(json).expect("Failed to parse languages.json");
            entries.into_iter().map(|e| (e.code, e.name)).collect()
        })
    }

    pub fn set_language(lang: &str) {
        let resolved = Self::resolve_language(lang);
        {
            let mut current = write_guard(&CURRENT_LANGUAGE);
            *current = resolved.clone();
        }
        Self::update_cached_messages(&resolved);
    }

    pub fn resolve_language(lang: &str) -> String {
        if Self::is_supported_language(lang) {
            lang.to_string()
        } else {
            Self::FALLBACK_LANGUAGE.to_string()
        }
    }

    pub fn get_language() -> String {
        init_current_language();
        read_guard(&CURRENT_LANGUAGE).clone()
    }

    pub fn get() -> &'static I18nMessages {
        /* WHY: Fast path: Atomic pointer access */
        let ptr = CURRENT_MESSAGES_CACHED.load(Ordering::Relaxed);
        if !ptr.is_null() {
            unsafe { return &*ptr }
        }

        /* WHY: Slow path: Initialization */
        let lang = Self::get_language();
        Self::update_cached_messages(&lang)
    }

    fn update_cached_messages(lang: &str) -> &'static I18nMessages {
        let messages = get_messages_for_lang(lang);
        CURRENT_MESSAGES_CACHED.store(messages as *const _ as *mut _, Ordering::SeqCst);
        messages
    }

    pub fn tf(template: &str, params: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in params {
            result = result.replace(&format!("{{{key}}}"), value);
        }
        result
    }

    pub fn display_name(code: &str) -> String {
        Self::supported_languages()
            .iter()
            .find(|(c, _)| c == code)
            .map(|(_, n)| n.clone())
            .unwrap_or_else(|| "???".to_string())
    }

    fn is_supported_language(lang: &str) -> bool {
        Self::supported_languages()
            .iter()
            .any(|(code, _)| code == lang)
    }
}

/* WHY: LazyLock is required because parking_lot::RwLock cannot be const-initialized in a static context. */
static CURRENT_LANGUAGE: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new(String::new()));
static CURRENT_MESSAGES_CACHED: AtomicPtr<I18nMessages> = AtomicPtr::new(ptr::null_mut());

fn init_current_language() {
    let mut current = write_guard(&CURRENT_LANGUAGE);
    if current.is_empty() {
        *current = I18nOps::FALLBACK_LANGUAGE.to_string();
    }
}

struct DictionaryEntry {
    lang: String,
    messages: OnceLock<I18nMessages>,
}

static DICT: OnceLock<Vec<DictionaryEntry>> = OnceLock::new();

fn get_dictionary() -> &'static Vec<DictionaryEntry> {
    DICT.get_or_init(|| {
        I18nOps::supported_languages()
            .iter()
            .map(|(code, _)| DictionaryEntry {
                lang: code.clone(),
                messages: OnceLock::new(),
            })
            .collect()
    })
}

fn get_messages_for_lang(lang: &str) -> &'static I18nMessages {
    let resolved = I18nOps::resolve_language(lang);
    let dict = get_dictionary();
    let entry = dict
        .iter()
        .find(|e| e.lang == resolved)
        .expect("BUG: Supported language missing from dictionary.");

    entry.messages.get_or_init(|| {
        let json = match resolved.as_str() {
            "en" => include_str!("../../locales/en.json"),
            "ja" => include_str!("../../locales/ja.json"),
            "zh-CN" => include_str!("../../locales/zh-CN.json"),
            "zh-TW" => include_str!("../../locales/zh-TW.json"),
            "ko" => include_str!("../../locales/ko.json"),
            "pt" => include_str!("../../locales/pt.json"),
            "fr" => include_str!("../../locales/fr.json"),
            "de" => include_str!("../../locales/de.json"),
            "es" => include_str!("../../locales/es.json"),
            "it" => include_str!("../../locales/it.json"),
            _ => panic!("BUG: Unhandled language code: {lang}"),
        };
        parse_messages_for_lang(&resolved, json)
    })
}

fn parse_messages_for_lang(lang: &str, json: &str) -> I18nMessages {
    serde_json::from_str(json).unwrap_or_else(|e| panic!("BUG: {lang}.json is invalid: {e}"))
}

fn read_guard(lock: &RwLock<String>) -> RwLockReadGuard<'_, String> {
    lock.read()
}

fn write_guard(lock: &RwLock<String>) -> RwLockWriteGuard<'_, String> {
    lock.write()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LanguageGuard {
        previous: String,
    }

    impl LanguageGuard {
        fn capture() -> Self {
            Self {
                previous: I18nOps::get_language(),
            }
        }
    }

    impl Drop for LanguageGuard {
        fn drop(&mut self) {
            I18nOps::set_language(&self.previous);
        }
    }

    #[test]
    fn unknown_runtime_language_resolves_to_fallback() {
        assert_eq!(I18nOps::resolve_language("unknown-lang"), "en");
    }

    #[test]
    fn set_language_with_unknown_code_does_not_panic() {
        let _guard = LanguageGuard::capture();

        I18nOps::set_language("unknown-lang");

        assert_eq!(I18nOps::get_language(), "en");
        assert_eq!(I18nOps::get().menu.file, "File");
    }

    #[test]
    #[should_panic(expected = "BUG: broken.json is invalid")]
    fn invalid_embedded_locale_still_fails_fast() {
        let _ = parse_messages_for_lang("broken", "{");
    }
}
