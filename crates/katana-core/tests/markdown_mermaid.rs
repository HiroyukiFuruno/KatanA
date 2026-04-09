use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn returns_commandnotfound_when_mmdc_not_found() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    };
    let result = mermaid_renderer::MermaidRenderOps::render_mermaid(&block);
    assert!(matches!(result, DiagramResult::CommandNotFound { .. }));
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn returns_its_path_when_resolve_mmdc_binary_env_var_is_set() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("MERMAID_MMDC", "/custom/mmdc") };
    let path = mermaid_renderer::MermaidBinaryOps::resolve_mmdc_binary();
    assert_eq!(path, std::path::PathBuf::from("/custom/mmdc"));
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn searches_system_path_when_resolve_mmdc_binary_env_var_is_not_set() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::remove_var("MERMAID_MMDC") };
    let path = mermaid_renderer::MermaidBinaryOps::resolve_mmdc_binary();
    assert!(!path.as_os_str().is_empty());
}

#[test]
fn create_input_file_creates_a_temporary_file() {
    let file = mermaid_renderer::MermaidRenderOps::create_input_file("graph TD; A-->B").unwrap();
    let path = file.path().to_path_buf();
    assert!(path.exists());
    assert!(path.to_string_lossy().ends_with(".mmd"));
}

#[test]
fn fake_binary_is_false_in_is_mmdc_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    assert!(!mermaid_renderer::MermaidRenderOps::is_mmdc_available());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
#[cfg(unix)]
fn hung_mmdc_probe_returns_false_instead_of_blocking() {
    use std::{fs, os::unix::fs::PermissionsExt};

    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("fake-mmdc.sh");
    fs::write(&script_path, "#!/bin/sh\nsleep 5\n").unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&script_path, permissions).unwrap();

    unsafe { std::env::set_var("MERMAID_MMDC", &script_path) };
    let start = Instant::now();
    assert!(!mermaid_renderer::MermaidRenderOps::is_mmdc_available());
    assert!(start.elapsed() < Duration::from_secs(4));
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}

#[test]
fn returns_png_correctly_if_mmdc_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    if std::env::var("MERMAID_MMDC").as_deref() == Ok("/nonexistent/mmdc") {
        return;
    }
    if !mermaid_renderer::MermaidRenderOps::is_mmdc_available() {
        return;
    }
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    };
    let result = mermaid_renderer::MermaidRenderOps::render_mermaid(&block);
    assert!(matches!(result, DiagramResult::OkPng(_)));
}

#[test]
fn run_mmdc_process_errors_when_mmdc_is_absent() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::set_var("MERMAID_MMDC", "/nonexistent/mmdc") };
    let result = mermaid_renderer::MermaidRenderOps::run_mmdc_process("graph TD; A-->B");
    assert!(result.is_err());
    unsafe { std::env::remove_var("MERMAID_MMDC") };
}
