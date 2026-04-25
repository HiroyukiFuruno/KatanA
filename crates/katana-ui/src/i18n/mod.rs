mod logic;
mod types;

pub use types::*;

pub struct I18nOps;

#[cfg(test)]
mod tests {
    #[test]
    fn test_i18n_default_action_values() {
        /* WHY: NOTE: These defaults are used by serde when JSON keys are missing. */
        /* WHY: They are defined in types.rs and visible here as children of i18n mod. */
    }
}
