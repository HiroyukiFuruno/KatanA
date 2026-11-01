use super::*;
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use std::sync::Arc;

const CHILD: &str = "KATANA_UPDATE_FAILURE_CHILD";
const HELPER_TIMEOUT_SECS: u64 = 15;
#[cfg(unix)]
const HELPER_PERMISSIONS: u32 = 0o700;

fn create_app() -> KatanaApp {
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    KatanaApp::new(state)
}

#[test]
fn restart_actions_keep_app_open_on_actual_spawn_failure() {
    if let Some(mode) = std::env::var_os(CHILD) {
        run_failed_child(mode);
        return;
    }
    let mut failures = Vec::new();
    for mode in ["confirm", "install"] {
        if let Some(error) = run_failed_case(mode) {
            failures.push(error);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn run_failed_child(mode: std::ffi::OsString) {
    let mut app = create_app();
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().to_path_buf();
    app.pending_relaunch = Some(katana_core::update::UpdatePreparation {
        script_path: path.join("nonexistent-relauncher"),
        app_bundle_path: path.join("application"),
        temp_dir,
    });
    app.state.update.phase = Some(crate::app_state::UpdatePhase::ReadyToRelaunch);
    let action = if mode == "confirm" {
        AppAction::ConfirmRelaunch
    } else {
        AppAction::InstallUpdateAndRestart
    };
    app.dispatch_secondary(&egui::Context::default(), action);
    assert!(app.show_update_dialog);
    assert!(app.state.update.install_error.is_some());
    assert!(app.state.update.phase.is_none());
    assert!(!path.exists());
    assert_failure_is_visible(app);
    std::fs::write(
        std::env::var_os("KATANA_UPDATE_RESULT").unwrap(),
        "continued",
    )
    .unwrap();
}

fn run_failed_case(mode: &str) -> Option<String> {
    let isolated = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut command =
        katana_core::system::ProcessService::create_command(executable.to_str().unwrap());
    command
        .args(["--exact", "app::action::dispatch_secondary::update_tests::restart_actions_keep_app_open_on_actual_spawn_failure", "--nocapture"])
        .env(CHILD, mode)
        .env("KATANA_UPDATE_RESULT", isolated.path().join("result"))
        .env("KATANA_CONFIG_DIR", isolated.path());
    #[cfg(windows)]
    command
        .current_dir(isolated.path())
        .env("PATH", isolated.path());
    let output = command.output().unwrap();
    if output.status.success()
        && std::fs::read_to_string(isolated.path().join("result"))
            .ok()
            .as_deref()
            == Some("continued")
    {
        None
    } else {
        Some(format!(
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn assert_failure_is_visible(app: KatanaApp) {
    use egui_kittest::kittest::Queryable;
    let heading = crate::i18n::I18nOps::get().update.failed_to_install.clone();
    let message = crate::i18n::I18nOps::get()
        .update
        .update_install_error_unknown
        .clone();
    let mut open = true;
    let mut cache = egui_commonmark::CommonMarkCache::default();
    let mut action = AppAction::None;
    let mut harness = egui_kittest::Harness::builder().build_ui(move |ui| {
        crate::views::modals::update::UpdateModal::new(
            &mut open,
            &app.state,
            &mut cache,
            &mut action,
        )
        .show(ui.ctx());
    });
    harness.step();
    harness.get_by_label(&heading);
    harness.get_by_label(&message);
}

#[test]
fn restart_actions_exit_only_after_actual_helper_starts() {
    const SUCCESS_CHILD: &str = "KATANA_UPDATE_SUCCESS_CHILD";
    if let Some(mode) = std::env::var_os(SUCCESS_CHILD) {
        run_success_child(mode);
        panic!("successful helper startup must terminate the parent");
    }
    for mode in ["confirm", "install"] {
        let root = tempfile::tempdir().unwrap();
        let executable = std::env::current_exe().unwrap();
        let output = katana_core::system::ProcessService::create_command(executable.to_str().unwrap())
            .args(["--exact", "app::action::dispatch_secondary::update_tests::restart_actions_exit_only_after_actual_helper_starts", "--nocapture"])
            .env(SUCCESS_CHILD, mode)
            .env("KATANA_UPDATE_SUCCESS_ROOT", root.path())
            .env("KATANA_CONFIG_DIR", root.path())
            .output().unwrap();
        assert!(
            output.status.success(),
            "{mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        wait_for_helper(&root.path().join("helper-completed"));
    }
}

fn run_success_child(mode: std::ffi::OsString) {
    let root = std::path::PathBuf::from(std::env::var_os("KATANA_UPDATE_SUCCESS_ROOT").unwrap());
    let temp_dir = tempfile::tempdir_in(&root).unwrap();
    let script_path = temp_dir.path().join(if cfg!(windows) {
        "helper.ps1"
    } else {
        "helper.sh"
    });
    write_success_helper(&script_path, &root.join("helper-completed"));
    let mut app = create_app();
    app.pending_relaunch = Some(katana_core::update::UpdatePreparation {
        app_bundle_path: temp_dir.path().join("unused"),
        script_path,
        temp_dir,
    });
    let action = if mode == "confirm" {
        AppAction::ConfirmRelaunch
    } else {
        AppAction::InstallUpdateAndRestart
    };
    app.dispatch_secondary(&egui::Context::default(), action);
}

fn wait_for_helper(marker: &std::path::Path) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(HELPER_TIMEOUT_SECS);
    while std::fs::read_to_string(marker).ok().as_deref() != Some("parent exited")
        && std::time::Instant::now() < deadline
    {
        std::thread::yield_now();
    }
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "parent exited");
}

#[cfg(unix)]
fn write_success_helper(path: &std::path::Path, marker: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let marker = marker.to_string_lossy().replace('\'', "'\\''");
    let script = format!(
        "#!/bin/sh\nwhile kill -0 {} 2>/dev/null; do sleep 0.05; done\nprintf 'parent exited' > '{}'\n",
        std::process::id(),
        marker
    );
    std::fs::write(path, script).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(HELPER_PERMISSIONS)).unwrap();
}

#[cfg(windows)]
fn write_success_helper(path: &std::path::Path, marker: &std::path::Path) {
    let marker = marker.to_string_lossy().replace('\'', "''");
    let script = format!(
        "param($parentPid)\n$ErrorActionPreference = 'Stop'\ntry {{ $parent = [System.Diagnostics.Process]::GetProcessById([int]$parentPid) }} catch {{ if (($_.Exception -isnot [System.ArgumentException]) -and ($_.Exception.InnerException -isnot [System.ArgumentException])) {{ throw }}; $parent = $null }}\nif ($null -ne $parent -and -not $parent.WaitForExit(10000)) {{ throw 'parent still running' }}\ntry {{ $live = [System.Diagnostics.Process]::GetProcessById([int]$parentPid) }} catch {{ if (($_.Exception -isnot [System.ArgumentException]) -and ($_.Exception.InnerException -isnot [System.ArgumentException])) {{ throw }}; $live = $null }}\nif ($null -ne $live) {{ throw 'parent PID is still present' }}\n[System.IO.File]::WriteAllText('{}', 'parent exited')\n",
        marker
    );
    std::fs::write(path, script).unwrap();
}
