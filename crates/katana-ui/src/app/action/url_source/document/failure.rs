use super::failed_document_identity;
use crate::app::preview::PreviewOps;
use crate::app_state::StatusType;
use crate::shell::KatanaApp;
use crate::state::{HtmlSourceError, PendingUrlRequest};
use katana_core::document_source::BinaryDocumentFormat;
use std::path::{Path, PathBuf};

impl KatanaApp {
    pub(in crate::app::action) fn retry_failed_url_source(
        &mut self,
        ctx: &egui::Context,
        path: &Path,
        is_manual: bool,
    ) -> bool {
        let source = self
            .tab_previews
            .iter()
            .find(|preview| preview.path == path)
            .and_then(|preview| preview.pane.document_failure.as_ref())
            .filter(|failure| failure.operation == "fetch")
            .and_then(|failure| {
                crate::app::url_source::ValidatedHttpUrl::parse(&failure.document).ok()
            });
        let Some(source) = source else {
            return false;
        };
        /* WHY: 初回失敗や失敗navigationは成功済みURL登録より元要求を優先して再取得する。 */
        self.fetch_html_url_for_refresh(ctx, source, Some(path.to_path_buf()), is_manual);
        true
    }

    pub(in crate::app::action) fn fail_html_source(&mut self, error: HtmlSourceError) {
        let source_url = Self::url_failure_source(&error, &self.state.url_tab.input).to_owned();
        self.record_url_source_failure(error, &source_url, None);
    }

    pub(in crate::app::action::url_source) fn fail_requested_url_source(
        &mut self,
        error: HtmlSourceError,
        request: PendingUrlRequest,
    ) {
        /* WHY: redirect先のエラーURLではなく元要求を診断対象にし、最終URLはcauseに残す。 */
        self.record_url_source_failure(
            error,
            &request.source_url,
            request.target_document.as_deref(),
        );
    }

    fn url_failure_source<'a>(error: &'a HtmlSourceError, requested_url: &'a str) -> &'a str {
        match error {
            HtmlSourceError::HttpStatus { url, .. }
            | HtmlSourceError::DocumentSource { url, .. }
            | HtmlSourceError::LocalFile { url, .. }
            | HtmlSourceError::Timeout { url, .. } => url,
            _ => requested_url,
        }
    }

    fn record_url_source_failure(
        &mut self,
        error: HtmlSourceError,
        source_url: &str,
        target: Option<&Path>,
    ) {
        let message = error.to_string();
        self.show_requested_url_failure(source_url, target, &message);
        self.state.layout.status_message = Some((message, StatusType::Error));
        self.state.url_tab.fail(error);
    }

    pub(super) fn url_failure_identity(
        source_url: &str,
        target: Option<&Path>,
    ) -> Option<(Option<BinaryDocumentFormat>, PathBuf)> {
        let inferred = failed_document_identity(source_url);
        if let Some(target) = target {
            let format = BinaryDocumentFormat::from_path(target)
                .or_else(|| inferred.map(|(format, _)| format));
            return Some((format, target.to_path_buf()));
        }
        inferred.map(|(format, path)| (Some(format), path))
    }

    fn show_requested_url_failure(
        &mut self,
        source_url: &str,
        target: Option<&Path>,
        message: &str,
    ) {
        let Some((format, document_path)) = Self::url_failure_identity(source_url, target) else {
            return;
        };
        let index = self.url_failure_document_index(&document_path);
        self.state.document.active_doc_idx = Some(index);
        self.state.document.scroll_to_active_tab = true;
        self.state.initialize_tab_split_state(document_path.clone());
        self.state
            .set_active_view_mode(crate::state::document::ViewMode::PreviewOnly);
        self.full_refresh_document_failure(
            &document_path,
            crate::preview_pane::DocumentFailure::source_intake(
                "fetch", source_url, format, message,
            ),
        );
    }

    fn url_failure_document_index(&mut self, path: &Path) -> usize {
        if let Some(index) = self
            .state
            .document
            .open_documents
            .iter()
            .position(|document| document.path == path)
        {
            return index;
        }
        let mut document = katana_core::document::Document::new(path.to_path_buf(), String::new());
        document.is_reference = true;
        self.state.document.open_documents.push(document);
        self.state.document.open_documents.len() - 1
    }
}
