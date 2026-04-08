/* WHY: OS-level locale detection.

Detects the current operating system locale and normalizes it to KatanA's internal representation.
Provides a unified cross-platform mechanism wrapping `sys-locale`. */

use sys_locale::get_locale;

/* WHY: Supported languages and their fallback policies. */
const PREFIX_MAP: &[(&str, &str)] = &[
    ("ja", "ja"),
    ("ko", "ko"),
    ("pt", "pt"),
    ("fr", "fr"),
    ("de", "de"),
    ("es", "es"),
    ("it", "it"),
];

pub struct OsLocaleOps;

impl OsLocaleOps {
    /* WHY: Returns the normalized OS default language string or None if it cannot be resolved.
    If the locale is successfully queried but falls back entirely in `resolve_locale_to_lang`,
    it will return `"en"`. */
    pub fn get_default_language() -> Option<String> {
        let locale = get_locale()?;
        Some(Self::resolve_locale_to_lang(&locale))
    }

    /* WHY: Normalizes standard BCP 47 locales (e.g. en-US, zh-Hant-TW, ja-JP) into KatanA's internal subset (e.g. en, zh-TW, ja). */
    pub(crate) fn resolve_locale_to_lang(locale: &str) -> String {
        let lower = locale.to_lowercase();

        if lower.starts_with("zh-hans") || lower.contains("hans") || lower.starts_with("zh-cn") {
            return "zh-CN".to_string();
        }
        if lower.starts_with("zh-hant")
            || lower.contains("hant")
            || lower.starts_with("zh-tw")
            || lower.starts_with("zh-hk")
        {
            return "zh-TW".to_string();
        }

        for &(prefix, lang) in PREFIX_MAP {
            if lower.starts_with(prefix) {
                return lang.to_string();
            }
        }
        "en".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_locale() {
        assert_eq!(OsLocaleOps::resolve_locale_to_lang("ja-JP"), "ja");
        assert_eq!(OsLocaleOps::resolve_locale_to_lang("ko-KR"), "ko");
        assert_eq!(OsLocaleOps::resolve_locale_to_lang("zh-Hans-CN"), "zh-CN");
        assert_eq!(OsLocaleOps::resolve_locale_to_lang("zh-TW"), "zh-TW");
        assert_eq!(OsLocaleOps::resolve_locale_to_lang("fr-CA"), "fr");
        assert_eq!(OsLocaleOps::resolve_locale_to_lang("unknown-locale"), "en"); /* Fallback */
    }
}
