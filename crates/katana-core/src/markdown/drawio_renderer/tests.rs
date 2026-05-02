use super::*;
use crate::markdown::color_preset::DiagramColorPreset;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

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
fn test_invalid_drawio_xml_returns_error() {
    use crate::markdown::{DiagramBlock, DiagramKind};
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let drawio_js = dir.path().join("drawio.min.js");
    std::fs::write(&drawio_js, "window.GraphViewer = {};").unwrap();
    unsafe { std::env::set_var("DRAWIO_JS", &drawio_js) };
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: "<mxGraphModel>".to_string(),
    };

    let result = DrawioRendererOps::render_drawio(&block);
    unsafe { std::env::remove_var("DRAWIO_JS") };
    assert!(matches!(result, DiagramResult::Err { .. }));
}

#[test]
fn official_object_cells_render_with_raw_js_runtime() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some(drawio_js) = DrawioRendererOps::find_drawio_js() else {
        return;
    };
    let source =
        include_str!("../../../../../assets/fixtures/drawio/basic/10-userobject-metadata.drawio");

    let result = super::js_runtime::DrawioJsRuntimeOps::render(
        source,
        &drawio_js,
        DiagramColorPreset::current(),
    );

    assert!(
        result.as_ref().is_ok_and(|svg| svg.contains("<svg")),
        "{result:?}"
    );
}

#[test]
fn official_atlas_image_fixture_reports_raw_js_result() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some(drawio_js) = DrawioRendererOps::find_drawio_js() else {
        return;
    };
    let source = include_str!(
        "../../../../../assets/fixtures/drawio/official/blog/azure-architecture-example.drawio"
    );

    let result = super::js_runtime::DrawioJsRuntimeOps::render(
        source,
        &drawio_js,
        DiagramColorPreset::current(),
    );

    assert!(
        result.as_ref().is_ok_and(|svg| svg.contains("<svg")),
        "{result:?}"
    );
}

#[test]
fn official_drawio_fixture_sequence_reports_raw_js_result() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some(drawio_js) = DrawioRendererOps::find_drawio_js() else {
        return;
    };
    for path in official_drawio_fixture_paths() {
        let source = std::fs::read_to_string(&path).unwrap();
        let result = super::js_runtime::DrawioJsRuntimeOps::render(
            &source,
            &drawio_js,
            DiagramColorPreset::current(),
        );

        assert!(
            result.as_ref().is_ok_and(|svg| svg.contains("<svg")),
            "{}: {result:?}",
            path.display()
        );
    }
}

fn official_drawio_fixture_paths() -> Vec<std::path::PathBuf> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/drawio/official");
    ["blog"]
        .into_iter()
        .flat_map(|directory| official_drawio_files(root.join(directory)))
        .collect()
}

fn official_drawio_files(root: std::path::PathBuf) -> Vec<std::path::PathBuf> {
    let mut paths = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "drawio")
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
}
