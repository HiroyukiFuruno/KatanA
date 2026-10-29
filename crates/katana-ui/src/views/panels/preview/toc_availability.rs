use crate::app_state::{AppAction, AppState, DocumentOperationRuntimeSnapshot, ViewMode};
use crate::shell::KatanaApp;

pub(crate) use super::document_operation_settings::{
    DocumentOperation, DocumentOperationSettings, PreviewMenu,
};

pub(crate) struct PreviewMenuAvailability;

impl PreviewMenuAvailability {
    pub(crate) fn settings_for_app(app: &mut KatanaApp) -> DocumentOperationSettings {
        app.refresh_document_operation_runtime();
        Self::settings_for_state(&app.state)
    }

    pub(crate) fn settings_for_state(state: &AppState) -> DocumentOperationSettings {
        let Some(document) = state.active_document() else {
            return DocumentOperationSettings::evaluate(None, Some(false));
        };
        let runtime = &state.document_operation_runtime;
        let current = runtime.active_document_path.as_deref() == Some(document.path.as_path())
            && runtime.active_document_revision == Some(document.revision)
            && runtime.active_doc_idx == state.document.active_doc_idx
            && runtime.preview_generation.is_some();
        DocumentOperationSettings::evaluate(
            Some(document.path.as_path()),
            Some(current && runtime.pptx_slideshow_ready),
        )
    }

    pub(crate) fn for_app(app: &mut KatanaApp, menu: PreviewMenu) -> bool {
        Self::settings_for_app(app).allows(DocumentOperation::PreviewMenu(menu))
    }

    pub(crate) fn for_state(state: &AppState, menu: PreviewMenu) -> bool {
        Self::settings_for_state(state).allows(DocumentOperation::PreviewMenu(menu))
    }

    pub(crate) fn for_view_mode(state: &AppState, mode: ViewMode) -> bool {
        Self::settings_for_state(state).allows(DocumentOperation::ViewMode(mode))
    }

    pub(crate) fn for_action(state: &AppState, action: &AppAction) -> Option<bool> {
        Self::settings_for_state(state).allows_action(action, state.active_view_mode())
    }

    #[cfg(test)]
    pub(crate) fn for_path(path: Option<&std::path::Path>, menu: PreviewMenu) -> bool {
        DocumentOperationSettings::evaluate(path, None).allows(DocumentOperation::PreviewMenu(menu))
    }
}

fn same_runtime_identity(
    left: &DocumentOperationRuntimeSnapshot,
    right: &DocumentOperationRuntimeSnapshot,
) -> bool {
    left.active_document_path == right.active_document_path
        && left.active_document_revision == right.active_document_revision
        && left.active_doc_idx == right.active_doc_idx
        && left.preview_generation == right.preview_generation
        && left.pptx_slideshow_ready == right.pptx_slideshow_ready
}

impl KatanaApp {
    pub(crate) fn refresh_document_operation_runtime(&mut self) {
        let mut next = self.document_operation_runtime_snapshot();
        let previous = &self.state.document_operation_runtime;
        next.epoch = if same_runtime_identity(previous, &next) {
            previous.epoch
        } else {
            previous.epoch.saturating_add(1)
        };
        self.state.document_operation_runtime = next;
    }

    fn document_operation_runtime_snapshot(&self) -> DocumentOperationRuntimeSnapshot {
        let Some((active_doc_idx, path, revision)) = self.state.active_document().map(|document| {
            (
                self.state.document.active_doc_idx,
                document.path.clone(),
                document.revision,
            )
        }) else {
            return DocumentOperationRuntimeSnapshot::default();
        };
        let preview = self
            .tab_previews
            .iter()
            .find(|preview| preview.path == path);
        DocumentOperationRuntimeSnapshot {
            active_document_path: Some(path),
            active_document_revision: Some(revision),
            active_doc_idx,
            preview_generation: preview.map(|preview| preview.pane.session_generation),
            pptx_slideshow_ready: preview
                .is_some_and(|preview| preview.pane.pptx_slideshow_available()),
            epoch: self.state.document_operation_runtime.epoch,
        }
    }
}

#[cfg(test)]
#[path = "toc_availability_tests.rs"]
mod tests;
