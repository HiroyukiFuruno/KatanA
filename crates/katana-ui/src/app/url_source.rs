use crate::state::url_tab::{HtmlSource, HtmlSourceError, UrlValidationError};

pub const MAX_HTML_SOURCE_BYTES: usize = 8 * 1024 * 1024;
const HTTP_SUCCESS_MIN: u16 = 200;
const HTTP_SUCCESS_MAX_EXCLUSIVE: u16 = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedHttpUrl {
    url: String,
}

impl ValidatedHttpUrl {
    pub fn parse(input: &str) -> Result<Self, UrlValidationError> {
        let url = input.trim();
        if url.is_empty() {
            return Err(UrlValidationError::Empty);
        }
        if url
            .chars()
            .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(UrlValidationError::Malformed);
        }

        let parsed = url::Url::parse(url).map_err(|_| UrlValidationError::Malformed)?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(UrlValidationError::UnsupportedScheme);
        }
        if parsed.host().is_none() {
            return Err(UrlValidationError::MissingHost);
        }

        Ok(Self {
            url: parsed.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.url
    }

    pub fn process_response(
        &self,
        response: ehttp::Result<ehttp::Response>,
    ) -> Result<HtmlSource, HtmlSourceError> {
        let response = response.map_err(HtmlSourceError::Network)?;
        if !response.ok
            || !(HTTP_SUCCESS_MIN..HTTP_SUCCESS_MAX_EXCLUSIVE).contains(&response.status)
        {
            return Err(HtmlSourceError::HttpStatus {
                status: response.status,
                status_text: response.status_text,
            });
        }

        let content_type = response.content_type().map(ToOwned::to_owned);
        let is_html = content_type.as_deref().is_some_and(|value| {
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
        });
        if !is_html {
            return Err(HtmlSourceError::NonHtmlContentType { content_type });
        }

        let actual = response.bytes.len();
        if actual > MAX_HTML_SOURCE_BYTES {
            return Err(HtmlSourceError::BodyTooLarge {
                limit: MAX_HTML_SOURCE_BYTES,
                actual,
            });
        }

        let response_url = if response.url.is_empty() {
            self.as_str().to_string()
        } else {
            response.url
        };
        let source_url = ValidatedHttpUrl::parse(&response_url)
            .map_err(HtmlSourceError::InvalidRedirectUrl)?
            .as_str()
            .to_string();
        let raw_html =
            String::from_utf8(response.bytes).map_err(|_| HtmlSourceError::InvalidUtf8)?;

        Ok(HtmlSource {
            raw_html,
            origin: source_url.clone(),
            source_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> ValidatedHttpUrl {
        ValidatedHttpUrl::parse("https://example.com/page").expect("valid request")
    }

    fn response(status: u16, content_type: &str, bytes: Vec<u8>) -> ehttp::Response {
        ehttp::Response {
            url: "https://example.com/page".to_string(),
            ok: (HTTP_SUCCESS_MIN..HTTP_SUCCESS_MAX_EXCLUSIVE).contains(&status),
            status,
            status_text: "status".to_string(),
            headers: ehttp::Headers::new(&[("content-type", content_type)]),
            bytes,
        }
    }

    #[test]
    fn validator_accepts_only_http_and_https_urls_with_a_host() {
        assert!(ValidatedHttpUrl::parse("https://example.com/path").is_ok());
        assert!(ValidatedHttpUrl::parse("http://localhost:3000").is_ok());
        assert_eq!(
            ValidatedHttpUrl::parse("file:///tmp/example.html"),
            Err(UrlValidationError::UnsupportedScheme)
        );
        assert_eq!(
            ValidatedHttpUrl::parse("https://"),
            Err(UrlValidationError::Malformed)
        );
        assert_eq!(
            ValidatedHttpUrl::parse("https://[::1"),
            Err(UrlValidationError::Malformed)
        );
        assert_eq!(
            ValidatedHttpUrl::parse("https://example.com:invalid"),
            Err(UrlValidationError::Malformed)
        );
    }

    #[test]
    fn response_keeps_raw_html_and_final_document_url_as_origin() {
        let source = request()
            .process_response(Ok(response(
                200,
                "text/html; charset=utf-8",
                b"<html></html>".to_vec(),
            )))
            .expect("HTML response");

        assert_eq!(source.raw_html, "<html></html>");
        assert_eq!(source.origin, "https://example.com/page");
    }

    #[test]
    fn response_rejects_non_html_status_large_body_and_network_failures() {
        assert!(matches!(
            request().process_response(Ok(response(200, "application/json", vec![]))),
            Err(HtmlSourceError::NonHtmlContentType { .. })
        ));
        assert!(matches!(
            request().process_response(Ok(response(404, "text/html", vec![]))),
            Err(HtmlSourceError::HttpStatus { status: 404, .. })
        ));
        assert!(matches!(
            request().process_response(Ok(response(
                200,
                "text/html",
                vec![0; MAX_HTML_SOURCE_BYTES + 1]
            ))),
            Err(HtmlSourceError::BodyTooLarge { .. })
        ));
        assert!(matches!(
            request().process_response(Err("connection refused".to_string())),
            Err(HtmlSourceError::Network(_))
        ));
    }

    #[test]
    fn response_rejects_an_invalid_or_unsupported_final_redirect_url() {
        let mut invalid = response(200, "text/html", b"<p>redirected</p>".to_vec());
        invalid.url = "file:///tmp/redirected.html".to_string();

        assert!(matches!(
            request().process_response(Ok(invalid)),
            Err(HtmlSourceError::InvalidRedirectUrl(
                UrlValidationError::UnsupportedScheme
            ))
        ));
    }
}
