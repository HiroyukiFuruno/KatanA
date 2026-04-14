pub(crate) struct TocPanel<'a> {
    pub preview: &'a mut crate::preview_pane::PreviewPane,
    pub state: &'a mut crate::app_state::AppState,
}

pub(crate) struct TocRenderContext<'a> {
    pub items: &'a [katana_core::markdown::outline::OutlineItem],
    pub active_index: usize,
    pub show_vertical_lines: bool,
    pub force_open: Option<bool>,
}
