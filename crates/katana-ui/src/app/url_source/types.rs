//! URL validation and HTML response boundary types.

use crate::state::url_tab::{HtmlSource, HtmlSourceError, UrlValidationError};

pub const MAX_HTML_SOURCE_BYTES: usize = 8 * 1024 * 1024;
const HTTP_SUCCESS_MIN: u16 = 200;
const HTTP_SUCCESS_MAX_EXCLUSIVE: u16 = 300;
const CHALLENGE_TOKEN: &str = "cf-mitigated=challenge";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedHttpUrl {
    url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedLocalHtmlUrl {
    url: String,
    path: std::path::PathBuf,
}

impl ValidatedLocalHtmlUrl {
    pub fn parse(input: &str) -> Result<Self, UrlValidationError> {
        let input = input.trim();
        if input.is_empty() {
            return Err(UrlValidationError::Empty);
        }
        let parsed = url::Url::parse(input).map_err(|_| UrlValidationError::Malformed)?;
        if parsed.scheme() != "file" {
            return Err(UrlValidationError::UnsupportedScheme);
        }
        if parsed
            .host_str()
            .is_some_and(|host| !host.is_empty() && host != "localhost")
        {
            return Err(UrlValidationError::UnsupportedFileHost);
        }
        let path = parsed
            .to_file_path()
            .map_err(|_| UrlValidationError::Malformed)?;
        Ok(Self {
            url: parsed.to_string(),
            path,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.url
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn canonical_url_for(&self, path: &std::path::Path) -> Result<String, UrlValidationError> {
        let original = url::Url::parse(&self.url).map_err(|_| UrlValidationError::Malformed)?;
        let mut canonical =
            url::Url::from_file_path(path).map_err(|_| UrlValidationError::Malformed)?;
        canonical.set_query(original.query());
        canonical.set_fragment(original.fragment());
        Ok(canonical.to_string())
    }
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

        let source_url = self.final_document_url(&response_url)?;
        let raw_html =
            String::from_utf8(response.bytes).map_err(|_| HtmlSourceError::InvalidUtf8)?;

        Ok(HtmlSource {
            raw_html,
            origin: source_url.clone(),
            source_url,
        })
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
    fn local_html_validator_accepts_file_urls_and_decodes_paths() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("local document.html");
        let url = url::Url::from_file_path(&path).expect("file URL");

        let validated = ValidatedLocalHtmlUrl::parse(url.as_str()).expect("local HTML URL");

        assert_eq!(validated.path(), path);
        assert_eq!(validated.as_str(), url.as_str());
        assert_eq!(
            ValidatedLocalHtmlUrl::parse("https://example.com/index.html"),
            Err(UrlValidationError::UnsupportedScheme)
        );
        assert_eq!(
            ValidatedLocalHtmlUrl::parse("file://example.com/index.html"),
            Err(UrlValidationError::UnsupportedFileHost)
        );
    }

    #[test]
    fn local_file_url_canonicalization_preserves_query_and_fragment() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let source_path = directory.path().join("source document.html");
        let canonical_path = directory.path().join("canonical document.html");
        let mut source_url = url::Url::from_file_path(source_path).expect("source file URL");
        source_url.set_query(Some("slide=2"));
        source_url.set_fragment(Some("deck"));
        let local = ValidatedLocalHtmlUrl::parse(source_url.as_str()).expect("local URL");
        let canonical = local
            .canonical_url_for(&canonical_path)
            .expect("canonical URL");
        let mut expected = url::Url::from_file_path(canonical_path).expect("canonical file URL");
        expected.set_query(source_url.query());
        expected.set_fragment(source_url.fragment());

        assert_eq!(canonical, expected.as_str());
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
    fn response_inherits_requested_fragment_when_http_final_url_omits_it() {
        let request =
            ValidatedHttpUrl::parse("https://example.com/start#linked-target").expect("request");
        let mut redirected = response(200, "text/html", b"<html></html>".to_vec());
        redirected.url = "https://example.com/final".to_string();

        let source = request.process_response(Ok(redirected)).expect("source");

        assert_eq!(source.origin, "https://example.com/final#linked-target");
        assert_eq!(source.source_url, source.origin);
    }

    #[test]
    fn response_fragment_overrides_requested_fragment() {
        let request = ValidatedHttpUrl::parse("https://example.com/start#old").expect("request");
        let mut redirected = response(200, "text/html", b"<html></html>".to_vec());
        redirected.url = "https://example.com/final#new".to_string();

        let source = request.process_response(Ok(redirected)).expect("source");

        assert_eq!(source.origin, "https://example.com/final#new");
    }

    #[test]
    fn response_rejects_non_html_status_large_body_and_network_failures() {
        assert!(matches!(
            request().process_response(Ok(response(200, "application/json", vec![]))),
            Err(HtmlSourceError::NonHtmlContentType { .. })
        ));
        match request().process_response(Ok(response(404, "text/html", vec![]))) {
            Err(HtmlSourceError::HttpStatus {
                status: 404,
                status_text,
                url,
                server: None,
                cloudflare_challenge: false,
            }) => {
                assert_eq!(status_text, "status");
                assert_eq!(url, "https://example.com/page");
            }
            Err(error) => panic!("expected http status error, got {error:?}"),
            Ok(_) => panic!("expected error response"),
        }

        let mut challenge_response = response(403, "text/html", vec![]);
        challenge_response.headers = ehttp::Headers::new(&[
            ("content-type", "text/html"),
            ("server", "cloudflare"),
            ("set-cookie", "id=abc; cf-mitigated=challenge; path=/"),
        ]);
        match request().process_response(Ok(challenge_response)) {
            Err(HtmlSourceError::HttpStatus {
                status: 403,
                status_text,
                url,
                server: Some(server),
                cloudflare_challenge: true,
            }) => {
                assert_eq!(status_text, "status");
                assert_eq!(url, "https://example.com/page");
                assert_eq!(server, "cloudflare");
            }
            Err(error) => panic!("expected http status error, got {error:?}"),
            Ok(_) => panic!("expected error response"),
        }

        let mut challenge_header_response = response(403, "text/html", vec![]);
        challenge_header_response.headers = ehttp::Headers::new(&[
            ("content-type", "text/html"),
            ("server", "cloudflare"),
            ("cf-mitigated", "challenge"),
        ]);
        assert!(matches!(
            request().process_response(Ok(challenge_header_response)),
            Err(HtmlSourceError::HttpStatus {
                cloudflare_challenge: true,
                ..
            })
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
