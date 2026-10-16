use super::boundary::{BoundaryDecoder as Fields, BoundaryFields, RawDocument};
use super::{
    CanvasDiagnostic, CanvasDocument, CanvasEdge, CanvasError, CanvasLocation, CanvasNode,
};
use super::{EdgeEnd, NodeContent};

pub(super) struct CanvasDecoder;

impl CanvasDecoder {
    pub(super) fn decode_document(raw: RawDocument) -> Result<CanvasDocument, CanvasError> {
        Self::decode_document_with(raw, || Ok::<(), CanvasError>(()))
    }

    pub(super) fn decode_document_with<E: From<CanvasError>>(
        raw: RawDocument,
        checkpoint: impl Fn() -> Result<(), E>,
    ) -> Result<CanvasDocument, E> {
        let mut document = CanvasDocument::default();
        Fields::extra_fields(
            raw.extra,
            CanvasLocation::Document,
            &mut document.diagnostics,
        );
        for (index, fields) in raw.nodes.into_iter().enumerate() {
            checkpoint()?;
            document
                .nodes
                .push(Self::decode_node(fields, index, &mut document.diagnostics)?);
        }
        for (index, fields) in raw.edges.into_iter().enumerate() {
            checkpoint()?;
            document
                .edges
                .push(Self::decode_edge(fields, index, &mut document.diagnostics)?);
        }
        checkpoint()?;
        Ok(document)
    }

    fn decode_node(
        mut fields: BoundaryFields,
        index: usize,
        diagnostics: &mut Vec<CanvasDiagnostic>,
    ) -> Result<CanvasNode, CanvasError> {
        let location = CanvasLocation::Node(index);
        let id = Fields::required(&mut fields, "id", location)?;
        let node_type: String = Fields::required(&mut fields, "type", location)?;
        let geometry = Fields::geometry(&mut fields, index)?;
        let color = Fields::optional(&mut fields, "color", location)?;
        let content = Self::decode_content(&mut fields, node_type, index, diagnostics)?;
        Fields::extra_fields(fields, location, diagnostics);
        Ok(CanvasNode {
            id,
            geometry,
            color,
            content,
        })
    }

    fn decode_content(
        fields: &mut BoundaryFields,
        node_type: String,
        index: usize,
        diagnostics: &mut Vec<CanvasDiagnostic>,
    ) -> Result<NodeContent, CanvasError> {
        let location = CanvasLocation::Node(index);
        match node_type.as_str() {
            "text" => Ok(NodeContent::Text {
                text: Fields::required(fields, "text", location)?,
            }),
            "file" => Self::decode_file(fields, location),
            "link" => Ok(NodeContent::Link {
                url: Fields::required(fields, "url", location)?,
            }),
            "group" => Self::decode_group(fields, location),
            _ => Ok(Self::unsupported_node(node_type, index, diagnostics)),
        }
    }

    fn decode_group(
        fields: &mut BoundaryFields,
        location: CanvasLocation,
    ) -> Result<NodeContent, CanvasError> {
        Ok(NodeContent::Group {
            label: Fields::optional(fields, "label", location)?,
            background: Fields::optional(fields, "background", location)?,
            background_style: Fields::optional(fields, "backgroundStyle", location)?,
        })
    }

    fn unsupported_node(
        node_type: String,
        index: usize,
        diagnostics: &mut Vec<CanvasDiagnostic>,
    ) -> NodeContent {
        diagnostics.push(CanvasDiagnostic::UnsupportedNodeType {
            index,
            node_type: node_type.clone(),
        });
        NodeContent::Unsupported { node_type }
    }

    fn decode_file(
        fields: &mut BoundaryFields,
        location: CanvasLocation,
    ) -> Result<NodeContent, CanvasError> {
        let file = Fields::required(fields, "file", location)?;
        let subpath: Option<String> = Fields::optional(fields, "subpath", location)?;
        if subpath
            .as_ref()
            .is_some_and(|value| !value.starts_with('#'))
        {
            return Err(CanvasError::InvalidField {
                location,
                field: "subpath",
            });
        }
        Ok(NodeContent::File { file, subpath })
    }

    fn decode_edge(
        mut fields: BoundaryFields,
        index: usize,
        diagnostics: &mut Vec<CanvasDiagnostic>,
    ) -> Result<CanvasEdge, CanvasError> {
        let location = CanvasLocation::Edge(index);
        let edge = CanvasEdge {
            id: Fields::required(&mut fields, "id", location)?,
            from_node: Fields::required(&mut fields, "fromNode", location)?,
            from_side: Fields::optional(&mut fields, "fromSide", location)?,
            from_end: Fields::optional(&mut fields, "fromEnd", location)?.unwrap_or(EdgeEnd::None),
            to_node: Fields::required(&mut fields, "toNode", location)?,
            to_side: Fields::optional(&mut fields, "toSide", location)?,
            to_end: Fields::optional(&mut fields, "toEnd", location)?.unwrap_or(EdgeEnd::Arrow),
            color: Fields::optional(&mut fields, "color", location)?,
            label: Fields::optional(&mut fields, "label", location)?,
        };
        Fields::extra_fields(fields, location, diagnostics);
        Ok(edge)
    }
}
