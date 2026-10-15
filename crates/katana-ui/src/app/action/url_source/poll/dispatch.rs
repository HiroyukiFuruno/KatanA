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
        if self.consume_superseded_url_selection(destination.as_deref(), &request, order) {
            return;
        }
        let background = self.url_response_is_background(&request, order);
        self.apply_selected_url_response(response, request, order);
        self.record_url_completion(destination, order, background);
    }

    fn consume_superseded_url_selection(
        &mut self,
        destination: Option<&std::path::Path>,
        request: &PendingUrlRequest,
        order: std::num::NonZeroU64,
    ) -> bool {
        let Some(path) =
            destination.filter(|path| self.state.url_tab.request_order.is_superseded(path, order))
        else {
            return false;
        };
        /* WHY: 古い本文は破棄するが、有効なユーザー選択は取得済みの最新版へ適用する。 */
        if !self.url_response_is_background(request, order) && self.select_cached_url_document(path)
        {
            self.state
                .url_tab
                .request_order
                .complete(Some(path.to_path_buf()), order);
        }
        true
    }

    fn record_url_completion(
        &mut self,
        destination: Option<std::path::PathBuf>,
        order: std::num::NonZeroU64,
        background: bool,
    ) {
        if background {
            self.state
                .url_tab
                .request_order
                .complete_destination(destination, order);
        } else {
            self.state
                .url_tab
                .request_order
                .complete(destination, order);
        }
    }

    pub(super) fn url_response_destination(
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

    pub(super) fn apply_ordered_url_response(
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
