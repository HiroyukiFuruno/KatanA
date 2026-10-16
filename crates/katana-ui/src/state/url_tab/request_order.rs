use std::collections::HashMap;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(crate) struct UrlRequestOrder {
    next: u64,
    completed: HashMap<PathBuf, NonZeroU64>,
    latest: Option<NonZeroU64>,
}

impl UrlRequestOrder {
    pub(crate) fn allocate(&mut self) -> Option<NonZeroU64> {
        self.next = self.next.checked_add(1)?;
        NonZeroU64::new(self.next)
    }

    pub(crate) fn is_superseded(&self, path: &Path, order: NonZeroU64) -> bool {
        self.completed
            .get(path)
            .is_some_and(|latest| *latest > order)
    }

    pub(crate) fn has_newer_completion(&self, order: NonZeroU64) -> bool {
        self.latest.is_some_and(|latest| latest > order)
    }

    pub(crate) fn complete(&mut self, path: Option<PathBuf>, order: NonZeroU64) {
        self.latest = Some(self.latest.map_or(order, |latest| latest.max(order)));
        self.complete_destination(path, order);
    }

    pub(crate) fn complete_destination(&mut self, path: Option<PathBuf>, order: NonZeroU64) {
        let Some(path) = path else {
            return;
        };
        self.completed
            .entry(path)
            .and_modify(|latest| *latest = (*latest).max(order))
            .or_insert(order);
    }

    pub(crate) fn retain_after(&mut self, earliest: Option<NonZeroU64>) {
        let Some(earliest) = earliest else {
            /* WHY: 空queueではpathだけでなくtable capacityも解放し、次の要求へ履歴を漏らさない。 */
            *self = Self::default();
            return;
        };
        let previous = self.completed.len();
        self.completed.retain(|_, order| *order > earliest);
        if self.completed.len() != previous {
            self.completed.shrink_to_fit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UrlRequestOrder;
    use std::path::PathBuf;

    #[test]
    fn newer_destination_completion_rejects_only_older_order() {
        let mut order = UrlRequestOrder::default();
        let first = order.allocate().unwrap();
        let second = order.allocate().unwrap();
        let path = PathBuf::from("destination.pdf");
        order.complete(Some(path.clone()), second);
        order.complete(Some(path.clone()), first);
        assert!(order.is_superseded(&path, first));
        assert!(!order.is_superseded(&path, second));
        assert!(!order.is_superseded(&PathBuf::from("other.pdf"), first));
    }

    #[test]
    fn pruning_keeps_only_records_that_can_supersede_pending_requests() {
        let mut order = UrlRequestOrder::default();
        let first = order.allocate().unwrap();
        let second = order.allocate().unwrap();
        order.complete(Some(PathBuf::from("old.pdf")), first);
        order.complete(Some(PathBuf::from("new.pdf")), second);
        order.retain_after(Some(first));
        assert_eq!(order.completed.len(), 1);
        assert!(order.completed.contains_key(&PathBuf::from("new.pdf")));
        order.retain_after(Some(second));
        assert!(order.completed.is_empty());
        assert_eq!(order.completed.capacity(), 0);
    }

    #[test]
    fn empty_queue_releases_history_and_restarts_counter() {
        let mut order = UrlRequestOrder::default();
        let first = order.allocate().unwrap();
        order.complete(Some(PathBuf::from("done.pdf")), first);
        order.retain_after(None);
        assert_eq!(order.completed.capacity(), 0);
        assert_eq!(order.allocate(), Some(first));
        assert!(!order.has_newer_completion(first));
    }

    #[test]
    fn completion_without_destination_still_orders_global_selection() {
        let mut order = UrlRequestOrder::default();
        let first = order.allocate().unwrap();
        let second = order.allocate().unwrap();
        order.complete(None, second);
        order.complete(None, first);
        assert!(order.has_newer_completion(first));
        assert!(!order.has_newer_completion(second));
        assert!(order.completed.is_empty());
        order.retain_after(Some(first));
        assert!(order.has_newer_completion(first));
    }

    #[test]
    fn background_completion_orders_content_without_claiming_foreground_selection() {
        let mut order = UrlRequestOrder::default();
        let first = order.allocate().unwrap();
        let second = order.allocate().unwrap();
        let path = PathBuf::from("background.pdf");
        order.complete_destination(Some(path.clone()), second);
        assert!(order.is_superseded(&path, first));
        assert!(!order.has_newer_completion(first));
    }

    #[test]
    fn counter_exhaustion_refuses_reuse_until_empty_queue_reset() {
        let mut order = UrlRequestOrder {
            next: u64::MAX,
            ..Default::default()
        };
        assert_eq!(order.allocate(), None);
        assert_eq!(order.allocate(), None);
        order.retain_after(None);
        assert_eq!(order.allocate().map(std::num::NonZeroU64::get), Some(1));
    }

    #[test]
    fn exhausted_request_is_reported_without_applying_ready_response() {
        let mut app = app();
        app.state.url_tab.request_order.next = u64::MAX;
        let (sender, response_rx) = std::sync::mpsc::channel();
        app.state
            .url_tab
            .pending_url_requests
            .push_back(super::super::PendingUrlRequest {
                order: None,
                selection_owner: None,
                response_rx,
                target_document: None,
                source_url: "https://example.test/page.html".to_owned(),
                deadline: std::time::Instant::now(),
            });
        sender
            .send(Err(super::super::HtmlSourceError::Network(
                "must not consume".to_owned(),
            )))
            .unwrap();
        app.poll_url_source(&egui::Context::default());
        assert!(app.state.url_tab.pending_url_requests.is_empty());
        assert!(!app.state.url_tab.is_loading);
        assert!(app.state.document.open_documents.is_empty());
        let error = app.state.url_tab.last_error.as_ref().unwrap().to_string();
        assert!(error.contains("counter exhausted"));
        assert_eq!(app.state.url_tab.request_order.next, 0);
    }

    fn app() -> crate::shell::KatanaApp {
        crate::shell::KatanaApp::new(crate::app_state::AppState::new(
            katana_core::ai::AiProviderRegistry::new(),
            katana_core::plugin::PluginRegistry::new(),
            katana_platform::SettingsService::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        ))
    }

    #[test]
    fn cancel_releases_completed_metadata_immediately() {
        let mut app = app();
        let order = app.state.url_tab.request_order.allocate().unwrap();
        app.state
            .url_tab
            .request_order
            .complete(Some(PathBuf::from("done.pdf")), order);
        app.state.url_tab.cancel_pending_url_requests();
        assert_eq!(app.state.url_tab.request_order.completed.capacity(), 0);
        assert_eq!(app.state.url_tab.request_order.next, 0);
    }
}
