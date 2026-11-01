use katana_core::system::ProcessService;
use std::path::Path;

const SHORT_HISTORY: usize = 2;
const LONG_HISTORY: usize = 14;
const WRITERS: [&str; 3] = [
    "shell::tests_extra::handle_open_explorer_save_error_does_not_panic",
    "shell::tests_extra::test_poll_explorer_load_disconnect",
    "shell_ui::tests::terms_modal_advances_startup_heartbeat_without_acceptance",
];

#[test]
fn workspace_test_children_preserve_existing_profile() {
    for count in [SHORT_HISTORY, LONG_HISTORY] {
        let root = tempfile::tempdir().expect("isolated workspace fixture");
        let profile = root.path().join("profile");
        std::fs::create_dir(&profile).expect("create profile");
        let before = write_fixture(root.path(), &profile, count);
        let settings = katana_platform::settings::AppSettings::default();
        assert!(settings.terms_accepted_version.is_none());
        let settings_before = serde_json::to_vec(&settings).expect("serialize synthetic settings");
        std::fs::write(profile.join("settings.json"), &settings_before).expect("write settings");
        for name in WRITERS {
            run_test_child(name, &profile);
            let after = std::fs::read(profile.join("workspace.json")).expect("read profile");
            assert_eq!(before, after, "profile changed after {name}");
            let settings_after =
                std::fs::read(profile.join("settings.json")).expect("read settings");
            assert_eq!(
                settings_before, settings_after,
                "settings changed after {name}"
            );
        }
    }
}

fn write_fixture(root: &Path, profile: &Path, count: usize) -> Vec<u8> {
    let paths = (0..count)
        .map(|index| {
            let workspace = root.join(format!("workspace-{index}"));
            std::fs::create_dir(&workspace).expect("create synthetic workspace");
            workspace.to_string_lossy().into_owned()
        })
        .collect::<Vec<_>>();
    let state = serde_json::json!({
        "persisted": paths,
        "histories": paths,
        "open_workspace_tabs": &paths[count - SHORT_HISTORY..],
        "active_workspace": paths[count - 1],
    });
    let data = serde_json::to_vec(&state).expect("serialize synthetic profile");
    std::fs::write(profile.join("workspace.json"), &data).expect("write profile");
    data
}

fn run_test_child(name: &str, profile: &Path) {
    let executable = std::env::current_exe().expect("test executable");
    let output = ProcessService::create_command(executable.to_str().expect("UTF-8 executable"))
        .args(["--exact", name, "--test-threads=1"])
        .env("KATANA_CONFIG_DIR", profile)
        .output()
        .expect("run exact test child");
    assert!(
        output.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1 passed;"),
        "exact test was not executed: {name}"
    );
}
