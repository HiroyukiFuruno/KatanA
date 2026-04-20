/* WHY: Verification of the volatile in-memory storage used for testing and transient sessions. */

use super::super::*;

#[test]
fn test_in_memory_repository_load_returns_defaults() {
    let repo = InMemoryRepository;
    let settings = repo.load();
    assert_eq!(settings.theme.theme, SettingsDefaultOps::default_theme());
}

#[test]
fn test_in_memory_repository_save_succeeds() {
    let repo = InMemoryRepository;
    let settings = AppSettings::default();
    assert!(repo.save(&settings).is_ok());
}
