use katana_core::update::{UpdateInstallerOps, UpdatePreparation};

#[test]
fn failed_actual_spawn_removes_prepared_directory() {
    const CHILD: &str = "KATANA_RELAUNCH_FAILURE_CHILD";
    if std::env::var_os(CHILD).is_some() {
        run_failed_child();
        return;
    }
    let isolated = tempfile::tempdir().unwrap();
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "failed_actual_spawn_removes_prepared_directory",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("KATANA_RELAUNCH_RESULT", isolated.path().join("result"));
    #[cfg(windows)]
    command
        .current_dir(isolated.path())
        .env("PATH", isolated.path());
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        std::fs::read_to_string(isolated.path().join("result")).unwrap(),
        "continued"
    );
}

fn run_failed_child() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().to_path_buf();
    let preparation = UpdatePreparation {
        script_path: path.join("missing-relauncher"),
        app_bundle_path: path.join("application"),
        temp_dir,
    };
    let error = UpdateInstallerOps::execute_relauncher(preparation).unwrap_err();
    assert_eq!(
        error.downcast_ref::<std::io::Error>().unwrap().kind(),
        std::io::ErrorKind::NotFound
    );
    assert!(
        !path.exists(),
        "failed startup must remove the prepared files"
    );
    std::fs::write(
        std::env::var_os("KATANA_RELAUNCH_RESULT").unwrap(),
        "continued",
    )
    .unwrap();
}
