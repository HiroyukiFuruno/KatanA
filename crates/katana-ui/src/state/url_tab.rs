mod error;

use std::collections::VecDeque;

pub use error::{HtmlSourceError, UrlValidationError};

pub const MAX_URL_HISTORY: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlSource {
    pub raw_html: String,
    pub source_url: String,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryUrlSource {
    pub bytes: Vec<u8>,
    pub source_url: String,
    pub mime: String,
    pub format: katana_core::document_source::BinaryDocumentFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchedUrlSource {
    Html(HtmlSource),
    Document(BinaryUrlSource),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlTab {
    pub source: HtmlSource,
    pub document_path: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryUrlTab {
    pub source_url: String,
    pub document_path: std::path::PathBuf,
}

pub(crate) struct PendingUrlRequest {
    pub(crate) response_rx: std::sync::mpsc::Receiver<Result<FetchedUrlSource, HtmlSourceError>>,
    pub(crate) target_document: Option<std::path::PathBuf>,
    pub(crate) source_url: String,
    pub(crate) deadline: std::time::Instant,
}

pub struct UrlTabState {
    pub input: String,
    pub history: VecDeque<String>,
    pub tabs: Vec<UrlTab>,
    pub document_tabs: Vec<BinaryUrlTab>,
    pub active_tab: Option<usize>,
    pub is_loading: bool,
    pub last_error: Option<HtmlSourceError>,
    pub(crate) pending_url_requests: VecDeque<PendingUrlRequest>,
}

impl Default for UrlTabState {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlTabState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            history: VecDeque::with_capacity(MAX_URL_HISTORY),
            tabs: Vec::new(),
            document_tabs: Vec::new(),
            active_tab: None,
            is_loading: false,
            last_error: None,
            pending_url_requests: VecDeque::new(),
        }
    }

    pub fn open_source(&mut self, source: HtmlSource, document_path: std::path::PathBuf) {
        self.input = source.source_url.clone();
        self.last_error = None;
        self.is_loading = false;
        self.record_history(&source.source_url);

        if let Some(index) = self
            .tabs
            .iter()
            .position(|tab| tab.document_path == document_path)
        {
            self.tabs[index] = UrlTab {
                source,
                document_path,
            };
            self.active_tab = Some(index);
        } else {
            self.tabs.push(UrlTab {
                source,
                document_path,
            });
            self.active_tab = Some(self.tabs.len() - 1);
        }
    }

    pub fn source_for_document(&self, document_path: &std::path::Path) -> Option<&HtmlSource> {
        self.tabs
            .iter()
            .find(|tab| tab.document_path == document_path)
            .map(|tab| &tab.source)
    }

    pub fn document_source_url_for_document(
        &self,
        document_path: &std::path::Path,
    ) -> Option<&str> {
        self.document_tabs
            .iter()
            .find(|tab| tab.document_path == document_path)
            .map(|tab| tab.source_url.as_str())
    }

    pub fn open_document_source(&mut self, source_url: String, document_path: std::path::PathBuf) {
        self.input = source_url.clone();
        self.last_error = None;
        self.is_loading = false;
        self.record_history(&source_url);
        if let Some(index) = self
            .document_tabs
            .iter()
            .position(|tab| tab.document_path == document_path)
        {
            self.document_tabs[index] = BinaryUrlTab {
                source_url,
                document_path,
            };
        } else {
            self.document_tabs.push(BinaryUrlTab {
                source_url,
                document_path,
            });
        }
    }

    pub(crate) fn cancel_pending_url_requests(&mut self) {
        self.pending_url_requests.clear();
        self.is_loading = false;
    }

    pub fn fail(&mut self, error: HtmlSourceError) {
        self.is_loading = false;
        self.last_error = Some(error);
    }

    fn record_history(&mut self, url: &str) {
        self.history.retain(|entry| entry != url);
        self.history.push_front(url.to_string());
        self.history.truncate(MAX_URL_HISTORY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_url_tabs_keep_refresh_identity_without_retaining_payloads() {
        let mut state = UrlTabState::new();
        let path = std::path::PathBuf::from("Katana://URL/report.pdf");
        state.open_document_source("https://example.test/first.pdf".to_owned(), path.clone());
        assert_eq!(
            Some("https://example.test/first.pdf"),
            state.document_source_url_for_document(&path)
        );

        state.open_document_source("https://example.test/final.pdf".to_owned(), path.clone());
        assert_eq!(1, state.document_tabs.len());
        assert_eq!(
            Some("https://example.test/final.pdf"),
            state.document_source_url_for_document(&path)
        );
        assert_eq!(
            Some("https://example.test/final.pdf"),
            state.history.front().map(String::as_str)
        );
    }

    fn source(url: &str) -> HtmlSource {
        HtmlSource {
            raw_html: "<html></html>".to_string(),
            source_url: url.to_string(),
            origin: "https://example.com".to_string(),
        }
    }

    #[test]
    fn opening_existing_url_replaces_its_source_and_keeps_a_single_tab() {
        let mut state = UrlTabState::new();
        let path = std::path::PathBuf::from("Katana://URL/example.html");
        state.open_source(source("https://example.com"), path.clone());
        state.open_source(
            HtmlSource {
                raw_html: "<html><body>updated</body></html>".to_string(),
                ..source("https://example.com")
            },
            path.clone(),
        );

        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.active_tab, Some(0));
        assert!(state.tabs[0].source.raw_html.contains("updated"));
        assert_eq!(
            state.history,
            VecDeque::from(["https://example.com".to_string()])
        );
        assert_eq!(
            state.source_for_document(&path).unwrap().origin,
            "https://example.com"
        );
    }

    #[test]
    fn http_status_error_includes_url_server_and_cloudflare_marker_when_printed() {
        let error = HtmlSourceError::HttpStatus {
            status: 403,
            status_text: "Forbidden".to_string(),
            url: "https://example.com/".to_string(),
            server: Some("cloudflare".to_string()),
            cloudflare_challenge: true,
        };
        let rendered = error.to_string();

        assert!(rendered.contains("Main document request failed"));
        assert!(rendered.contains("HTTP 403: Forbidden"));
        assert!(rendered.contains("URL: https://example.com/"));
        assert!(rendered.contains("Server: cloudflare"));
        assert!(rendered.contains("browser verification challenge"));
        assert!(rendered.contains("cf-mitigated=challenge"));
        assert!(rendered.contains("CSS and JavaScript were not started"));
    }
}
