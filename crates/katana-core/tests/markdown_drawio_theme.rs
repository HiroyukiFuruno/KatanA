use katana_core::markdown::color_preset::DiagramColorPreset;
use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::DrawioRenderOps;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn renders_basic_flow_labels_as_svg_text_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    DiagramColorPreset::set_dark_mode(false);
    let Some(result) = render_with_official_drawio_js(include_str!(
        "../../../assets/fixtures/drawio/basic/03-basic-flow.drawio"
    )) else {
        return;
    };
    let svg = expect_drawio_svg(result);

    assert_svg_text(&svg, "Start");
    assert_svg_text(&svg, "Process");
    assert_svg_text(&svg, "OK?");
    assert_svg_text(&svg, "End");
    assert!(!svg.contains("<foreignObject"), "{svg}");
}

#[test]
fn keeps_drawio_dark_theme_without_white_background_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    DiagramColorPreset::set_dark_mode(true);
    let Some(result) = render_with_official_drawio_js(include_str!(
        "../../../assets/fixtures/drawio/basic/03-basic-flow.drawio"
    )) else {
        return;
    };
    let svg = expect_drawio_svg(result);

    assert!(svg.contains("Start"), "{svg}");
    assert!(!svg.contains("data-katana-drawio-background"), "{svg}");
    assert!(!svg.contains("light-dark("), "{svg}");
    assert!(svg.contains(r##"fill="#1f2f1e""##), "{svg}");
    assert!(svg.contains(r##"stroke="#ffffff""##), "{svg}");
    assert!(!svg.contains(r##"fill="#d5e8d4""##), "{svg}");
}

#[test]
fn crops_drawio_page_to_visible_content_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    DiagramColorPreset::set_dark_mode(false);
    let Some(result) = render_with_official_drawio_js(include_str!(
        "../../../assets/fixtures/drawio/basic/02-standalone-mxgraphmodel.drawio"
    )) else {
        return;
    };
    let svg = expect_drawio_svg(result);

    assert!(svg.contains(r#"viewBox="0 0 161 73""#), "{svg}");
    assert!(svg.contains(r#"width="161px""#), "{svg}");
    assert!(svg.contains(r#"height="73px""#), "{svg}");
}

#[test]
fn removes_oversized_drawio_edge_label_backgrounds_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    DiagramColorPreset::set_dark_mode(false);
    let Some(result) = render_with_official_drawio_js(include_str!(
        "../../../assets/fixtures/drawio/basic/05-edge-variants.drawio"
    )) else {
        return;
    };
    let svg = expect_drawio_svg(result);

    assert_svg_text(&svg, "block");
    assert!(
        !svg.contains(r##"<rect fill="#ffffff" stroke="none""##),
        "{svg}"
    );
    assert!(!svg.contains(r#"width="802" height="601""#), "{svg}");
}

#[test]
fn maps_drawio_white_shape_fill_to_dark_theme_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    DiagramColorPreset::set_dark_mode(true);
    let Some(result) = render_with_official_drawio_js(include_str!(
        "../../../assets/fixtures/drawio/basic/09-layers-and-swimlane.drawio"
    )) else {
        return;
    };
    let svg = expect_drawio_svg(result);

    assert!(svg.contains(r##"fill="#121212""##), "{svg}");
    assert!(
        !svg.contains(
            r##"rect x="341" y="355" width="120" height="60" rx="9" ry="9" fill="#ffffff""##
        ),
        "{svg}"
    );
}

#[test]
fn renders_drawio_html_labels_as_svg_text_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    DiagramColorPreset::set_dark_mode(true);
    let Some(result) = render_with_official_drawio_js(include_str!(
        "../../../assets/fixtures/drawio/basic/07-html-labels-and-entities.drawio"
    )) else {
        return;
    };
    let svg = expect_drawio_svg(result);

    assert!(
        svg.contains(">Line</tspan>") || svg.contains(">Line</text>"),
        "{svg}"
    );
    assert!(
        svg.contains(">Break</tspan>") || svg.contains(">Break</text>"),
        "{svg}"
    );
    assert_svg_text(&svg, "Bold");
    assert_svg_text(&svg, "Italic");
    assert!(svg.contains(">A &amp; B &lt; C</text>"), "{svg}");
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

fn assert_svg_text(svg: &str, expected: &str) {
    assert!(
        svg.contains(&format!(">{expected}</text>"))
            || svg.contains(&format!(">{expected}</tspan>")),
        "{svg}"
    );
}
