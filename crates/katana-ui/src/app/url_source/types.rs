//! URL validation and HTML response boundary types.

mod response;

use crate::state::url_tab::UrlValidationError;

pub(super) use response::response_body_limit;

pub const MAX_HTML_SOURCE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_DOCUMENT_SOURCE_BYTES: usize =
    katana_core::document_source::MAX_BINARY_DOCUMENT_BYTES;
const HTTP_SUCCESS_MIN: u16 = 200;
const HTTP_SUCCESS_MAX_EXCLUSIVE: u16 = 300;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::url_tab::{FetchedUrlSource, HtmlSourceError};

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
        let FetchedUrlSource::Html(source) = source else {
            panic!("expected HTML source");
        };

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
        let FetchedUrlSource::Html(source) = source else {
            panic!("expected HTML source");
        };

        assert_eq!(source.origin, "https://example.com/final#linked-target");
        assert_eq!(source.source_url, source.origin);
    }

    #[test]
    fn response_fragment_overrides_requested_fragment() {
        let request = ValidatedHttpUrl::parse("https://example.com/start#old").expect("request");
        let mut redirected = response(200, "text/html", b"<html></html>".to_vec());
        redirected.url = "https://example.com/final#new".to_string();

        let source = request.process_response(Ok(redirected)).expect("source");
        let FetchedUrlSource::Html(source) = source else {
            panic!("expected HTML source");
        };

        assert_eq!(source.origin, "https://example.com/final#new");
    }

    #[test]
    fn document_url_rejects_html_authentication_pages() {
        let request =
            ValidatedHttpUrl::parse("https://example.com/report.pdf").expect("document URL");
        let mut response = response(200, "text/html", b"<html>Sign in</html>".to_vec());
        response.url = "https://example.com/sign-in".to_owned();

        let failure = request
            .process_response(Ok(response))
            .expect_err("HTML must not replace a PDF document");

        assert!(matches!(failure, HtmlSourceError::DocumentSource { .. }));
        assert!(failure.to_string().contains("authentication or error page"));
        assert!(failure.to_string().contains("https://example.com/sign-in"));
    }

    #[test]
    fn response_routes_pdf_to_a_binary_source() {
        let pdf_request =
            ValidatedHttpUrl::parse("https://example.com/report.pdf").expect("PDF URL");
        let mut pdf_response = response(200, "application/pdf", b"%PDF-1.7".to_vec());
        pdf_response.url = pdf_request.as_str().to_owned();
        let pdf = pdf_request
            .process_response(Ok(pdf_response))
            .expect("PDF response");
        let FetchedUrlSource::Document(pdf) = pdf else {
            panic!("expected PDF document source");
        };
        assert_eq!(
            pdf.format,
            katana_core::document_source::BinaryDocumentFormat::Pdf
        );
        assert_eq!(pdf.mime, "application/pdf");
        assert_eq!(pdf.source_url, pdf_request.as_str());
        assert_eq!(pdf.bytes, b"%PDF-1.7");
    }

    #[test]
    fn response_routes_mime_only_ooxml_to_a_binary_source() {
        let docx_request =
            ValidatedHttpUrl::parse("https://example.com/download").expect("DOCX URL");
        let docx = docx_request
            .process_response(Ok(response(
                200,
                katana_core::document_source::BinaryDocumentFormat::Docx.mime(),
                vec![0x50, 0x4b, 0x03, 0x04],
            )))
            .expect("DOCX response");
        let FetchedUrlSource::Document(docx) = docx else {
            panic!("expected DOCX document source");
        };
        assert_eq!(
            docx.format,
            katana_core::document_source::BinaryDocumentFormat::Docx
        );
        assert_eq!(docx.mime, docx.format.mime());
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
