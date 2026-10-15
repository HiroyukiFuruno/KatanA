use crate::shell::KatanaApp;
use crate::state::{FetchedUrlSource, HtmlSourceError, PendingUrlRequest};

impl KatanaApp {
    pub(super) fn consume_url_response(
        &mut self,
        index: usize,
        response: Result<FetchedUrlSource, HtmlSourceError>,
    ) {
        let Some(request) = self.state.url_tab.pending_url_requests.remove(index) else {
            return;
        };
        let Some(order) = request.order else {
            self.fail_unordered_url_request(request);
            return;
        };
        let destination = Self::url_response_destination(&response, &request);
        if destination
            .as_ref()
            .is_some_and(|path| self.state.url_tab.request_order.is_superseded(path, order))
        {
            return;
        }
        self.apply_ordered_url_response(response, request);
        if let Some(path) = destination {
            self.state.url_tab.request_order.complete(path, order);
        }
    }

    fn url_response_destination(
        response: &Result<FetchedUrlSource, HtmlSourceError>,
        request: &PendingUrlRequest,
    ) -> Option<std::path::PathBuf> {
        match response {
            Ok(source) => Some(Self::url_response_document_path(
                source,
                request.target_document.as_deref(),
            )),
            Err(_) => Self::url_request_failure_path(request),
        }
    }

    fn apply_ordered_url_response(
        &mut self,
        response: Result<FetchedUrlSource, HtmlSourceError>,
        request: PendingUrlRequest,
    ) {
        /* WHY: queueをstateに保持し、形式移行による後続targetの付け替えを受け取る。 */
        match response {
            Ok(source) => {
                self.apply_fetched_url_source(source, request.target_document);
                self.state.layout.status_message = None;
            }
            Err(error) => self.fail_requested_url_source(error, request),
        }
    }

    pub(super) fn discard_unordered_url_request(&mut self, index: usize) -> bool {
        let unordered = self
            .state
            .url_tab
            .pending_url_requests
            .get(index)
            .is_some_and(|request| request.order.is_none());
        if unordered && let Some(request) = self.state.url_tab.pending_url_requests.remove(index) {
            self.fail_unordered_url_request(request);
        }
        unordered
    }

    fn fail_unordered_url_request(&mut self, request: PendingUrlRequest) {
        let error = HtmlSourceError::DocumentSource {
            url: request.source_url.clone(),
            content_type: None,
            reason: "URL request order counter exhausted".to_owned(),
        };
        self.fail_requested_url_source(error, request);
    }
}
