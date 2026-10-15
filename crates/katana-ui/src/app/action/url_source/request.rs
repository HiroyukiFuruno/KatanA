use super::URL_SOURCE_TIMEOUT;
use crate::app::url_source::{UrlResponseCollector, ValidatedHttpUrl};
use crate::shell::KatanaApp;
use crate::state::{FetchedUrlSource, HtmlSourceError, PendingUrlRequest};
use std::path::PathBuf;
use std::sync::{Arc, mpsc};

enum UrlRequestIntent {
    Foreground,
    Background,
}

impl KatanaApp {
    pub(in crate::app::action) fn fetch_html_url(
        &mut self,
        ctx: &egui::Context,
        url: ValidatedHttpUrl,
        target: Option<PathBuf>,
    ) {
        self.fetch_url_source(ctx, url, target, UrlRequestIntent::Foreground);
    }

    pub(in crate::app::action) fn fetch_html_url_for_refresh(
        &mut self,
        ctx: &egui::Context,
        url: ValidatedHttpUrl,
        target: Option<PathBuf>,
        is_manual: bool,
    ) {
        let intent = if is_manual {
            UrlRequestIntent::Foreground
        } else {
            UrlRequestIntent::Background
        };
        self.fetch_url_source(ctx, url, target, intent);
    }

    fn fetch_url_source(
        &mut self,
        ctx: &egui::Context,
        url: ValidatedHttpUrl,
        target: Option<PathBuf>,
        intent: UrlRequestIntent,
    ) {
        let (sender, receiver) = mpsc::channel();
        self.enqueue_url_source(&url, target, receiver, intent);
        start_url_fetch(ctx, url, sender);
    }

    fn enqueue_url_source(
        &mut self,
        url: &ValidatedHttpUrl,
        target_document: Option<PathBuf>,
        response_rx: mpsc::Receiver<Result<FetchedUrlSource, HtmlSourceError>>,
        intent: UrlRequestIntent,
    ) {
        let selection_owner = self.url_request_selection_owner(url, intent);
        self.state
            .url_tab
            .pending_url_requests
            .push_back(PendingUrlRequest {
                response_rx,
                target_document,
                source_url: url.as_str().to_owned(),
                order: None,
                selection_owner: Some(selection_owner),
                deadline: std::time::Instant::now() + URL_SOURCE_TIMEOUT,
            });
        self.state.url_tab.is_loading = true;
    }

    fn url_request_selection_owner(
        &mut self,
        url: &ValidatedHttpUrl,
        intent: UrlRequestIntent,
    ) -> Arc<()> {
        let owner = Arc::new(());
        if matches!(intent, UrlRequestIntent::Foreground) {
            /* WHY: 完了前の新操作を守り、自動再取得には選択権を渡さない。 */
            self.state.url_tab.selection_owner = owner.clone();
            self.state.url_tab.input = url.as_str().to_owned();
            self.state.url_tab.last_error = None;
        }
        owner
    }
}

fn start_url_fetch(
    ctx: &egui::Context,
    url: ValidatedHttpUrl,
    sender: mpsc::Sender<Result<FetchedUrlSource, HtmlSourceError>>,
) {
    let collector = std::cell::RefCell::new(UrlResponseCollector::new(url.clone()));
    let repaint = ctx.clone();
    ehttp::streaming::fetch(
        ehttp::Request::get(url.as_str()).with_timeout(Some(URL_SOURCE_TIMEOUT)),
        move |part| {
            let Some(result) = collector.borrow_mut().accept(part) else {
                return std::ops::ControlFlow::Continue(());
            };
            let _ = sender.send(result);
            repaint.request_repaint_after(std::time::Duration::ZERO);
            std::ops::ControlFlow::Break(())
        },
    );
}
