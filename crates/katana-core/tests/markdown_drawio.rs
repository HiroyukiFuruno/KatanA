use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::DrawioRenderOps;
use std::thread;

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
fn valid_drawio_xml_is_converted_to_png() {
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        // Make the source slightly unique so it doesn't hit cache from other tests
        source: SIMPLE_DRAWIO_XML.replace("test", "test1"),
    };
    let result = DrawioRenderOps::render_drawio(&block);
    match result {
        DiagramResult::OkPng(_) => {}
        DiagramResult::Err { error, .. }
            if error.contains("Could not auto detect a chrome executable")
                || error.contains("Cannot find browser")
                || error.contains("Failed to launch browser") =>
        { /* WHY: Chrome is missing in the CI runner environment */ }
        other => panic!("Expected OkPng or Chrome error, got {:?}", other),
    }
}

#[test]
fn invalid_xml_returns_error_result() {
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: "not xml".to_string(),
    };
    let result = DrawioRenderOps::render_drawio(&block);
    assert!(
        matches!(result, DiagramResult::Err { .. }),
        "Expected Err, got {:?}",
        result
    );
}

#[test]
fn concurrent_drawio_rendering_succeeds() {
    let mut handles = vec![];
    for i in 0..3 {
        handles.push(thread::spawn(move || {
            let block = DiagramBlock {
                kind: DiagramKind::DrawIo,
                source: SIMPLE_DRAWIO_XML.replace("test", &format!("test_concurrent_{}", i)),
            };
            let result = DrawioRenderOps::render_drawio(&block);
            match result {
                DiagramResult::OkPng(_) => {}
                DiagramResult::Err { error, .. }
                    if error.contains("Could not auto detect a chrome executable")
                        || error.contains("Cannot find browser")
                        || error.contains("Failed to launch browser") =>
                { /* WHY: Chrome is missing in the CI runner environment */ }
                other => panic!("Expected OkPng or Chrome error, got {:?}", other),
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
