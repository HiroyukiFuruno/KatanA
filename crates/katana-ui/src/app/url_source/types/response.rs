use super::{
    HTTP_SUCCESS_MAX_EXCLUSIVE, HTTP_SUCCESS_MIN, MAX_DOCUMENT_SOURCE_BYTES, MAX_HTML_SOURCE_BYTES,
    ValidatedHttpUrl,
};
use crate::state::url_tab::{
    BinaryUrlSource, FetchedUrlSource, HtmlSource, HtmlSourceError, UrlValidationError,
};

const CHALLENGE_TOKEN: &str = "cf-mitigated=challenge";

impl ValidatedHttpUrl {
    pub fn process_response(
        &self,
        response: ehttp::Result<ehttp::Response>,
    ) -> Result<FetchedUrlSource, HtmlSourceError> {
        let response = response.map_err(HtmlSourceError::Network)?;
        let response_url = if response.url.is_empty() {
            self.as_str().to_string()
        } else {
            response.url.clone()
        };
        if !response.ok
            || !(HTTP_SUCCESS_MIN..HTTP_SUCCESS_MAX_EXCLUSIVE).contains(&response.status)
        {
            return Err(HtmlSourceError::HttpStatus {
                status: response.status,
                status_text: response.status_text,
                url: response_url,
                server: response
                    .headers
                    .get("server")
                    .map(std::string::ToString::to_string),
                cloudflare_challenge: has_cloudflare_challenge(&response.headers),
            });
        }

        let source_url = self.final_document_url(&response_url)?;
        let content_type = response.content_type().map(ToOwned::to_owned);
        if is_html_content_type(content_type.as_deref()) {
            if expected_document_format(self.as_str()).is_some()
                || expected_document_format(&source_url).is_some()
            {
                return Err(HtmlSourceError::DocumentSource {
                    url: self.as_str().to_owned(),
                    content_type,
                    reason: format!(
                        "document URL returned HTML content at {source_url}, such as an authentication or error page"
                    ),
                });
            }
            return html_source(source_url, response.bytes);
        }
        document_source(source_url, content_type, response.bytes)
    }

    fn final_document_url(&self, response_url: &str) -> Result<String, HtmlSourceError> {
        let response_url =
            ValidatedHttpUrl::parse(response_url).map_err(HtmlSourceError::InvalidRedirectUrl)?;
        let mut final_url = url::Url::parse(response_url.as_str())
            .map_err(|_| HtmlSourceError::InvalidRedirectUrl(UrlValidationError::Malformed))?;
        let requested_url = url::Url::parse(self.as_str())
            .map_err(|_| HtmlSourceError::InvalidRedirectUrl(UrlValidationError::Malformed))?;
        if final_url.fragment().is_none() {
            final_url.set_fragment(requested_url.fragment());
        }
        Ok(final_url.to_string())
    }
}

fn html_source(source_url: String, bytes: Vec<u8>) -> Result<FetchedUrlSource, HtmlSourceError> {
    enforce_body_limit(bytes.len(), MAX_HTML_SOURCE_BYTES)?;
    let raw_html = String::from_utf8(bytes).map_err(|_| HtmlSourceError::InvalidUtf8)?;
    Ok(FetchedUrlSource::Html(HtmlSource {
        raw_html,
        origin: source_url.clone(),
        source_url,
    }))
}

fn document_source(
    source_url: String,
    content_type: Option<String>,
    bytes: Vec<u8>,
) -> Result<FetchedUrlSource, HtmlSourceError> {
    enforce_body_limit(bytes.len(), MAX_DOCUMENT_SOURCE_BYTES)?;
    let path = url::Url::parse(&source_url)
        .ok()
        .map(|url| std::path::PathBuf::from(url.path()))
        .unwrap_or_default();
    let format = katana_core::document_source::BinaryDocumentFormat::detect(
        &path,
        content_type.as_deref(),
        &bytes,
    )
    .map_err(|error| match error {
        katana_core::document_source::DocumentSourceError::UnsupportedContentType(_)
        | katana_core::document_source::DocumentSourceError::UnsupportedFormat => {
            HtmlSourceError::NonHtmlContentType {
                content_type: content_type.clone(),
            }
        }
        error => HtmlSourceError::DocumentSource {
            url: source_url.clone(),
            content_type: content_type.clone(),
            reason: error.to_string(),
        },
    })?;
    Ok(FetchedUrlSource::Document(BinaryUrlSource {
        bytes,
        source_url,
        mime: format.mime().to_owned(),
        format,
    }))
}

fn expected_document_format(
    source_url: &str,
) -> Option<katana_core::document_source::BinaryDocumentFormat> {
    let url = url::Url::parse(source_url).ok()?;
    katana_core::document_source::BinaryDocumentFormat::from_path(std::path::Path::new(url.path()))
}

fn enforce_body_limit(actual: usize, limit: usize) -> Result<(), HtmlSourceError> {
    if actual <= limit {
        return Ok(());
    }
    Err(HtmlSourceError::BodyTooLarge { limit, actual })
}

pub(in crate::app::url_source) fn response_body_limit(content_type: Option<&str>) -> usize {
    if is_html_content_type(content_type) {
        MAX_HTML_SOURCE_BYTES
    } else {
        MAX_DOCUMENT_SOURCE_BYTES
    }
}

fn is_html_content_type(content_type: Option<&str>) -> bool {
    content_type.is_some_and(|value| {
        matches!(
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "text/html" | "application/xhtml+xml"
        )
    })
}

fn has_cloudflare_challenge(headers: &ehttp::Headers) -> bool {
    if headers
        .get("cf-mitigated")
        .is_some_and(|value| value.eq_ignore_ascii_case("challenge"))
    {
        return true;
    }

    headers
        .get_all("set-cookie")
        .any(|value| value.to_ascii_lowercase().contains(CHALLENGE_TOKEN))
}
