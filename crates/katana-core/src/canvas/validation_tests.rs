use serde_json::json;

use super::*;

fn text_node() -> serde_json::Value {
    json!({"id":"a","type":"text","x":0,"y":0,"width":100,"height":80,"text":"A"})
}

#[test]
fn malformed_json_returns_bounded_error_without_input_echo() {
    let error = CanvasDocument::parse("{broken").expect_err("malformed JSON");
    assert!(matches!(error, CanvasError::InvalidJson { .. }));
    assert!(!error.to_string().contains("broken"));
}

#[test]
fn missing_geometry_returns_field_specific_error() {
    let mut node = text_node();
    node.as_object_mut().expect("object").remove("x");
    assert_eq!(
        CanvasDocument::parse(&json!({"nodes":[node]}).to_string()),
        Err(CanvasError::InvalidGeometry {
            index: 0,
            field: GeometryField::X
        })
    );
}

#[test]
fn zero_negative_fractional_string_and_extreme_sizes_are_rejected() {
    for width in [
        json!(0),
        json!(-1),
        json!(0.5),
        json!("10"),
        json!(1e300),
        json!(null),
    ] {
        let mut node = text_node();
        node["width"] = width;
        assert_eq!(
            CanvasDocument::parse(&json!({"nodes":[node]}).to_string()),
            Err(CanvasError::InvalidGeometry {
                index: 0,
                field: GeometryField::Width
            })
        );
    }
}

#[test]
fn duplicate_node_ids_are_rejected() {
    assert_eq!(
        CanvasDocument::parse(&json!({"nodes":[text_node(),text_node()]}).to_string()),
        Err(CanvasError::DuplicateId {
            location: CanvasLocation::Node(1)
        })
    );
}

#[test]
fn duplicate_edge_ids_are_rejected() {
    let edge = json!({"id":"e","fromNode":"a","toNode":"a"});
    let source = json!({"nodes":[text_node()],"edges":[edge.clone(),edge]});
    assert_eq!(
        CanvasDocument::parse(&source.to_string()),
        Err(CanvasError::DuplicateId {
            location: CanvasLocation::Edge(1)
        })
    );
}

#[test]
fn dangling_edge_in_either_direction_is_rejected() {
    for (from, to) in [("missing", "a"), ("a", "missing")] {
        let source =
            json!({"nodes":[text_node()],"edges":[{"id":"e","fromNode":from,"toNode":to}]});
        assert_eq!(
            CanvasDocument::parse(&source.to_string()),
            Err(CanvasError::DanglingEdge { index: 0 })
        );
    }
}

#[test]
fn node_and_edge_ids_have_independent_namespaces() {
    let source = json!({"nodes":[text_node()],"edges":[{"id":"a","fromNode":"a","toNode":"a"}]});
    let document = CanvasDocument::parse(&source.to_string()).expect("separate ID namespaces");
    assert_eq!(document.edges[0].from_node, document.nodes[0].id);
}

#[test]
fn unique_empty_node_and_edge_ids_are_accepted_and_referenced() {
    let mut node = text_node();
    node["id"] = json!("");
    let source = json!({"nodes":[node],"edges":[{"id":"","fromNode":"","toNode":""}]});
    let document = CanvasDocument::parse(&source.to_string()).expect("unique empty IDs");
    assert_eq!(document.nodes[0].id, "");
    assert_eq!(document.edges[0].from_node, "");
}

#[test]
fn raw_fractional_coordinate_cannot_be_rounded_to_an_integer() {
    let source = r#"{"nodes":[{"id":"a","type":"text","text":"test","x":4503599627370496.5,"y":0,"width":10,"height":10}]}"#;
    assert_eq!(
        CanvasDocument::parse(source),
        Err(CanvasError::InvalidGeometry {
            index: 0,
            field: GeometryField::X
        })
    );
}

#[test]
fn invalid_required_content_and_optional_style_are_rejected() {
    for (field, value) in [("text", json!(42)), ("color", json!("bad"))] {
        let mut node = text_node();
        node[field] = value;
        assert_eq!(
            CanvasDocument::parse(&json!({"nodes":[node]}).to_string()),
            Err(CanvasError::InvalidField {
                location: CanvasLocation::Node(0),
                field
            })
        );
    }
}

#[test]
fn invalid_edge_sides_are_rejected() {
    let source = json!({"nodes":[text_node()],"edges":[{"id":"e","fromNode":"a","toNode":"a","fromSide":"middle"}]});
    assert_eq!(
        CanvasDocument::parse(&source.to_string()),
        Err(CanvasError::InvalidField {
            location: CanvasLocation::Edge(0),
            field: "fromSide"
        })
    );
}

#[test]
fn extra_edge_fields_are_reported() {
    let source =
        json!({"nodes":[text_node()],"edges":[{"id":"e","fromNode":"a","toNode":"a","weight":3}]});
    let document = CanvasDocument::parse(&source.to_string()).expect("valid edge");
    assert_eq!(
        document.diagnostics,
        vec![CanvasDiagnostic::UnsupportedFields {
            location: CanvasLocation::Edge(0),
            fields: vec!["weight".into()],
        }]
    );
}

#[test]
fn file_subpath_requires_spec_prefix() {
    let source = r#"{"nodes":[{"id":"f","type":"file","x":0,"y":0,"width":10,"height":10,"file":"../missing.md","subpath":"heading"}]}"#;
    assert_eq!(
        CanvasDocument::parse(source),
        Err(CanvasError::InvalidField {
            location: CanvasLocation::Node(0),
            field: "subpath",
        })
    );
}

#[test]
fn explicit_null_arrays_are_rejected() {
    assert!(matches!(
        CanvasDocument::parse(r#"{"nodes":null}"#),
        Err(CanvasError::InvalidJson { .. })
    ));
}
