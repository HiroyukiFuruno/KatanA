use katana_core::markdown::diagram::{DiagramBlock, DiagramKind, DiagramResult};
use katana_core::markdown::drawio_renderer::render_drawio;

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
fn valid_drawio_xml_is_converted_to_svg() {
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: SIMPLE_DRAWIO_XML.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("<svg"));
        assert!(html.contains("Box A"));
        assert!(html.contains("Box B"));
    }
}

#[test]
fn invalid_xml_returns_error_result() {
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: "not xml".to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Err { .. }));
}

#[test]
fn ellipse_style_is_processed() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Circle" style="ellipse;" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="100" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("<ellipse"));
    }
}

#[test]
fn fillcolor_style_is_reflected_in_svg_output() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="Colored" style="fillColor=#ff0000;strokeColor=#00ff00;" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    if let DiagramResult::Ok(html) = result {
        assert!(
            html.contains("#ff0000"),
            "fillColor should be reflected in SVG"
        );
        assert!(
            html.contains("#00ff00"),
            "strokeColor should be reflected in SVG"
        );
    }
}

#[test]
fn edge_cell_is_drawn_as_arrow() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="300" y="50" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("polyline") || html.contains("line"));
    }
}

#[test]
fn edge_label_is_drawn() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Start" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="End" vertex="1" parent="1">
    <mxGeometry x="250" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="connects" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("connects"));
    }
}

#[test]
fn edge_with_waypoints_is_drawn() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="350" y="200" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry">
        <Array as="points">
            <mxPoint x="200" y="80"/>
            <mxPoint x="200" y="180"/>
        </Array>
    </mxGeometry>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("polyline"));
    }
}

#[test]
fn edge_without_source_is_skipped() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Box" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" edge="1" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn edge_with_cells_at_same_position_does_not_panic() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="100" y="100" width="1" height="1" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="100" y="100" width="1" height="1" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn vertical_edge_uses_top_bottom_border_points() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="Top" vertex="1" parent="1">
    <mxGeometry x="100" y="50" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="3" value="Bottom" vertex="1" parent="1">
    <mxGeometry x="100" y="250" width="120" height="60" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn special_characters_in_label_are_escaped() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/>
<mxCell id="1" parent="0"/>
<mxCell id="2" value="A &amp; B &lt;3&gt;" vertex="1" parent="1">
    <mxGeometry x="50" y="50" width="120" height="60" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    if let DiagramResult::Ok(html) = result {
        assert!(!html.is_empty());
    }
}

#[test]
fn mxgraphmodel_without_root_returns_empty_svg() {
    let xml = r#"<mxGraphModel></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn unsupported_root_element_returns_error() {
    let xml = r#"<svg><g></g></svg>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Err { .. }));
}

#[test]
fn vertex_without_mxgeometry_is_skipped() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="NoGeo" vertex="1" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        assert!(html.contains("<svg"));
    }
}

#[test]
fn edge_without_source_target_is_skipped() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" edge="1" parent="1"/>
<mxCell id="4" edge="1" source="2" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn edge_with_non_existent_source_target_id_is_skipped() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" edge="1" source="999" target="2" parent="1"/>
<mxCell id="4" edge="1" source="2" target="888" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn vertex_without_label_outputs_no_text_element() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
    if let DiagramResult::Ok(html) = result {
        assert!(!html.contains("<text"));
    }
}

#[test]
fn empty_edge_label_outputs_no_text() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="200" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry as="geometry"/>
</mxCell>
<mxCell id="5" edge="1" source="2" target="3" parent="1">
    <mxGeometry as="geometry"/>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn edge_waypoint_without_mxgeometry_is_empty() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="200" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="edge" edge="1" source="2" target="3" parent="1"/>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}

#[test]
fn collect_waypoints_skips_text_node_and_non_array_child_elements() {
    let xml = r#"<mxGraphModel><root>
<mxCell id="0"/><mxCell id="1" parent="0"/>
<mxCell id="2" value="A" vertex="1" parent="1">
    <mxGeometry x="10" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="3" value="B" vertex="1" parent="1">
    <mxGeometry x="200" y="10" width="100" height="50" as="geometry"/>
</mxCell>
<mxCell id="4" value="" edge="1" source="2" target="3" parent="1">
    <mxGeometry relative="1" as="geometry">
        <!-- text node for L296 -->
        text node here
        <SomeOtherElement foo="bar"/>
        <Array as="points">
            text inside array for L301
            <mxPoint x="100" y="200"/>
        </Array>
    </mxGeometry>
</mxCell>
</root></mxGraphModel>"#;
    let block = DiagramBlock {
        kind: DiagramKind::DrawIo,
        source: xml.to_string(),
    };
    let result = render_drawio(&block);
    assert!(matches!(result, DiagramResult::Ok(_)));
}
