use crate::app_state::AppAction;
use crate::preview_pane::PreviewPane;

pub struct PreviewLogicOps;

pub(crate) struct PreviewContent<'a> {
    pub preview: &'a mut PreviewPane,
    pub document: Option<&'a katana_core::document::Document>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub action: &'a mut AppAction,
    pub scroll_sync: bool,
    pub search_query: Option<String>,
    pub doc_search_active_index: Option<usize>,
}
