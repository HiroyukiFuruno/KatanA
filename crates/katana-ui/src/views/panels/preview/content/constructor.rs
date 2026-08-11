use super::PreviewContent;
use crate::app_state::{AppAction, ScrollState};
use crate::preview_pane::PreviewPane;

impl<'a> PreviewContent<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        preview: &'a mut PreviewPane,
        document: Option<&'a katana_core::document::Document>,
        scroll: &'a mut ScrollState,
        action: &'a mut AppAction,
        scroll_sync: bool,
        search_query: Option<String>,
        doc_search_active_index: Option<usize>,
    ) -> Self {
        Self {
            preview,
            document,
            scroll,
            action,
            scroll_sync,
            search_query,
            doc_search_active_index,
        }
    }
}
