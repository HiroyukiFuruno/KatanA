use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::DrawioRenderOps;
use std::sync::Mutex;
use std::thread;

static ENV_LOCK: Mutex<()> = Mutex::new(());

const SIMPLE_DRAWIO_XML: &str = r#"<mxfile><diagram name="test"><mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Box A" style="rounded=1;fillColor=#fff2cc;strokeColor=#d6b656;" vertex="1" parent="1">
    <mxGeometry x="80" y="80" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="Box B" vertex="1" parent="1">
    <mxGeometry x="280" y="80" width="120" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel></diagram></mxfile>"#;

#[test]
fn returns_not_installed_when_drawio_js_is_missing() {
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
            kind,
            download_url,
            install_path,
        } => {
            assert_eq!(kind, "Draw.io");
            assert!(download_url.contains("viewer-static.min.js"));
            assert_eq!(install_path, missing_path);
        }
        other => panic!("Expected NotInstalled when Draw.io JS is missing, got {other:?}"),
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
fn concurrent_drawio_rendering_succeeds() {
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
