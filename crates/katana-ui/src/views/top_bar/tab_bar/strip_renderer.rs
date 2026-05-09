use crate::app_state::AppAction;
use eframe::egui;

use super::group_header;
use super::items;
use super::tab_item;

pub(super) struct DrawItemRenderer<'a> {
    pub open_documents: &'a [katana_core::document::Document],
    pub active_doc_idx: Option<usize>,
    pub workspace_root: Option<&'a std::path::Path>,
    pub tab_groups: &'a [crate::state::document::TabGroup],
    pub recently_closed_tabs_empty: bool,
    pub inline_rename_group: &'a Option<String>,
    pub show_dirty_indicator: bool,
}

impl<'a> DrawItemRenderer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        item: items::DrawItem<'_>,
        should_scroll: bool,
        tab_action: &mut Option<AppAction>,
        close_idx: &mut Option<usize>,
        dragged_source: &mut Option<(usize, f32)>,
        tab_rects: &mut Vec<(usize, egui::Rect)>,
        ghost_info_acc: &mut Option<(egui::Rect, egui::Rangef)>,
    ) {
        match item {
            items::DrawItem::GroupHeader(g) => {
                group_header::GroupHeader {
                    g,
                    inline_rename_group: self.inline_rename_group.as_ref(),
                }
                .show(ui, tab_action);
            }
            items::DrawItem::Tab { idx, group } => {
                let row_top = ui.cursor().min.y;
                let res = tab_item::TabItem {
                    idx,
                    doc: &self.open_documents[idx],
                    is_active: self.active_doc_idx == Some(idx),
                    group,
                    ws_root: self.workspace_root,
                    tab_groups: self.tab_groups,
                    recently_closed_tabs_empty: self.recently_closed_tabs_empty,
                    should_scroll,
                    show_dirty_indicator: self.show_dirty_indicator,
                    row_top,
                }
                .show(ui, tab_action);
                if let Some(res) = res {
                    tab_rects.push((idx, res.rect));
                    if let Some(c) = res.close_idx {
                        *close_idx = Some(c);
                    }
                    if let Some(g) = res.ghost_info {
                        *ghost_info_acc = Some(g);
                    }
                    if let Some(s) = res.dragged_source {
                        *dragged_source = Some(s);
                    }
                }
            }
        }
    }
}
