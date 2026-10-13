use std::collections::HashSet;

use super::{CanvasDocument, CanvasError, CanvasLocation};

pub(super) struct CanvasValidator;

impl CanvasValidator {
    pub(super) fn validate_references(document: &CanvasDocument) -> Result<(), CanvasError> {
        let mut node_ids = HashSet::new();
        for (index, node) in document.nodes.iter().enumerate() {
            Self::validate_id(&node.id, CanvasLocation::Node(index), &mut node_ids)?;
        }
        let mut edge_ids = HashSet::new();
        for (index, edge) in document.edges.iter().enumerate() {
            Self::validate_id(&edge.id, CanvasLocation::Edge(index), &mut edge_ids)?;
            if !node_ids.contains(edge.from_node.as_str())
                || !node_ids.contains(edge.to_node.as_str())
            {
                return Err(CanvasError::DanglingEdge { index });
            }
        }
        Ok(())
    }

    fn validate_id<'a>(
        id: &'a str,
        location: CanvasLocation,
        ids: &mut HashSet<&'a str>,
    ) -> Result<(), CanvasError> {
        if !ids.insert(id) {
            return Err(CanvasError::DuplicateId { location });
        }
        Ok(())
    }
}
