use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

#[derive(Clone, Copy)]
enum WatchMode {
    RootRecursive,
    OldNonRecursive,
    NestedNonRecursive,
}

fn watch_paths(
    root: &Path,
    old: &Path,
    nested: &Path,
    modes: &[WatchMode],
) -> Vec<(PathBuf, RecursiveMode)> {
    modes
        .iter()
        .map(|mode| match mode {
            WatchMode::RootRecursive => (root.to_path_buf(), RecursiveMode::Recursive),
            WatchMode::OldNonRecursive => (old.to_path_buf(), RecursiveMode::NonRecursive),
            WatchMode::NestedNonRecursive => (nested.to_path_buf(), RecursiveMode::NonRecursive),
        })
        .collect()
}

fn setup_watcher(
    watch_modes: Vec<(PathBuf, RecursiveMode)>,
) -> (Option<RecommendedWatcher>, Vec<serde_json::Value>, bool) {
    if watch_modes.is_empty() {
        return (None, Vec::new(), true);
    }
    let (tx, _events) = mpsc::channel::<Result<Event, notify::Error>>();
    let mut watcher = RecommendedWatcher::new(
        move |event| {
            let _ = tx.send(event);
        },
        Config::default(),
    )
    .expect("native rename diagnostic watcher");
    let mut setup = Vec::new();
    for (path, mode) in watch_modes {
        let result = watcher.watch(&path, mode);
        setup.push(match result {
            Ok(()) => serde_json::json!({"path": path, "status": "ok"}),
            Err(error) => serde_json::json!({
                "path": path,
                "status": "error",
                "error_kind": format!("{:?}", error.kind),
                "error": format!("{error:?}"),
            }),
        });
    }
    let setup_ok = setup.iter().all(|entry| entry["status"] == "ok");
    (Some(watcher), setup, setup_ok)
}

fn rename_value(old: &Path, renamed: &Path, setup_ok: bool) -> serde_json::Value {
    if !setup_ok {
        return serde_json::json!({"status": "unexecuted", "reason": "watch setup failed"});
    }
    match std::fs::rename(old, renamed) {
        Ok(()) => serde_json::json!({"status": "success"}),
        Err(error) => serde_json::json!({
            "status": "error",
            "error_kind": format!("{:?}", error.kind()),
            "raw_os_error": error.raw_os_error(),
            "error": format!("{error:?}"),
        }),
    }
}

fn probe_directory_rename(label: &str, modes: &[WatchMode]) -> serde_json::Value {
    let root = tempfile::tempdir().expect("rename probe fixture");
    let old = root.path().join("old");
    let nested = old.join("nested");
    std::fs::create_dir_all(&nested).expect("rename probe nested directory");
    let watch_modes = watch_paths(root.path(), &old, &nested, modes);
    let (mut watcher, setup, setup_ok) = setup_watcher(watch_modes);
    let renamed = root.path().join("new");
    let rename = rename_value(&old, &renamed, setup_ok);
    let result = serde_json::json!({"label": label, "setup": setup, "rename": rename});
    eprintln!("windows rename probe {result}");
    drop(watcher.take());
    result
}

fn write_inventory_log(samples: &[serde_json::Value]) {
    let Some(inventory_root) = std::env::var_os("KATANA_TEST_INVENTORY_ROOT") else {
        return;
    };
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let inventory_root = PathBuf::from(inventory_root);
    let inventory_root = if inventory_root.is_absolute() {
        inventory_root
    } else {
        workspace_root.join(inventory_root)
    };
    std::fs::create_dir_all(&inventory_root).expect("diagnostic inventory directory");
    let contents = samples
        .iter()
        .map(|sample| serde_json::to_string(sample).expect("diagnostic JSON"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(
        inventory_root.join("windows-rename-watch-matrix.log"),
        contents,
    )
    .expect("diagnostic inventory log");
}

#[test]
fn windows_directory_rename_watch_matrix_reports_native_handle_constraints() {
    /* WHY: Windowsの実handle制約をfixtureの推測で置き換えず、各watch構成をfresh filesystemで記録する。 */
    let mut samples = vec![probe_directory_rename("none", &[])];
    for (label, modes) in [
        ("root-recursive", vec![WatchMode::RootRecursive]),
        ("old-nonrecursive", vec![WatchMode::OldNonRecursive]),
        ("nested-nonrecursive", vec![WatchMode::NestedNonRecursive]),
        (
            "root-old-nested",
            vec![
                WatchMode::RootRecursive,
                WatchMode::OldNonRecursive,
                WatchMode::NestedNonRecursive,
            ],
        ),
        (
            "production-old-nested",
            vec![WatchMode::OldNonRecursive, WatchMode::NestedNonRecursive],
        ),
    ] {
        samples.push(probe_directory_rename(label, &modes));
    }
    write_inventory_log(&samples);
    assert_eq!(samples[0]["rename"]["status"], "success");
    assert!(
        samples.iter().all(|sample| sample["setup"]
            .as_array()
            .is_some_and(|setup| { setup.iter().all(|entry| entry["status"] == "ok") })),
        "watch setup failure was recorded as an unusable diagnostic sample"
    );
}
