use crate::app::PreviewOps;
use crate::shell::KatanaApp;
use katana_core::workspace::TreeEntry;
use std::path::Path;

pub(super) struct DocumentPreviewPolicy;

impl DocumentPreviewPolicy {
    pub(super) fn select(app: &mut KatanaApp, path: &Path, activate: bool) {
        if !activate {
            return;
        }
        if TreeEntry::path_is_document(path) || TreeEntry::path_is_canvas(path) {
            app.state
                .set_active_view_mode(crate::state::document::ViewMode::PreviewOnly);
        }
    }

    pub(super) fn refresh_edited_markdown(app: &mut KatanaApp, path: &Path, content: &str) {
        if TreeEntry::path_is_html(path) {
            return;
        }
        let concurrency = app
            .state
            .config
            .settings
            .settings()
            .performance
            .resolved_diagram_concurrency();
        app.full_refresh_preview(path, content, true, concurrency);
    }
}
