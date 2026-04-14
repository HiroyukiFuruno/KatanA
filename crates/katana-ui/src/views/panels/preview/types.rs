use crate::app_state::AppAction;
use crate::preview_pane::PreviewPane;

pub struct PreviewLogicOps;

pub(crate) struct PreviewContent<'a> {
    pub preview: &'a mut PreviewPane,
    pub document: Option<&'a katana_core::document::Document>,
    pub scroll: &'a mut crate::app_state::ScrollState,
    pub toc_visible: bool,
    pub show_toc: bool,
    pub show_export: bool,
    pub show_story: bool,
    pub action: &'a mut AppAction,
    pub scroll_sync: bool,
    pub search_query: Option<String>,
    pub doc_search_active_index: Option<usize>,
}

pub(crate) struct PreviewHeader<'a> {
    pub has_doc: bool,
    pub toc_visible: bool,
    pub show_toc: bool,
    pub show_export: bool,
    pub show_story: bool,
    pub action: &'a mut AppAction,
}
