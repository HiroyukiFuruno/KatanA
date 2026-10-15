use super::URL_SOURCE_TIMEOUT;
use crate::shell::KatanaApp;
use crate::state::{FetchedUrlSource, HtmlSourceError, PendingUrlRequest};
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

const URL_SOURCE_REPAINT_INTERVAL: Duration = Duration::from_millis(100);

impl KatanaApp {
    pub(crate) fn poll_url_source(&mut self, ctx: &egui::Context) {
        let mut index = 0;
        while index < self.state.url_tab.pending_url_requests.len() {
            if self.discard_closed_url_request(index) {
                continue;
            }
            if self.url_request_waits_for_earlier(index) || self.poll_url_request(ctx, index) {
                index += 1;
            }
        }
        self.state.url_tab.is_loading = !self.state.url_tab.pending_url_requests.is_empty();
    }

    fn url_request_waits_for_earlier(&self, index: usize) -> bool {
        let Some(request) = self.state.url_tab.pending_url_requests.get(index) else {
            return false;
        };
        self.state
            .url_tab
            .pending_url_requests
            .iter()
            .take(index)
            .any(|earlier| {
                match (&earlier.target_document, &request.target_document) {
                    /* WHY: 形式移行は同sourceの既存targetへ合流するため、その更新も直列化する。 */
                    (Some(first), Some(next)) => {
                        first == next || earlier.source_url == request.source_url
                    }
                    /* WHY: URL入力欄の連続OpenUrlは既存の最終表示・履歴順を維持する。 */
                    (None, None) => true,
                    /* WHY: target未確定の取得はredirectで既存文書と合流するため順序を保つ。 */
                    _ => true,
                }
            })
    }

    fn poll_url_request(&mut self, ctx: &egui::Context, index: usize) -> bool {
        let Some(request) = self.state.url_tab.pending_url_requests.get(index) else {
            return false;
        };
        match request.response_rx.try_recv() {
            Ok(response) => self.consume_url_response(index, response),
            Err(TryRecvError::Empty) => return self.poll_url_request_deadline(ctx, index),
            Err(TryRecvError::Disconnected) => self.consume_url_response(
                index,
                Err(HtmlSourceError::Network(
                    "URL request channel disconnected".to_owned(),
                )),
            ),
        }
        false
    }

    fn url_request_target_is_closed(&self, request: &PendingUrlRequest) -> bool {
        request.target_document.as_ref().is_some_and(|target| {
            !self
                .state
                .document
                .open_documents
                .iter()
                .any(|document| document.path == *target)
        })
    }

    fn discard_closed_url_request(&mut self, index: usize) -> bool {
        let closed = self
            .state
            .url_tab
            .pending_url_requests
            .get(index)
            .is_some_and(|request| self.url_request_target_is_closed(request));
        if closed {
            self.state.url_tab.pending_url_requests.remove(index);
        }
        closed
    }

    fn poll_url_request_deadline(&mut self, ctx: &egui::Context, index: usize) -> bool {
        let Some(request) = self.state.url_tab.pending_url_requests.get(index) else {
            return false;
        };
        let now = Instant::now();
        if now >= request.deadline {
            let error = HtmlSourceError::Timeout {
                url: request.source_url.clone(),
                seconds: URL_SOURCE_TIMEOUT.as_secs(),
            };
            self.consume_url_response(index, Err(error));
            return false;
        }
        ctx.request_repaint_after(
            request
                .deadline
                .saturating_duration_since(now)
                .min(URL_SOURCE_REPAINT_INTERVAL),
        );
        true
    }

    fn consume_url_response(
        &mut self,
        index: usize,
        response: Result<FetchedUrlSource, HtmlSourceError>,
    ) {
        let Some(request) = self.state.url_tab.pending_url_requests.remove(index) else {
            return;
        };
        /* WHY: queueはstateに残し、形式移行による後続targetの付け替えを受け取る。 */
        match response {
            Ok(source) => {
                self.apply_fetched_url_source(source, request.target_document);
                self.state.layout.status_message = None;
            }
            Err(error) => self.fail_requested_url_source(error, request),
        }
    }
}
