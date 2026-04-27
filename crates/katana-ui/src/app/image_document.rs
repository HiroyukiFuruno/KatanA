use crate::app_state::AppState;
use katana_core::document::{Document, DocumentError};
use katana_platform::FilesystemService;
use std::path::Path;

pub(crate) struct ImageDocumentOps;

impl ImageDocumentOps {
    pub(crate) fn is_image_path(path: &Path) -> bool {
        path.exists() && katana_core::workspace::TreeEntry::path_is_image(path)
    }

    pub(crate) fn load_or_create(
        fs: &FilesystemService,
        path: &Path,
    ) -> Result<Document, DocumentError> {
        if Self::is_image_path(path) {
            return Ok(Self::new_document(path));
        }
        fs.load_document(path)
    }

    pub(crate) fn replacement_for_unloaded(
        fs: &FilesystemService,
        state: &AppState,
        idx: usize,
        path: &Path,
    ) -> Option<Document> {
        let doc = state.document.open_documents.get(idx)?;
        if doc.is_loaded || path.to_string_lossy().starts_with("Katana://") {
            return None;
        }
        let mut loaded_doc = match Self::load_or_create(fs, path) {
            Ok(doc) => doc,
            Err(err) => {
                tracing::warn!("Lazy document load failed: {err}");
                return None;
            }
        };
        loaded_doc.is_pinned = doc.is_pinned;
        Some(loaded_doc)
    }

    pub(crate) fn refresh_payload(
        state: &AppState,
        idx: usize,
        path: &Path,
    ) -> Option<(String, usize)> {
        if !katana_core::workspace::TreeEntry::path_is_image(path) {
            return None;
        }
        let src = state.document.open_documents.get(idx)?.buffer.clone();
        let concurrency = state
            .config
            .settings
            .settings()
            .performance
            .resolved_diagram_concurrency();
        Some((src, concurrency))
    }

    pub(crate) fn buffer_for_path(path: &Path) -> String {
        format!("![](file://{})", path.display())
    }

    pub(crate) fn refresh_reference_path(
        doc: &mut Document,
        source_path: &Path,
        target_path: &Path,
    ) {
        if doc.path != source_path {
            return;
        }
        doc.path = target_path.to_path_buf();
        if doc.is_reference && katana_core::workspace::TreeEntry::path_is_image(target_path) {
            doc.buffer = Self::buffer_for_path(target_path);
        }
    }

    fn new_document(path: &Path) -> Document {
        let mut doc = Document::new(path.to_path_buf(), Self::buffer_for_path(path));
        doc.is_reference = true;
        doc
    }
}
