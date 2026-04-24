use super::*;

#[test]
fn test_default_install_path() {
    let path = DrawioRendererOps::default_install_path();
    assert!(path.is_some());
    let path = path.unwrap();
    assert!(path.to_string_lossy().contains(".local"));
    assert!(path.to_string_lossy().contains("katana"));
    assert!(path.to_string_lossy().contains("drawio.min.js"));
}

#[test]
fn test_find_drawio_js() {
    /* WHY: Just ensuring it doesn't panic. It might be None on CI. */
    let _ = DrawioRendererOps::find_drawio_js();
}

#[test]
fn test_render_drawio_file_read_error() {
    use crate::markdown::{DiagramBlock, DiagramKind};
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: "<mxGraphModel><root><mxCell id=\"0\"/><mxCell id=\"1\" parent=\"0\"/></root></mxGraphModel>".to_string(),
    };

    /* WHY: Test should not panic. It will either return OkPng or Err, depending on chrome availability. */
    let _ = DrawioRendererOps::render_drawio(&block);
}
