use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());
const PNG_WIDTH_START: usize = 16;
const PNG_WIDTH_END: usize = 20;
const PNG_HEIGHT_START: usize = 20;
const PNG_HEIGHT_END: usize = 24;
const MMDC_COMPATIBLE_GANTT_MAX_WIDTH: u32 = 1000;

fn mermaid_block() -> DiagramBlock {
    DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: "graph TD; A-->B".to_string(),
    }
}

fn gantt_source(today_marker: &str) -> String {
    format!(
        "gantt\n\
         title Katana schedule\n\
         dateFormat YYYY-MM-DD\n\
         {today_marker}\n\
         section Core\n\
         Markdown rendering :done, a1, 2026-01-04, 2026-01-17\n\
         Diagram support :a2, 2026-02-01, 2026-02-15\n\
         Preview pane :a3, 2026-01-18, 2026-02-15\n\
         section UI\n\
         Theme support :a4, 2026-03-01, 2026-03-28\n\
         section Test\n\
         Unit test :a5, 2026-02-01, 2026-02-15\n\
         Integration test :a6, 2026-03-01, 2026-03-28"
    )
}

fn render_png(source: String) -> Vec<u8> {
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source,
    };
    match mermaid_renderer::MermaidRenderOps::render_mermaid(&block) {
        DiagramResult::OkPng(bytes) => bytes,
        other => panic!("Expected Mermaid PNG rendering, got {other:?}"),
    }
}

fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    let width = u32::from_be_bytes(bytes[PNG_WIDTH_START..PNG_WIDTH_END].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[PNG_HEIGHT_START..PNG_HEIGHT_END].try_into().unwrap());
    (width, height)
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

#[test]
fn gantt_future_today_marker_does_not_expand_canvas() {
    let _guard = ENV_LOCK.lock().unwrap();
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }

    let with_marker = png_dimensions(&render_png(gantt_source("")));
    let without_marker = png_dimensions(&render_png(gantt_source("todayMarker off")));

    assert_eq!(with_marker, without_marker);
    assert!(
        with_marker.0 <= MMDC_COMPATIBLE_GANTT_MAX_WIDTH,
        "gantt width must stay close to mmdc output width, got {}",
        with_marker.0
    );
}
