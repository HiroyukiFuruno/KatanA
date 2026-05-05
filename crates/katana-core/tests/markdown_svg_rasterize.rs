use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::svg_rasterize::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

const MINIMAL_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/></svg>"#;
const NBSP_SVG: &str =
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="40"><text x="10" y="24">A&nbsp;B</text></svg>"#;
const BASIC_DRAWIO_XML: &str = r#"<mxfile><diagram name="test"><mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Box A" vertex="1" parent="1">
    <mxGeometry x="80" y="80" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="Box B" vertex="1" parent="1">
    <mxGeometry x="280" y="80" width="120" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel></diagram></mxfile>"#;

#[test]
fn valid_svg_is_rasterized() {
    let result = SvgRasterizeOps::rasterize_svg(MINIMAL_SVG, 1.0).expect("rasterize failed");
    assert_eq!(result.width, 100);
    assert_eq!(result.height, 100);
    assert_eq!(result.rgba.len(), 100 * 100 * 4);
}

#[test]
fn scale_is_applied() {
    let result = SvgRasterizeOps::rasterize_svg(MINIMAL_SVG, 2.0).expect("rasterize failed");
    assert_eq!(result.width, 200);
    assert_eq!(result.height, 200);
}

#[test]
fn scaled_raster_keeps_original_display_size() {
    let result = SvgRasterizeOps::rasterize_svg(MINIMAL_SVG, 2.0).expect("rasterize failed");
    assert_eq!(result.width, 200);
    assert_eq!(result.height, 200);
    assert_eq!(result.display_width, 100.0);
    assert_eq!(result.display_height, 100.0);
}

#[test]
fn oversized_svg_is_scaled_to_gpu_safe_size() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50000" height="1000"><rect width="50000" height="1000" fill="red"/></svg>"#;
    let result = SvgRasterizeOps::rasterize_svg(svg, 1.5).expect("rasterize failed");
    assert_eq!(result.width, 8192);
    assert!(result.height <= 8192);
}

#[test]
fn rendered_drawio_svg_is_rasterized() {
    let Some(svg) = render_drawio_svg(BASIC_DRAWIO_XML) else {
        return;
    };
    let result = SvgRasterizeOps::rasterize_svg(&svg, 1.0).expect("rasterize failed");
    assert!(result.width > 0);
    assert!(result.height > 0);
}

#[test]
fn html_nbsp_entity_is_rasterized_as_xml_entity() {
    let result = SvgRasterizeOps::rasterize_svg(NBSP_SVG, 1.0).expect("rasterize failed");
    assert_eq!(result.width, 100);
    assert_eq!(result.height, 40);
}

#[test]
fn invalid_svg_returns_error() {
    let result = SvgRasterizeOps::rasterize_svg("not valid svg", 1.0);
    assert!(matches!(result, Err(SvgRasterizeError::ParseFailed(_))));
}

fn render_drawio_svg(source: &str) -> Option<String> {
    let _guard = ENV_LOCK.lock().unwrap();
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: source.to_string(),
    };

    match block.render() {
        DiagramResult::Ok(svg) => Some(svg),
        DiagramResult::NotInstalled { .. } => None,
        other => panic!("Draw.io SVG rendering failed: {other:?}"),
    }
}
