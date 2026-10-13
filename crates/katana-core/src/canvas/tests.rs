use super::*;

const PREVIEW_FIXTURE: &str =
    include_str!("../../../../assets/fixtures/katana/read-only-preview.canvas");

#[test]
fn fixture_preserves_node_order_and_content_types() {
    let document = CanvasDocument::parse(PREVIEW_FIXTURE).expect("valid fixture");
    let kinds: Vec<_> = document
        .nodes
        .iter()
        .map(|node| match &node.content {
            NodeContent::Text { .. } => "text",
            NodeContent::File { .. } => "file",
            NodeContent::Link { .. } => "link",
            NodeContent::Group { .. } => "group",
            NodeContent::Unsupported { .. } => "unsupported",
        })
        .collect();
    assert_eq!(
        kinds,
        ["group", "text", "file", "link", "unsupported", "text"]
    );
}

#[test]
fn fixture_keeps_markdown_text_and_negative_positions() {
    let document = CanvasDocument::parse(PREVIEW_FIXTURE).expect("valid fixture");
    assert!(
        matches!(&document.nodes[1].content, NodeContent::Text { text }
        if text.contains("**Markdown**"))
    );
    assert_eq!(document.nodes[0].geometry.x, -40.0);
}

#[test]
fn fixture_keeps_file_reference_without_opening_file() {
    let document = CanvasDocument::parse(PREVIEW_FIXTURE).expect("valid fixture");
    assert_eq!(
        document.nodes[2].content,
        NodeContent::File {
            file: "notes/example.md".into(),
            subpath: Some("#Example".into()),
        }
    );
    assert_eq!(document.nodes[2].content.file_label(), Some("example.md"));
}

#[test]
fn fixture_retains_link_without_fetching_url() {
    let document = CanvasDocument::parse(PREVIEW_FIXTURE).expect("valid fixture");
    assert_eq!(
        document.nodes[3].content,
        NodeContent::Link {
            url: "https://jsoncanvas.org/spec/1.0/".into(),
        }
    );
}

#[test]
fn fixture_reports_unknown_type_and_extra_fields() {
    let document = CanvasDocument::parse(PREVIEW_FIXTURE).expect("valid fixture");
    assert_eq!(
        document.diagnostics,
        vec![
            CanvasDiagnostic::UnsupportedNodeType {
                index: 4,
                node_type: "future-card".into()
            },
            CanvasDiagnostic::UnsupportedFields {
                location: CanvasLocation::Node(4),
                fields: vec!["futureContent".into()],
            },
            CanvasDiagnostic::UnsupportedFields {
                location: CanvasLocation::Node(5),
                fields: vec!["customDecoration".into()],
            },
        ]
    );
}

#[test]
fn fixture_keeps_edge_style_and_spec_endpoint_defaults() {
    let document = CanvasDocument::parse(PREVIEW_FIXTURE).expect("valid fixture");
    let edge = &document.edges[0];
    assert_eq!(
        (edge.from_side, edge.to_side),
        (Some(EdgeSide::Right), Some(EdgeSide::Left))
    );
    assert_eq!(edge.color, Some(CanvasColor::Preset(2)));
    assert_eq!(edge.label.as_deref(), Some("references"));
    assert_eq!(
        (document.edges[1].from_end, document.edges[1].to_end),
        (EdgeEnd::None, EdgeEnd::Arrow)
    );
}

#[test]
fn omitted_arrays_and_empty_arrays_produce_empty_documents() {
    for source in ["{}", r#"{"nodes":[],"edges":[]}"#, r#"{"edges":[]}"#] {
        assert_eq!(
            CanvasDocument::parse(source).expect("empty document"),
            CanvasDocument::default()
        );
    }
}

#[test]
fn document_and_edge_extra_fields_are_reported() {
    let source = r#"{"nodes":[],"edges":[],"metadata":{"version":2}}"#;
    let document = CanvasDocument::parse(source).expect("valid extension");
    assert_eq!(
        document.diagnostics,
        vec![CanvasDiagnostic::UnsupportedFields {
            location: CanvasLocation::Document,
            fields: vec!["metadata".into()],
        }]
    );
}

#[test]
fn group_background_attributes_remain_typed() {
    let source = r#"{"nodes":[{"id":"g","type":"group","x":0,"y":0,"width":20,"height":10,"background":"missing.png","backgroundStyle":"repeat","label":"G"}]}"#;
    let document = CanvasDocument::parse(source).expect("valid group");
    assert_eq!(
        document.nodes[0].content,
        NodeContent::Group {
            label: Some("G".into()),
            background: Some("missing.png".into()),
            background_style: Some(BackgroundStyle::Repeat),
        }
    );
}

#[test]
fn hex_color_is_preserved_and_invalid_color_is_rejected() {
    assert_eq!(
        CanvasColor::try_from("#Ab00FF".to_owned()),
        Ok(CanvasColor::Hex("#Ab00FF".into()))
    );
    assert!(CanvasColor::try_from("7".to_owned()).is_err());
}
