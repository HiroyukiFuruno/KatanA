use super::URL_DOCUMENT_PREFIX;
use crate::app::preview::PreviewOps;
use crate::app_state::StatusType;
use crate::shell::KatanaApp;
use crate::state::{BinaryUrlSource, FetchedUrlSource, HtmlSourceError};

impl KatanaApp {
    pub(super) fn apply_fetched_url_source(
        &mut self,
        source: FetchedUrlSource,
        target_document: Option<std::path::PathBuf>,
    ) {
        match source {
            FetchedUrlSource::Html(source) => {
                let document_path = target_document
                    .unwrap_or_else(|| remote_document_path(&source.source_url, "html"));
                self.state
                    .url_tab
                    .open_source(source.clone(), document_path.clone());
                self.replace_html_document(source, document_path, None);
            }
            FetchedUrlSource::Document(source) => {
                self.apply_fetched_document_source(source, target_document);
            }
        }
    }

    fn apply_fetched_document_source(
        &mut self,
        source: BinaryUrlSource,
        target_document: Option<std::path::PathBuf>,
    ) {
        let BinaryUrlSource {
            bytes,
            source_url,
            mime,
            format,
        } = source;
        let document_path = target_document
            .unwrap_or_else(|| remote_document_path(&source_url, format.extension()));
        let surface_source = match crate::preview_pane::DocumentSurfaceSource::remote(
            source_url.clone(),
            Some(&mime),
            bytes,
        ) {
            Ok(source) => source,
            Err(error) => {
                self.full_refresh_document_failure(&document_path, error.clone());
                self.state.layout.status_message = Some((error.to_string(), StatusType::Error));
                return;
            }
        };
        self.state
            .url_tab
            .open_document_source(source_url, document_path.clone());
        let index = self
            .state
            .document
            .open_documents
            .iter()
            .position(|document| document.path == document_path)
            .unwrap_or_else(|| {
                let mut document =
                    katana_core::document::Document::new(document_path.clone(), String::new());
                document.is_reference = true;
                self.state.document.open_documents.push(document);
                self.state.document.open_documents.len() - 1
            });
        self.state.document.active_doc_idx = Some(index);
        self.state.document.scroll_to_active_tab = true;
        self.state.initialize_tab_split_state(document_path.clone());
        self.state
            .set_active_view_mode(crate::state::document::ViewMode::PreviewOnly);
        self.full_refresh_document_source(&document_path, surface_source, true);
    }

    pub(in crate::app::action) fn fail_html_source(&mut self, error: HtmlSourceError) {
        let message = error.to_string();
        self.show_document_url_failure(&error, &message);
        self.state.layout.status_message = Some((message, StatusType::Error));
        self.state.url_tab.fail(error);
    }

    fn show_document_url_failure(&mut self, error: &HtmlSourceError, message: &str) {
        let source_url = match error {
            HtmlSourceError::HttpStatus { url, .. }
            | HtmlSourceError::DocumentSource { url, .. }
            | HtmlSourceError::LocalFile { url, .. }
            | HtmlSourceError::Timeout { url, .. } => url.clone(),
            _ => self.state.url_tab.input.clone(),
        };
        let Some((format, document_path)) = failed_document_identity(&source_url) else {
            return;
        };
        let index = self
            .state
            .document
            .open_documents
            .iter()
            .position(|document| document.path == document_path)
            .unwrap_or_else(|| {
                let mut document =
                    katana_core::document::Document::new(document_path.clone(), String::new());
                document.is_reference = true;
                self.state.document.open_documents.push(document);
                self.state.document.open_documents.len() - 1
            });
        self.state.document.active_doc_idx = Some(index);
        self.state.document.scroll_to_active_tab = true;
        self.state.initialize_tab_split_state(document_path.clone());
        self.state
            .set_active_view_mode(crate::state::document::ViewMode::PreviewOnly);
        self.full_refresh_document_failure(
            &document_path,
            crate::preview_pane::DocumentFailure::source_intake(
                "fetch",
                source_url,
                Some(format),
                message,
            ),
        );
    }
}

pub(super) fn remote_document_path(source_url: &str, extension: &str) -> std::path::PathBuf {
    let hash = katana_core::document::DocumentOps::compute_hash(source_url);
    std::path::PathBuf::from(format!("{URL_DOCUMENT_PREFIX}/{hash:x}.{extension}"))
}

pub(super) fn failed_document_identity(
    source_url: &str,
) -> Option<(
    katana_core::document_source::BinaryDocumentFormat,
    std::path::PathBuf,
)> {
    let parsed = url::Url::parse(source_url).ok()?;
    let source_path = std::path::Path::new(parsed.path());
    let format = katana_core::document_source::BinaryDocumentFormat::from_path(source_path)?;
    let document_path = if parsed.scheme() == "file" {
        parsed.to_file_path().ok()?
    } else {
        remote_document_path(source_url, format.extension())
    };
    Some((format, document_path))
}
