use super::types::{ValidatedHttpUrl, response_body_limit};
use crate::state::{FetchedUrlSource, HtmlSourceError};

pub(in crate::app) struct UrlResponseCollector {
    request: ValidatedHttpUrl,
    response: Option<ehttp::PartialResponse>,
    bytes: Vec<u8>,
    body_limit: usize,
}

impl UrlResponseCollector {
    pub(in crate::app) fn new(request: ValidatedHttpUrl) -> Self {
        Self {
            request,
            response: None,
            bytes: Vec::new(),
            body_limit: 0,
        }
    }

    pub(in crate::app) fn accept(
        &mut self,
        part: ehttp::Result<ehttp::streaming::Part>,
    ) -> Option<Result<FetchedUrlSource, HtmlSourceError>> {
        match part {
            Ok(ehttp::streaming::Part::Response(response)) => self.accept_response(response),
            Ok(ehttp::streaming::Part::Chunk(chunk)) => self.accept_chunk(chunk),
            Err(error) => Some(Err(HtmlSourceError::Network(error))),
        }
    }

    fn accept_response(
        &mut self,
        response: ehttp::PartialResponse,
    ) -> Option<Result<FetchedUrlSource, HtmlSourceError>> {
        if self.response.is_some() {
            return Some(Err(Self::protocol_error("response headers received twice")));
        }
        if !response.ok {
            return Some(
                self.request
                    .process_response(Ok(response.complete(Vec::new()))),
            );
        }
        self.body_limit = response_body_limit(response.headers.get("content-type"));
        if let Some(actual) = Self::declared_body_size(&response)
            && actual > self.body_limit
        {
            return Some(Err(HtmlSourceError::BodyTooLarge {
                limit: self.body_limit,
                actual,
            }));
        }
        self.response = Some(response);
        None
    }

    fn accept_chunk(
        &mut self,
        chunk: Vec<u8>,
    ) -> Option<Result<FetchedUrlSource, HtmlSourceError>> {
        if self.response.is_none() {
            return Some(Err(Self::protocol_error(
                "body received before response headers",
            )));
        }
        if chunk.is_empty() {
            return Some(self.finish());
        }
        let actual = self.bytes.len().saturating_add(chunk.len());
        if actual > self.body_limit {
            return Some(Err(HtmlSourceError::BodyTooLarge {
                limit: self.body_limit,
                actual,
            }));
        }
        self.bytes.extend_from_slice(&chunk);
        None
    }

    fn finish(&mut self) -> Result<FetchedUrlSource, HtmlSourceError> {
        let response = self
            .response
            .take()
            .ok_or_else(|| Self::protocol_error("response headers are unavailable"))?;
        let bytes = std::mem::take(&mut self.bytes);
        self.request.process_response(Ok(response.complete(bytes)))
    }

    fn declared_body_size(response: &ehttp::PartialResponse) -> Option<usize> {
        response
            .headers
            .get("content-length")
            .and_then(|value| value.trim().parse().ok())
    }

    fn protocol_error(message: &str) -> HtmlSourceError {
        HtmlSourceError::Network(format!("invalid streaming response: {message}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::url_source::types::MAX_HTML_SOURCE_BYTES;

    fn request() -> ValidatedHttpUrl {
        ValidatedHttpUrl::parse("https://example.com/page").expect("valid URL")
    }

    fn response(status: u16, headers: &[(&str, &str)]) -> ehttp::PartialResponse {
        ehttp::PartialResponse {
            url: "https://example.com/page".to_owned(),
            ok: (200..300).contains(&status),
            status,
            status_text: "status".to_owned(),
            headers: ehttp::Headers::new(headers),
        }
    }

    #[test]
    fn streams_html_to_completion() {
        let mut collector = UrlResponseCollector::new(request());
        assert!(
            collector
                .accept(Ok(ehttp::streaming::Part::Response(response(
                    200,
                    &[("content-type", "text/html"), ("content-length", "11")],
                ))))
                .is_none()
        );
        assert!(
            collector
                .accept(Ok(ehttp::streaming::Part::Chunk(b"<p>ok".to_vec())))
                .is_none()
        );
        assert!(
            collector
                .accept(Ok(ehttp::streaming::Part::Chunk(b"</p>".to_vec())))
                .is_none()
        );
        let result = collector
            .accept(Ok(ehttp::streaming::Part::Chunk(Vec::new())))
            .expect("completed response")
            .expect("HTML source");
        let FetchedUrlSource::Html(source) = result else {
            panic!("expected HTML source");
        };
        assert_eq!("<p>ok</p>", source.raw_html);
    }

    #[test]
    fn rejects_declared_and_streamed_oversized_bodies() {
        let mut declared = UrlResponseCollector::new(request());
        let length = (MAX_HTML_SOURCE_BYTES + 1).to_string();
        assert!(matches!(
            declared.accept(Ok(ehttp::streaming::Part::Response(response(
                200,
                &[("content-type", "text/html"), ("content-length", &length)],
            )))),
            Some(Err(HtmlSourceError::BodyTooLarge { .. }))
        ));

        let mut streamed = UrlResponseCollector::new(request());
        assert!(
            streamed
                .accept(Ok(ehttp::streaming::Part::Response(response(
                    200,
                    &[("content-type", "text/html"), ("content-length", "invalid")],
                ))))
                .is_none()
        );
        assert!(
            streamed
                .accept(Ok(ehttp::streaming::Part::Chunk(vec![
                    0;
                    MAX_HTML_SOURCE_BYTES
                ])))
                .is_none()
        );
        assert!(matches!(
            streamed.accept(Ok(ehttp::streaming::Part::Chunk(vec![0]))),
            Some(Err(HtmlSourceError::BodyTooLarge { .. }))
        ));
    }

    #[test]
    fn rejects_status_network_and_protocol_failures() {
        let mut status = UrlResponseCollector::new(request());
        assert!(matches!(
            status.accept(Ok(ehttp::streaming::Part::Response(response(404, &[])))),
            Some(Err(HtmlSourceError::HttpStatus { status: 404, .. }))
        ));
        let mut network = UrlResponseCollector::new(request());
        assert!(matches!(
            network.accept(Err("offline".to_owned())),
            Some(Err(HtmlSourceError::Network(_)))
        ));
        let mut missing_headers = UrlResponseCollector::new(request());
        assert!(matches!(
            missing_headers.accept(Ok(ehttp::streaming::Part::Chunk(vec![1]))),
            Some(Err(HtmlSourceError::Network(_)))
        ));
        let mut duplicate = UrlResponseCollector::new(request());
        assert!(
            duplicate
                .accept(Ok(ehttp::streaming::Part::Response(response(200, &[]))))
                .is_none()
        );
        assert!(matches!(
            duplicate.accept(Ok(ehttp::streaming::Part::Response(response(200, &[])))),
            Some(Err(HtmlSourceError::Network(_)))
        ));
    }
}
