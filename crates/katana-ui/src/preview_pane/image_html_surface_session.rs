use super::{BrowserSessionAdapter, HtmlBrowserSurface, HtmlBrowserViewport};
use katana_document_viewer::browser_session::BrowserSessionRequest;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ADAPTER_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

impl HtmlBrowserSurface {
    pub(super) fn take_start_request(
        &mut self,
        viewport: HtmlBrowserViewport,
    ) -> Option<BrowserSessionRequest> {
        let source = self.pending_source.take()?;
        self.viewport = Some(viewport);
        Some(BrowserSessionRequest::new(source, viewport))
    }

    pub(super) fn start_pending_session(&mut self, viewport: HtmlBrowserViewport) -> bool {
        let Some(request) = self.take_start_request(viewport) else {
            return false;
        };
        let adapter = BrowserSessionAdapter::start(request);
        let Some(adapter_instance_id) = NEXT_ADAPTER_INSTANCE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .ok()
        else {
            drop(adapter);
            self.record_error("HTML browser adapter identity overflow".to_owned());
            return false;
        };
        self.adapter = Some(adapter);
        self.katana_adapter_instance_id = Some(adapter_instance_id);
        self.pending_renderer_session = true;
        self.await_frame();
        true
    }

    pub(super) fn confirm_renderer_session(&mut self) -> bool {
        if !self.pending_renderer_session {
            return true;
        }
        let Some(next_epoch) = self
            .katana_renderer_session_epoch
            .unwrap_or_default()
            .checked_add(1)
        else {
            self.record_error("HTML renderer session identity overflow".to_owned());
            return false;
        };
        self.katana_renderer_session_epoch = Some(next_epoch);
        self.pending_renderer_session = false;
        true
    }
}
