use crate::shell::KatanaApp;
use crate::state::{FetchedUrlSource, HtmlSourceError, PendingUrlRequest};
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
struct UrlSelection {
    document: Option<PathBuf>,
    migrated_document: Option<PathBuf>,
    url_document: Option<PathBuf>,
    input: String,
    scroll_to_active_tab: bool,
    status_message: Option<(String, crate::app_state::StatusType)>,
    last_error: Option<HtmlSourceError>,
}

impl UrlSelection {
    fn capture(app: &KatanaApp, request: &PendingUrlRequest, destination: Option<PathBuf>) -> Self {
        let tab = &app.state.url_tab;
        let document = app.state.active_path();
        let migrated_document = document
            .as_ref()
            .filter(|path| Some(*path) == request.target_document.as_ref())
            .and(destination);
        Self {
            document,
            migrated_document,
            url_document: tab
                .active_tab
                .and_then(|index| tab.tabs.get(index))
                .map(|tab| tab.document_path.clone()),
            input: tab.input.clone(),
            scroll_to_active_tab: app.state.document.scroll_to_active_tab,
            status_message: app.state.layout.status_message.clone(),
            last_error: tab.last_error.clone(),
        }
    }

    fn restore(self, app: &mut KatanaApp) {
        /* WHY: 元の文書が移行で消えた場合だけ、その実移行先へ選択を追従する。 */
        app.state.document.active_doc_idx = self
            .document
            .as_ref()
            .and_then(|path| document_index(app, path))
            .or_else(|| {
                self.migrated_document
                    .as_ref()
                    .and_then(|path| document_index(app, path))
            });
        app.state.url_tab.active_tab = self.url_document.as_ref().and_then(|path| {
            app.state
                .url_tab
                .tabs
                .iter()
                .position(|tab| tab.document_path == *path)
        });
        app.state.url_tab.input = self.input;
        app.state.document.scroll_to_active_tab = self.scroll_to_active_tab;
        app.state.layout.status_message = self.status_message;
        app.state.url_tab.last_error = self.last_error;
    }
}

impl KatanaApp {
    pub(super) fn select_cached_url_document(&mut self, path: &std::path::Path) -> bool {
        let Some(index) = document_index(self, path) else {
            return false;
        };
        let tab = &self.state.url_tab;
        let html_index = tab.tabs.iter().position(|tab| tab.document_path == path);
        let source_url = tab
            .source_for_document(path)
            .map(|source| source.source_url.as_str())
            .or_else(|| tab.document_source_url_for_document(path));
        let Some(source_url) = source_url.map(str::to_owned) else {
            return false;
        };
        self.state.document.active_doc_idx = Some(index);
        self.state.document.scroll_to_active_tab = true;
        self.state.url_tab.input = source_url;
        self.state.url_tab.last_error = None;
        self.state.layout.status_message = None;
        if let Some(index) = html_index {
            self.state.url_tab.active_tab = Some(index);
        } else {
            self.state
                .set_active_view_mode(crate::state::document::ViewMode::PreviewOnly);
        }
        true
    }

    pub(super) fn apply_selected_url_response(
        &mut self,
        response: Result<FetchedUrlSource, HtmlSourceError>,
        request: PendingUrlRequest,
        order: NonZeroU64,
    ) {
        let selection = self.url_response_is_background(&request, order).then(|| {
            let destination = Self::url_response_destination(&response, &request);
            UrlSelection::capture(self, &request, destination)
        });
        self.apply_ordered_url_response(response, request);
        if let Some(selection) = selection {
            selection.restore(self);
        }
    }

    pub(super) fn url_response_is_background(
        &self,
        request: &PendingUrlRequest,
        order: NonZeroU64,
    ) -> bool {
        self.state.url_tab.request_order.has_newer_completion(order)
            || request
                .selection_owner
                .as_ref()
                .is_some_and(|owner| !Arc::ptr_eq(owner, &self.state.url_tab.selection_owner))
    }
}

fn document_index(app: &KatanaApp, path: &std::path::Path) -> Option<usize> {
    app.state
        .document
        .open_documents
        .iter()
        .position(|document| document.path == path)
}
