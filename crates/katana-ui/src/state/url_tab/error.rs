#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UrlValidationError {
    Empty,
    UnsupportedScheme,
    UnsupportedFileHost,
    MissingHost,
    Malformed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlSourceError {
    InvalidUrl(UrlValidationError),
    InvalidRedirectUrl(UrlValidationError),
    LocalFile {
        url: String,
        reason: String,
    },
    Network(String),
    Timeout {
        url: String,
        seconds: u64,
    },
    HttpStatus {
        status: u16,
        status_text: String,
        url: String,
        server: Option<String>,
        cloudflare_challenge: bool,
    },
    NonHtmlContentType {
        content_type: Option<String>,
    },
    DocumentSource {
        url: String,
        content_type: Option<String>,
        reason: String,
    },
    BodyTooLarge {
        limit: usize,
        actual: usize,
    },
    InvalidUtf8,
}

impl std::fmt::Display for HtmlSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl(error) => write!(formatter, "Invalid URL: {error:?}"),
            Self::InvalidRedirectUrl(error) => {
                write!(formatter, "Invalid final redirect URL: {error:?}")
            }
            Self::LocalFile { url, reason } => {
                write!(formatter, "Local file error: {url}: {reason}")
            }
            Self::Network(error) => write!(formatter, "Network error: {error}"),
            Self::Timeout { url, seconds } => {
                write!(
                    formatter,
                    "URL request timed out after {seconds} seconds: {url}"
                )
            }
            Self::HttpStatus {
                status,
                status_text,
                url,
                server,
                cloudflare_challenge,
            } => {
                let server = server
                    .as_deref()
                    .map_or("server not disclosed", |server| server);
                if *cloudflare_challenge {
                    return write!(
                        formatter,
                        "Main document request failed: HTTP {status}: {status_text}. URL: {url}. \
                         Server: {server}. Cause: browser verification challenge \
                         (cf-mitigated=challenge); CSS and JavaScript were not started."
                    );
                }
                write!(
                    formatter,
                    "Main document request failed: HTTP {status}: {status_text}. \
                     URL: {url}. Server: {server}."
                )
            }
            Self::NonHtmlContentType { content_type } => {
                write!(
                    formatter,
                    "URL response is neither HTML nor a supported document: {content_type:?}"
                )
            }
            Self::DocumentSource {
                url,
                content_type,
                reason,
            } => {
                write!(
                    formatter,
                    "Document source intake failed. URL: {url}. Content-Type: {content_type:?}. Cause: {reason}"
                )
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
