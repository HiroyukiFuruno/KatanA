use katana_core::update::{UpdateInstallerOps, UpdatePreparation};
use std::path::Path;
use std::time::{Duration, Instant};

#[test]
fn successful_spawn_keeps_files_until_the_parent_exits() {
    const CHILD: &str = "KATANA_RELAUNCH_SUCCESS_CHILD";
    if let Some(directory) = std::env::var_os(CHILD) {
        run_successful_child(Path::new(&directory));
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "successful_spawn_keeps_files_until_the_parent_exits",
            "--nocapture",
        ])
        .env(CHILD, root.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let marker = root.path().join("helper-completed");
    let deadline = Instant::now() + Duration::from_secs(15);
    while std::fs::read_to_string(&marker).ok().as_deref() != Some("parent exited")
        && Instant::now() < deadline
    {
        std::thread::yield_now();
    }
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "parent exited");
}

fn run_successful_child(root: &Path) {
    let temp_dir = tempfile::tempdir_in(root).unwrap();
    let script_path = temp_dir.path().join(script_name());
    let marker = root.join("helper-completed");
    write_helper(&script_path, &marker);
    let preparation = UpdatePreparation {
        app_bundle_path: temp_dir.path().join("unused"),
        script_path,
        temp_dir,
    };
    UpdateInstallerOps::execute_relauncher(preparation).unwrap();
    panic!("successful startup must exit the parent");
}

#[cfg(unix)]
fn script_name() -> &'static str {
    "real-helper.sh"
}
#[cfg(windows)]
fn script_name() -> &'static str {
    "real-helper.ps1"
}

#[cfg(unix)]
fn write_helper(path: &Path, marker: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let path_text = marker.to_string_lossy().replace('\'', "'\\''");
    let script = format!(
        "#!/bin/sh\nwhile kill -0 {} 2>/dev/null; do sleep 0.05; done\nprintf 'parent exited' > '{}'\n",
        std::process::id(),
        path_text
    );
    std::fs::write(path, script).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

#[cfg(windows)]
fn write_helper(path: &Path, marker: &Path) {
    let path_text = marker.to_string_lossy().replace('\'', "''");
    let script = format!(
        "param($parentPid)\n$ErrorActionPreference = 'Stop'\ntry {{ $parent = [System.Diagnostics.Process]::GetProcessById([int]$parentPid) }} catch {{ if (($_.Exception -isnot [System.ArgumentException]) -and ($_.Exception.InnerException -isnot [System.ArgumentException])) {{ throw }}; $parent = $null }}\nif ($null -ne $parent -and -not $parent.WaitForExit(10000)) {{ throw 'parent still running' }}\ntry {{ $live = [System.Diagnostics.Process]::GetProcessById([int]$parentPid) }} catch {{ if (($_.Exception -isnot [System.ArgumentException]) -and ($_.Exception.InnerException -isnot [System.ArgumentException])) {{ throw }}; $live = $null }}\nif ($null -ne $live) {{ throw 'parent PID is still present' }}\n[System.IO.File]::WriteAllText('{}', 'parent exited')\n",
        path_text
    );
    std::fs::write(path, script).unwrap();
}
