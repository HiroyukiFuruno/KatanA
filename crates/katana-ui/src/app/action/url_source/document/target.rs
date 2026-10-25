use super::remote_document_path;
use crate::shell::KatanaApp;
use crate::state::{FetchedUrlSource, PendingUrlRequest};
use katana_core::document_source::BinaryDocumentFormat;
use std::path::{Path, PathBuf};

impl KatanaApp {
    pub(in crate::app::action::url_source) fn url_response_document_path(
        source: &FetchedUrlSource,
        target: Option<&Path>,
    ) -> PathBuf {
        let (source_url, format) = match source {
            FetchedUrlSource::Html(source) => (source.source_url.as_str(), None),
            FetchedUrlSource::Document(source) => (source.source_url.as_str(), Some(source.format)),
        };
        /* WHY: 反映と順序照合で同じ移行先を使い、redirect後の文書合流を見落とさない。 */
        target
            .filter(|path| BinaryDocumentFormat::from_path(path) == format)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| {
                remote_document_path(
                    source_url,
                    format.map_or("html", BinaryDocumentFormat::extension),
                )
            })
    }

    pub(in crate::app::action::url_source) fn url_request_failure_path(
        request: &PendingUrlRequest,
    ) -> Option<PathBuf> {
        Self::url_failure_identity(&request.source_url, request.target_document.as_deref())
            .map(|(_, path)| path)
    }
}
