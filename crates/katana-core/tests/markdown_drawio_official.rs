use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::DrawioRenderOps;
use katana_core::markdown::fence::MarkdownFenceOps;
use std::path::PathBuf;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn renders_official_drawio_fixture_set_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    if DrawioRenderOps::find_drawio_js().is_none() {
        return;
    }

    for source_path in official_drawio_fixture_paths() {
        let source = std::fs::read_to_string(&source_path)
            .unwrap_or_else(|error| panic!("Failed to read {}: {error}", source_path.display()));
        let result = render_with_official_drawio_js(&source).unwrap_or_else(|| {
            panic!("Draw.io JavaScript missing while rendering {source_path:?}")
        });
        let svg = expect_drawio_svg(result);
        assert!(svg.contains("<svg"), "{}: {svg}", source_path.display());
    }
}

#[test]
fn renders_feature_drawio_mxfile_with_uncompressed_model_when_official_js_is_available() {
    let _guard = ENV_LOCK.lock().unwrap();
    let Some((block, _)) = MarkdownFenceOps::extract_fence_block(include_str!(
        "../../../assets/feature/katana-architecture.md"
    )) else {
        panic!("feature architecture fixture must contain a drawio fence");
    };
    let Some(result) = render_with_official_drawio_js(&block.content) else {
        return;
    };

    let svg = expect_drawio_svg(result);

    assert!(svg.contains("Katana Core"), "{svg}");
    assert!(svg.contains("Preview Pane"), "{svg}");
    assert!(svg.contains("<text"), "{svg}");
    assert!(!svg.contains("<foreignObject"), "{svg}");
    assert!(!svg.contains("light-dark("), "{svg}");
    assert!(!svg.contains("data-katana-drawio-background"), "{svg}");
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

fn official_drawio_fixture_paths() -> Vec<PathBuf> {
    ["official/blog"]
        .into_iter()
        .flat_map(drawio_fixture_directory_paths)
        .collect()
}

fn drawio_fixture_directory_paths(directory: &str) -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/fixtures/drawio")
        .join(directory);
    let mut paths = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|it| it == "drawio"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}
