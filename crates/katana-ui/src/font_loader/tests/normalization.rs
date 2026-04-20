/* WHY: Verification of font normalization states and idempotent behavior. */

use super::*;

#[test]
fn test_normalize_fonts_is_normalized_state() {
    let raw = NormalizeFonts::new(FontDefinitions::default());
    assert!(!raw.is_normalized());
    let normalized = raw.normalize(&[]);
    assert!(normalized.is_normalized());
}

#[test]
fn test_normalize_fonts_double_normalize_is_noop() {
    let fonts = NormalizeFonts::new(FontDefinitions::default()).normalize(&[]);
    let family_before = fonts.fonts().families.clone();
    let fonts = fonts.normalize(&[]);
    assert_eq!(fonts.fonts().families, family_before);
}
