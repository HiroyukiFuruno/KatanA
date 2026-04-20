/* WHY: Verification of the disk-based persistence using JSON files. */

use super::super::*;
use tempfile::TempDir;

#[test]
fn test_json_file_repository_save_and_load() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");
    let repo = JsonFileRepository::new(path);

    let settings = AppSettings {
        theme: ThemeSettings {
            theme: "light".to_string(),
            ..Default::default()
        },
        language: "ja".to_string(),
        ..Default::default()
    };
    repo.save(&settings).unwrap();

    let loaded = repo.load();
    assert_eq!(loaded.theme.theme, "light");
    assert_eq!(loaded.language, "ja");
}

#[test]
fn test_json_file_repository_load_missing_file_returns_defaults() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("nonexistent.json");
    let repo = JsonFileRepository::new(path);
    let settings = repo.load();
    assert_eq!(settings.theme.theme, SettingsDefaultOps::default_theme());
}

#[test]
fn test_json_file_repository_load_corrupt_file_returns_defaults() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("corrupt.json");
    std::fs::write(&path, "NOT VALID JSON").unwrap();
    let repo = JsonFileRepository::new(path.clone());
    let settings = repo.load();
    assert_eq!(settings.theme.theme, SettingsDefaultOps::default_theme());
}

#[test]
fn test_json_file_repository_creates_parent_dirs() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("nested").join("dir").join("settings.json");
    let repo = JsonFileRepository::new(path.clone());
    let settings = AppSettings::default();
    repo.save(&settings).unwrap();
    assert!(path.exists());
}

#[test]
fn test_json_file_repository_with_default_path() {
    let repo = JsonFileRepository::with_default_path();
    assert!(repo.path.ends_with("settings.json"));
}

#[test]
fn test_json_file_repository_save_bare_filename_no_parent() {
    let tmp = TempDir::new().unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let repo = JsonFileRepository::new(std::path::PathBuf::from("bare.json"));
    let settings = AppSettings::default();
    repo.save(&settings).unwrap();
    assert!(tmp.path().join("bare.json").exists());

    std::env::set_current_dir(cwd).unwrap();
}

#[test]
fn test_json_file_repository_save_create_dir_fails() {
    let tmp = TempDir::new().unwrap();
    let blocker = tmp.path().join("blocker");
    std::fs::write(&blocker, "I am a file").unwrap();

    let path = blocker.join("nested").join("settings.json");
    let repo = JsonFileRepository::new(path);
    let settings = AppSettings::default();
    let result = repo.save(&settings);
    assert!(
        result.is_err(),
        "save should fail when create_dir_all fails"
    );
}

#[test]
fn test_skipped_version_persistence_roundtrip() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");

    let mut settings = AppSettings::default();
    settings.updates.skipped_version = Some("v0.8.0".to_string());
    settings.updates.previous_app_version = Some("v0.7.0".to_string());
    let repo = JsonFileRepository::new(path.clone());
    repo.save(&settings).unwrap();

    let loaded = repo.load();
    assert_eq!(loaded.updates.skipped_version, Some("v0.8.0".to_string()));
    assert_eq!(
        loaded.updates.previous_app_version,
        Some("v0.7.0".to_string())
    );
}
