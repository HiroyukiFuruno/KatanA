#[cfg(all(target_os = "macos", not(test)))]
const LOCALE_BUF_SIZE: usize = 32;

#[cfg(all(target_os = "macos", not(test)))]
pub(super) fn resolve_locale_to_lang(locale: &str) -> String {
    let lower = locale.to_lowercase();

    if lower.starts_with("zh-hans") || lower.contains("hans") {
        return "zh-CN".to_string();
    }
    if lower.starts_with("zh-hant")
        || lower.contains("hant")
        || lower.starts_with("zh-tw")
        || lower.starts_with("zh-hk")
    {
        return "zh-TW".to_string();
    }

    const PREFIX_MAP: &[(&str, &str)] = &[
        ("ja", "ja"),
        ("ko", "ko"),
        ("pt", "pt"),
        ("fr", "fr"),
        ("de", "de"),
        ("es", "es"),
        ("it", "it"),
    ];
    for &(prefix, lang) in PREFIX_MAP {
        if lower.starts_with(prefix) {
            return lang.to_string();
        }
    }
    "en".to_string()
}

#[cfg(not(test))]
pub(super) fn detect_initial_language() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        unsafe extern "C" {
            fn katana_get_mac_locale(buf: *mut std::ffi::c_char, max_len: usize);
        }
        let mut buf = [0u8; LOCALE_BUF_SIZE];
        unsafe { katana_get_mac_locale(buf.as_mut_ptr() as _, buf.len()) };
        let c_str = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as _) };
        let locale = c_str.to_string_lossy().to_string();
        return Some(resolve_locale_to_lang(&locale));
    }
    #[allow(unreachable_code)]
    None
}
