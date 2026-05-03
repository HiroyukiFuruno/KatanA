use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::DrawioRenderOps;
use katana_core::markdown::svg_rasterize::*;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

const MINIMAL_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="red"/></svg>"#;

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
fn drawio_official_light_dark_svg_is_rasterized() {
    let Some(svg) = render_official_drawio_svg(include_str!(
        "../../../assets/fixtures/drawio/official/templates/aws/aws_10.drawio"
    )) else {
        return;
    };
    let result = SvgRasterizeOps::rasterize_svg(&svg, 1.0).expect("rasterize failed");
    assert_eq!(result.width, 2054);
    assert_eq!(result.height, 1091);
}

#[test]
fn drawio_official_svg_with_nbsp_entity_is_rasterized() {
    let Some(svg) = render_official_drawio_svg(include_str!(
        "../../../assets/fixtures/drawio/official/templates/azure/azure_2.drawio"
    )) else {
        return;
    };
    let result = SvgRasterizeOps::rasterize_svg(&svg, 1.0).expect("rasterize failed");
    assert_eq!(result.width, 1065);
    assert_eq!(result.height, 740);
}

#[test]
fn invalid_svg_returns_error() {
    let result = SvgRasterizeOps::rasterize_svg("not valid svg", 1.0);
    assert!(matches!(result, Err(SvgRasterizeError::ParseFailed(_))));
}

fn render_official_drawio_svg(source: &str) -> Option<String> {
    let _guard = ENV_LOCK.lock().unwrap();
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: source.to_string(),
    };

    match DrawioRenderOps::render_drawio(&block) {
        DiagramResult::Ok(svg) => Some(svg),
        DiagramResult::NotInstalled { .. } => None,
        other => panic!("Draw.io SVG rendering failed: {other:?}"),
    }
}
