use super::extracted_file::{
    self, ExtractedFileFallback, LINUX_EXECUTABLE_NAME, WINDOWS_EXECUTABLE_NAME,
};
use super::*;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[test]
fn test_generate_relauncher_script() {
    let temp_dir = tempfile::tempdir().unwrap();
    let extracted_path = temp_dir.path().join("KatanA-extract.app");
    let target_path = temp_dir.path().join("KatanA.app");
    let script_path = temp_dir.path().join("relauncher.sh");

    UpdateInstallerOps::generate_relauncher_script(
        &extracted_path,
        &target_path,
        &script_path,
        temp_dir.path(),
    )
    .unwrap();

    assert!(script_path.exists());

    let content = std::fs::read_to_string(&script_path).unwrap();

    #[cfg(target_os = "macos")]
    {
        assert!(content.contains(&format!("TARGET_BAK=\"{}.bak\"", target_path.display())));
        assert!(content.contains(&format!("mv \"{}\" \"$TARGET_BAK\"", target_path.display())));
        assert!(content.contains(&format!(
            "if ! mv \"{}\" \"{}\"; then",
            extracted_path.display(),
            target_path.display()
        )));
        assert!(content.contains("display alert \"Update Failed\""));
        assert!(content.contains("Swap failed! Rolling back..."));
        assert!(content.contains(&format!("xattr -cr \"{}\"", target_path.display())));
        assert!(content.contains(&format!("rm -rf \"{}\"", temp_dir.path().display())));
    }

    #[cfg(target_os = "linux")]
    {
        assert!(content.contains(&format!("TARGET_BAK=\"{}.bak\"", target_path.display())));
        assert!(content.contains(&format!("mv \"{}\" \"$TARGET_BAK\"", target_path.display())));
        assert!(content.contains(&format!(
            "if mv \"{}\" \"{}\"",
            extracted_path.display(),
            target_path.display()
        )));
        assert!(content.contains(&format!("chmod +x \"{}\"", target_path.display())));
        assert!(content.contains(&format!("\"{}\" &", target_path.display())));
        assert!(content.contains(&format!("rm -rf \"{}\"", temp_dir.path().display())));
    }

    let perms = std::fs::metadata(&script_path).unwrap().permissions();
    assert_eq!(perms.mode() & 0o111, 0o111, "Script must be executable");
}

#[test]
fn resolve_extracted_file_prefers_linux_asset_name() {
    let temp_dir = tempfile::tempdir().unwrap();
    let expected_path = temp_dir.path().join(LINUX_EXECUTABLE_NAME);
    write_file_with_mode(&expected_path, 0o755);

    let actual = extracted_file::resolve_extracted_file(
        temp_dir.path(),
        LINUX_EXECUTABLE_NAME,
        ExtractedFileFallback::ExecutableRegularFile,
    )
    .unwrap();

    assert_eq!(actual, expected_path);
}

#[test]
fn resolve_extracted_file_prefers_windows_asset_name() {
    let temp_dir = tempfile::tempdir().unwrap();
    let expected_path = temp_dir.path().join(WINDOWS_EXECUTABLE_NAME);
    write_file_with_mode(&expected_path, 0o644);

    let actual = extracted_file::resolve_extracted_file(
        temp_dir.path(),
        WINDOWS_EXECUTABLE_NAME,
        ExtractedFileFallback::RegularFile,
    )
    .unwrap();

    assert_eq!(actual, expected_path);
}

#[test]
fn resolve_extracted_file_uses_single_executable_fallback() {
    let temp_dir = tempfile::tempdir().unwrap();
    let fallback_path = temp_dir.path().join("katana-desktop");
    write_file_with_mode(&fallback_path, 0o755);

    let actual = extracted_file::resolve_extracted_file(
        temp_dir.path(),
        LINUX_EXECUTABLE_NAME,
        ExtractedFileFallback::ExecutableRegularFile,
    )
    .unwrap();

    assert_eq!(actual, fallback_path);
}

#[test]
fn resolve_extracted_file_reports_empty_directory() {
    let temp_dir = tempfile::tempdir().unwrap();

    let error = extracted_file::resolve_extracted_file(
        temp_dir.path(),
        LINUX_EXECUTABLE_NAME,
        ExtractedFileFallback::ExecutableRegularFile,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains(LINUX_EXECUTABLE_NAME));
    assert!(error.contains("<empty>"));
}

#[test]
fn resolve_extracted_file_reports_multiple_candidates() {
    let temp_dir = tempfile::tempdir().unwrap();
    write_file_with_mode(&temp_dir.path().join("alpha"), 0o755);
    write_file_with_mode(&temp_dir.path().join("beta"), 0o755);

    let error = extracted_file::resolve_extracted_file(
        temp_dir.path(),
        LINUX_EXECUTABLE_NAME,
        ExtractedFileFallback::ExecutableRegularFile,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("alpha (file)"));
    assert!(error.contains("beta (file)"));
}

fn write_file_with_mode(path: &Path, mode: u32) {
    std::fs::write(path, "binary").unwrap();
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions).unwrap();
}
