use super::{CanvasError, CanvasLocation, GeometryField, NodeContent};

#[test]
fn geometry_error_formats_only_node_index_and_field() {
    let error = CanvasError::InvalidGeometry {
        index: 3,
        field: GeometryField::Width,
    };
    assert_eq!(error.to_string(), "Invalid Canvas node 3 geometry: Width");
}

#[test]
fn field_error_formats_only_location_and_field() {
    let error = CanvasError::InvalidField {
        location: CanvasLocation::Edge(2),
        field: "id",
    };
    assert_eq!(error.to_string(), "Invalid Canvas field id at Edge(2)");
}

#[test]
fn duplicate_id_error_formats_only_location() {
    let error = CanvasError::DuplicateId {
        location: CanvasLocation::Node(4),
    };
    assert_eq!(error.to_string(), "Duplicate Canvas ID at Node(4)");
}

#[test]
fn dangling_edge_error_formats_only_index() {
    let error = CanvasError::DanglingEdge { index: 5 };
    assert_eq!(error.to_string(), "Canvas edge 5 references a missing node");
}

#[test]
fn non_file_canvas_content_has_no_file_label() {
    let cases = [
        NodeContent::Text {
            text: "text/path".into(),
        },
        NodeContent::Link {
            url: "https://example.invalid/file".into(),
        },
        NodeContent::Group {
            label: Some("group/path".into()),
            background: None,
            background_style: None,
        },
        NodeContent::Unsupported {
            node_type: "extension/path".into(),
        },
    ];
    for content in cases {
        assert_eq!(content.file_label(), None);
    }
}
