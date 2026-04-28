use crate::app_state::AppAction;
use crate::shell::TAB_DROP_ANIMATION_TIME;
use crate::shell::TAB_DROP_INDICATOR_WIDTH;
use crate::shell::TAB_INTER_ITEM_SPACING;
use crate::views::top_bar::types::TopBarOps;
use eframe::egui;

use super::group_header;
use super::items;
use super::tab_item;

pub(super) struct DropIndicator<'a> {
    pub tab_rects: &'a [(usize, egui::Rect)],
    pub ghost_info: Option<(egui::Rect, egui::Rangef)>,
}

impl<'a> DropIndicator<'a> {
    pub fn render(self, ui: &mut egui::Ui) {
        let Some((ghost_rect, y_range)) = self.ghost_info else {
            return;
        };
        let drop_points = TopBarOps::compute_drop_points(self.tab_rects);
        let best_x = drop_points
            .iter()
            .min_by(|(_, a), (_, b)| {
                let da = (ghost_rect.center().x - a).abs();
                let db = (ghost_rect.center().x - b).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(_, x)| *x);
        let Some(target_x) = best_x else { return };
        let animated_x = ui.ctx().animate_value_with_time(
            egui::Id::new("tab_drop_indicator"),
            target_x,
            TAB_DROP_ANIMATION_TIME,
        );
        ui.painter().vline(
            animated_x,
            y_range,
            egui::Stroke::new(TAB_DROP_INDICATOR_WIDTH, ui.visuals().selection.bg_fill),
        );
    }
}

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
                    ui.add_space(TAB_INTER_ITEM_SPACING);
                }
            }
        }
    }
}
