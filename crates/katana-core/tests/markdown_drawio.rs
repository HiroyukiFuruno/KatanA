use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::DrawioRenderOps;
use std::sync::Mutex;
use std::thread;

#[path = "support/drawio_fixtures.rs"]
mod drawio_fixtures;

use drawio_fixtures::{
    COMPRESSED_DRAWIO_XML, IMAGE_DRAWIO_XML, OFFICIAL_STENCIL_DRAWIO_XML, SIMPLE_DRAWIO_XML,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn returns_not_installed_without_drawio_js() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let missing_path = dir.path().join("missing-drawio.min.js");
    unsafe { std::env::set_var("DRAWIO_JS", &missing_path) };

    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: SIMPLE_DRAWIO_XML.replace("test", "test1"),
    };
    let result = DrawioRenderOps::render_drawio(&block);

    unsafe { std::env::remove_var("DRAWIO_JS") };
    match result {
        DiagramResult::NotInstalled {
            kind, install_path, ..
        } => {
            assert_eq!(kind, "Draw.io");
            assert_eq!(install_path, missing_path);
        }
        other => panic!("Expected Draw.io NotInstalled, got {other:?}"),
    }
}

#[test]
fn resolve_drawio_js_prefers_env_var() {
    let _guard = ENV_LOCK.lock().unwrap();
    let custom_path = std::path::PathBuf::from("/custom/drawio.min.js");
    unsafe { std::env::set_var("DRAWIO_JS", &custom_path) };

    let path = DrawioRenderOps::resolve_drawio_js();

    unsafe { std::env::remove_var("DRAWIO_JS") };
    assert_eq!(path, custom_path);
}

#[test]
fn concurrent_drawio_renders_without_drawio_js() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    unsafe { std::env::set_var("DRAWIO_JS", dir.path().join("missing-drawio.min.js")) };

    let mut handles = vec![];
    for i in 0..3 {
        handles.push(thread::spawn(move || {
            let block = DiagramBlock {
                kind: DiagramKind::DrawIo,
                source: SIMPLE_DRAWIO_XML.replace("test", &format!("test_concurrent_{}", i)),
            };
            let result = DrawioRenderOps::render_drawio(&block);
            assert!(matches!(result, DiagramResult::NotInstalled { .. }));
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    unsafe { std::env::remove_var("DRAWIO_JS") };
}

#[test]
fn fake_drawio_js_does_not_fallback_to_native_svg() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let drawio_js = dir.path().join("drawio.min.js");
    std::fs::write(&drawio_js, "window.GraphViewer = {};").unwrap();
    unsafe { std::env::set_var("DRAWIO_JS", &drawio_js) };

    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: SIMPLE_DRAWIO_XML.replace("test", "test_managed_chromium_missing"),
    };
    let result = DrawioRenderOps::render_drawio(&block);
    unsafe { std::env::remove_var("DRAWIO_JS") };

    assert!(matches!(result, DiagramResult::Err { .. }));
}

fn render_with_official_drawio_js(source: &str) -> Option<DiagramResult> {
    let drawio_js = DrawioRenderOps::find_drawio_js()?;
    unsafe { std::env::set_var("DRAWIO_JS", drawio_js) };
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: source.to_string(),
    };
    let result = DrawioRenderOps::render_drawio(&block);
    unsafe { std::env::remove_var("DRAWIO_JS") };
    Some(result)
}

fn expect_drawio_svg(result: DiagramResult) -> String {
    match result {
        DiagramResult::Ok(svg) => svg,
        other => panic!("Expected Draw.io SVG, got {other:?}"),
    }
}

#[test]
fn renders_when_official_drawio_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some(result) = render_with_official_drawio_js(
        &SIMPLE_DRAWIO_XML.replace("test", "test_drawio_js_runtime"),
    ) else {
        return;
    };
    let svg = expect_drawio_svg(result);
    assert!(svg.contains("Box A"), "{svg}");
    assert!(svg.contains("Box B"), "{svg}");
}

#[test]
fn renders_parentless_drawio_geometry_without_source_crop_crash_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    let source = SIMPLE_DRAWIO_XML
        .replace("test", "test_parentless_geometry")
        .replace(r#" vertex="1" parent="1""#, r#" vertex="1""#);
    let Some(result) = render_with_official_drawio_js(&source) else {
        return;
    };
    let svg = expect_drawio_svg(result);

    assert!(svg.contains("<svg"), "{svg}");
}

#[test]
fn renders_compressed_official_drawio_xml_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some(result) = render_with_official_drawio_js(COMPRESSED_DRAWIO_XML) else {
        return;
    };
    let svg = expect_drawio_svg(result);
    assert!(svg.contains("Compressed Box"), "{svg}");
}

#[test]
fn embeds_official_drawio_image_resource_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some(result) = render_with_official_drawio_js(IMAGE_DRAWIO_XML) else {
        return;
    };
    let svg = expect_drawio_svg(result);
    assert!(svg.contains("<image"), "{svg}");
    assert!(svg.contains("data:image/svg+xml;base64,"), "{svg}");
}

#[test]
fn renders_official_mxgraph_stencil_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some(result) = render_with_official_drawio_js(OFFICIAL_STENCIL_DRAWIO_XML) else {
        return;
    };
    let svg = expect_drawio_svg(result);
    assert!(svg.contains("<path"), "{svg}");
    assert!(
        !svg.contains("<rect x=\"40\" y=\"40\" width=\"120\" height=\"120\""),
        "{svg}"
    );
}

#[test]
fn renders_official_aws4_fixture_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    let source =
        include_str!("../../../assets/fixtures/drawio/official/templates/aws/aws_9.drawio");
    let Some(result) = render_with_official_drawio_js(source) else {
        return;
    };
    let svg = expect_drawio_svg(result);
    assert!(svg.contains("AWS Config"), "{svg}");
    assert!(svg.matches("<path").count() > 40, "{svg}");
    assert!(
        !svg.contains("<rect x=\"360.5\" y=\"554.5\" width=\"78\" height=\"78\""),
        "{svg}"
    );
}

#[test]
fn renders_basic_drawio_fixtures_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    if DrawioRenderOps::find_drawio_js().is_none() {
        return;
    }

    for (name, source) in drawio_fixtures::BASIC_DRAWIO_FIXTURES {
        let result = render_with_official_drawio_js(source)
            .unwrap_or_else(|| panic!("Draw.io JavaScript missing while rendering {name}"));
        let svg = expect_drawio_svg(result);
        assert!(svg.contains("<svg"), "{name}: {svg}");
    }
}
