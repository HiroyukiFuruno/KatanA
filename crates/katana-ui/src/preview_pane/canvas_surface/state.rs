use std::sync::Arc;

use katana_core::canvas::{CanvasDocument, CanvasError};

#[derive(Debug)]
pub(crate) enum CanvasLoadError {
    Request(crate::preview_pane::canvas_request::CanvasRequestError),
    InvalidCanvas(CanvasError),
    UnrepresentableGeometry { index: usize },
}

pub(super) enum CanvasLoadState {
    Loading,
    Ready(Arc<CanvasDocument>),
    Failed(CanvasLoadError),
}
