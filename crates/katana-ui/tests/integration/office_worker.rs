use std::process::Command;

#[test]
fn bundled_office_worker_delegates_to_kdv_entrypoint() {
    let output = Command::new(env!("CARGO_BIN_EXE_kdv-office-worker"))
        .output()
        .expect("bundled KDV office worker should start");

    assert_eq!(output.status.code(), Some(64));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("KDV office worker usage error: workspace argument is missing")
    );
}
