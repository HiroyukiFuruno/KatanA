use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn mermaid_block() -> DiagramBlock {
    DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    }
}

#[test]
fn returns_not_installed_when_mermaid_js_is_missing() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let missing_path = dir.path().join("missing-mermaid.min.js");
    unsafe { std::env::set_var("MERMAID_JS", &missing_path) };

    let result = mermaid_renderer::MermaidRenderOps::render_mermaid(&mermaid_block());

    unsafe { std::env::remove_var("MERMAID_JS") };
    match result {
        DiagramResult::NotInstalled {
            kind,
            download_url,
            install_path,
        } => {
            assert_eq!(kind, "Mermaid");
            assert!(download_url.contains("mermaid.min.js"));
            assert_eq!(install_path, missing_path);
        }
        other => panic!("Expected NotInstalled when Mermaid.js is missing, got {other:?}"),
    }
}

#[test]
fn resolve_mermaid_js_prefers_env_var() {
    let _guard = ENV_LOCK.lock().unwrap();
    let custom_path = std::path::PathBuf::from("/custom/mermaid.min.js");
    unsafe { std::env::set_var("MERMAID_JS", &custom_path) };

    let path = mermaid_renderer::MermaidBinaryOps::resolve_mermaid_js();

    unsafe { std::env::remove_var("MERMAID_JS") };
    assert_eq!(path, custom_path);
}

#[test]
fn resolve_mermaid_js_falls_back_to_default_install_path() {
    let _guard = ENV_LOCK.lock().unwrap();
    unsafe { std::env::remove_var("MERMAID_JS") };

    let path = mermaid_renderer::MermaidBinaryOps::resolve_mermaid_js();

    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().contains("mermaid.min.js"));
}
