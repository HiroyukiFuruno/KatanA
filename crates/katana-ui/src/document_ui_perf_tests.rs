use super::{DocumentUiPerf, MAX_EVENTS};

const CHILD: &str = "KATANA_DOCUMENT_UI_PERF_TEST_CHILD";
const TEST: &str = "document_ui_perf::tests::debug_only_timing_pairs_real_file_work";
const UNCHANGED_FRAMES: usize = 4;

#[test]
fn debug_only_timing_pairs_real_file_work() {
    if std::env::var_os(CHILD).is_some() {
        run_file_work();
        return;
    }
    for enabled in [false, true] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", TEST, "--nocapture"])
            .env(CHILD, "1")
            .env("DEBUG", enabled.to_string())
            .output()
            .unwrap();
        assert!(output.status.success());
        let log = String::from_utf8(output.stderr).unwrap();
        assert_eq!(log.contains("event=document_ui_perf"), enabled, "{log}");
        if enabled {
            assert!(log.contains("stage=begin phase=real_file_read"), "{log}");
            assert!(log.contains("stage=end phase=real_file_read"), "{log}");
            assert!(!log.contains("stage=begin phase=unchanged_target"), "{log}");
            assert!(log.contains("monotonic_us="), "{log}");
        }
    }
}

fn run_file_work() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("input.html");
    std::fs::write(&path, "<p>actual file input</p>").unwrap();
    {
        let _span = DocumentUiPerf::begin("real_file_read", Some(&path), true);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "<p>actual file input</p>"
        );
    }
    for _ in 0..UNCHANGED_FRAMES {
        let _span = DocumentUiPerf::begin("unchanged_target", Some(&path), false);
    }
}

#[test]
fn diagnostic_output_reserves_complete_pairs_with_a_fixed_limit() {
    const TEST: &str =
        "document_ui_perf::tests::diagnostic_output_reserves_complete_pairs_with_a_fixed_limit";
    if std::env::var_os(CHILD).is_some() {
        for _ in 0..MAX_EVENTS {
            let _span = DocumentUiPerf::begin("bounded_work", None, true);
        }
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env(CHILD, "1")
        .env("DEBUG", "true")
        .output()
        .unwrap();
    assert!(output.status.success());
    let log = String::from_utf8(output.stderr).unwrap();
    assert_eq!(log.matches("event=document_ui_perf").count(), MAX_EVENTS);
    assert_eq!(log.matches("stage=begin").count(), MAX_EVENTS / 2);
    assert_eq!(log.matches("stage=end").count(), MAX_EVENTS / 2);
}
