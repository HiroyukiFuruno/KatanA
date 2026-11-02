use crate::rules::ReleaseScriptOps;
use std::path::Path;

const VALID: &str = "git commit -S -m \"chore: v${TARGET_VERSION}\u{306e} (#${KATANA_RELEASE_ISSUE})\" -- Cargo.toml Cargo.lock \"$INFO_PLIST\"";

fn write_file(path: impl AsRef<Path>, content: &str) {
    let path = path.as_ref();
    std::fs::create_dir_all(path.parent().expect("test directory")).expect("create test directory");
    std::fs::write(path, content).expect("write test fixture");
}

fn lint_script(content: &str) -> Vec<crate::Violation> {
    let root = tempfile::tempdir().expect("temporary directory");
    write_file(root.path().join("scripts/release/bump-version.sh"), content);
    ReleaseScriptOps::lint_ci_integrity(root.path())
}

#[test]
fn detects_ci_bypass_markers_in_release_automation() {
    let root = tempfile::tempdir().expect("temporary directory");
    write_file(
        root.path().join("scripts/release/bump-version.sh"),
        &format!("{VALID}\n# [skip ci]\n"),
    );
    write_file(
        root.path().join(".github/workflows/release.yml"),
        "message: [ci skip]\n",
    );
    let violations = ReleaseScriptOps::lint_ci_integrity(root.path());
    assert_eq!(violations.len(), 2);
    assert!(
        violations
            .iter()
            .all(|item| item.message.contains("release-ci-integrity"))
    );
}

#[test]
fn accepts_the_actual_bump_script_and_the_required_command() {
    assert!(lint_script(VALID).is_empty());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = std::fs::read_to_string(root.join("scripts/release/bump-version.sh"))
        .expect("actual release script");
    assert!(lint_script(&source).is_empty());
}

#[test]
fn rejects_hooks_bypass_and_missing_required_fields() {
    for invalid in [
        VALID.replace("commit -S", "commit -S -n"),
        VALID.replace("commit -S", "commit --no-verify -S"),
        VALID.replace(" (#${KATANA_RELEASE_ISSUE})", ""),
        VALID.replace("v${TARGET_VERSION}", "v${OTHER_VERSION}"),
        VALID.replace("Cargo.toml Cargo.lock", "Cargo.toml"),
        VALID.replace("\u{306e}", "${MUTABLE}"),
    ] {
        assert_eq!(lint_script(&invalid).len(), 1, "{invalid}");
    }
}

#[test]
fn rejects_comment_echo_and_multiple_commit_commands() {
    for invalid in [
        format!("# {VALID}"),
        format!("echo '{VALID}'"),
        format!("{VALID}\ngit commit --no-verify -m other"),
        format!("{VALID}\ngit -C . commit -m other"),
        format!("{VALID}\ngit \\\n commit -m other"),
        format!("{VALID}; git commit -m other"),
    ] {
        assert_eq!(lint_script(&invalid).len(), 1, "{invalid}");
    }
}
