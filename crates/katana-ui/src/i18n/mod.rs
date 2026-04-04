mod logic;
mod types;

pub use types::*;

pub struct I18nOps;

#[cfg(test)]
mod tests {
    use super::*;
    // removed unused panic and RwLock imports

    #[test]
    fn test_i18n_default_action_values() {
        // NOTE: These defaults are used by serde when JSON keys are missing.
        // They are defined in types.rs and visible here as children of i18n mod.
    }

    #[test]
    #[should_panic(expected = "BUG: Supported language missing from dictionary.")]
    fn test_get_panic_on_unsupported() {
        I18nOps::set_language("unsupported-lang-code");
        let _ = I18nOps::get();
    }
}
