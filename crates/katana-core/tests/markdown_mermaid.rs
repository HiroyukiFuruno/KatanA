use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::mermaid_renderer;
use katana_core::markdown::svg_rasterize::SvgRasterizeOps;
use std::sync::Mutex;
use std::thread;

static ENV_LOCK: Mutex<()> = Mutex::new(());
const RUST_MANAGED_GANTT_MAX_WIDTH: u32 = 1200;
const JAPANESE_FLOWCHART_SOURCE: &str =
    "flowchart TD\n    A[\u{958b}\u{59cb}] --> B{\u{78ba}\u{8a8d}}\n    B --> C[\u{5b8c}\u{4e86}]";
const JAPANESE_KANBAN_SOURCE: &str = concat!(
    "---\n",
    "config:\n",
    "  kanban:\n",
    "    ticketBaseUrl: 'https://github.com/mermaid-js/mermaid/issues/#TICKET#'\n",
    "---\n",
    "kanban\n",
    "  \u{672a}\u{7740}\u{624b}\n",
    "    [\u{30c9}\u{30ad}\u{30e5}\u{30e1}\u{30f3}\u{30c8}\u{4f5c}\u{6210}]\n",
    "  [\u{9032}\u{884c}\u{4e2d}]\n",
    "    id6[\u{3059}\u{3079}\u{3066}\u{306e}\u{5834}\u{5408}\u{306b}\u{52d5}\u{4f5c}\u{3059}\u{308b}\u{30ec}\u{30f3}\u{30c0}\u{30e9}\u{30fc}\u{3092}\u{4f5c}\u{6210}\u{3059}\u{308b}\u{3002}\u{8868}\u{793a}\u{78ba}\u{8a8d}\u{306e}\u{305f}\u{3081}\u{3001}\u{9577}\u{3081}\u{306e}\u{30c6}\u{30ad}\u{30b9}\u{30c8}\u{3082}\u{5165}\u{308c}\u{3066}\u{3044}\u{308b}\u{3002}]\n",
    "  id11[\u{5b8c}\u{4e86}]\n",
    "    id5[\u{30c7}\u{30fc}\u{30bf}\u{53d6}\u{5f97}\u{3092}\u{5b9a}\u{7fa9}]\n",
);

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

fn render_svg(source: String) -> String {
    let block = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source,
    };
    match mermaid_renderer::MermaidRenderOps::render_mermaid(&block) {
        DiagramResult::Ok(svg) => svg,
        other => panic!("Expected Mermaid SVG rendering, got {other:?}"),
    }
}

fn rasterized_dimensions(svg: &str) -> (u32, u32) {
    let image = SvgRasterizeOps::rasterize_svg(svg, 1.0).unwrap();
    (image.width, image.height)
}

#[test]
fn returns_not_installed_when_mermaid_js_is_missing() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
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
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let custom_path = std::path::PathBuf::from("/custom/mermaid.min.js");
    unsafe { std::env::set_var("MERMAID_JS", &custom_path) };

    let path = mermaid_renderer::MermaidBinaryOps::resolve_mermaid_js();

    unsafe { std::env::remove_var("MERMAID_JS") };
    assert_eq!(path, custom_path);
}

#[test]
fn resolve_mermaid_js_falls_back_to_default_install_path() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    unsafe { std::env::remove_var("MERMAID_JS") };

    let path = mermaid_renderer::MermaidBinaryOps::resolve_mermaid_js();

    assert!(!path.as_os_str().is_empty());
    assert!(path.to_string_lossy().contains("mermaid.min.js"));
}

#[test]
fn gantt_future_today_marker_does_not_expand_canvas() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }

    let with_marker = rasterized_dimensions(&render_svg(gantt_source("")));
    let without_marker = rasterized_dimensions(&render_svg(gantt_source("todayMarker off")));

    assert_eq!(with_marker, without_marker);
    assert!(
        with_marker.0 <= RUST_MANAGED_GANTT_MAX_WIDTH,
        "gantt width must stay close to mmdc output width, got {}",
        with_marker.0
    );
}

#[test]
fn japanese_flowchart_labels_render() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }

    let svg = render_svg(JAPANESE_FLOWCHART_SOURCE.to_string());

    assert!(svg.contains("\u{958b}\u{59cb}"));
    assert!(svg.contains("\u{78ba}\u{8a8d}"));
    assert!(svg.contains("\u{5b8c}\u{4e86}"));
}

#[test]
fn japanese_kanban_labels_render_without_native_segmenter_crash() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }

    let svg = render_svg(JAPANESE_KANBAN_SOURCE.to_string());

    assert!(svg.contains("\u{672a}\u{7740}\u{624b}"));
    assert!(svg.contains("\u{9032}\u{884c}\u{4e2d}"));
    assert!(svg.contains("\u{5b8c}\u{4e86}"));
}

#[test]
fn concurrent_mermaid_rendering_succeeds() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    if mermaid_renderer::MermaidBinaryOps::find_mermaid_js().is_none() {
        return;
    }

    let handles: Vec<_> = [
        "graph TD; A-->B",
        "sequenceDiagram\n  participant User\n  participant KatanA\n  User->>KatanA: Open",
        "classDiagram\n  class PreviewPane\n  PreviewPane --> RenderedSection",
        "stateDiagram-v2\n  [*] --> Pending\n  Pending --> Image : success",
    ]
    .into_iter()
    .map(|source| thread::spawn(move || render_svg(source.to_string())))
    .collect();

    for handle in handles {
        let svg = handle.join().unwrap();
        assert!(svg.contains("<svg"));
        let dimensions = rasterized_dimensions(&svg);
        assert!(dimensions.0 > 0);
        assert!(dimensions.1 > 0);
    }
}
