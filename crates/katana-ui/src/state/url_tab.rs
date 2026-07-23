use std::collections::VecDeque;

pub const MAX_URL_HISTORY: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlSource {
    pub raw_html: String,
    pub source_url: String,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlTab {
    pub source: HtmlSource,
    pub document_path: std::path::PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlValidationError {
    Empty,
    UnsupportedScheme,
    MissingHost,
    Malformed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlSourceError {
    InvalidUrl(UrlValidationError),
    InvalidRedirectUrl(UrlValidationError),
    Network(String),
    HttpStatus { status: u16, status_text: String },
    NonHtmlContentType { content_type: Option<String> },
    BodyTooLarge { limit: usize, actual: usize },
    InvalidUtf8,
}

pub(crate) type PendingUrlRequest = (
    std::sync::mpsc::Receiver<Result<HtmlSource, HtmlSourceError>>,
    Option<std::path::PathBuf>,
);

impl std::fmt::Display for HtmlSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(error) => write!(formatter, "Invalid URL: {error:?}"),
            Self::InvalidRedirectUrl(error) => {
                write!(formatter, "Invalid final redirect URL: {error:?}")
            }
            Self::Network(error) => write!(formatter, "Network error: {error}"),
            Self::HttpStatus {
                status,
                status_text,
            } => write!(formatter, "HTTP {status}: {status_text}"),
            Self::NonHtmlContentType { content_type } => {
                write!(formatter, "Expected HTML content, got {content_type:?}")
            }
            Self::BodyTooLarge { limit, actual } => {
                write!(
                    formatter,
                    "Response body is {actual} bytes; limit is {limit} bytes"
                )
            }
            Self::InvalidUtf8 => write!(formatter, "Response body is not valid UTF-8"),
        }
    }
}

pub struct UrlTabState {
    pub input: String,
    pub history: VecDeque<String>,
    pub tabs: Vec<UrlTab>,
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
}
