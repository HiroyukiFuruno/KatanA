pub mod drag;
pub mod group_header;
pub mod group_header_popup;
pub mod items;
pub mod nav;
pub mod strip_renderer;
pub mod tab_context_menu;
pub mod tab_ghost;
pub mod tab_item;

use crate::app_state::AppAction;
use crate::shell::{TAB_NAV_BUTTONS_AREA_WIDTH, TAB_TOOLTIP_SHOW_DELAY_SECS};
use crate::shell_ui::LIGHT_MODE_ICON_BG;
use eframe::egui;

pub(crate) struct TabBar<'a> {
    pub workspace_root: Option<&'a std::path::Path>,
    pub open_documents: &'a [katana_core::document::Document],
    pub active_doc_idx: Option<usize>,
    pub recently_closed_tabs: &'a std::collections::VecDeque<(std::path::PathBuf, bool)>,
    pub tab_groups: &'a [crate::state::document::TabGroup],
    pub inline_rename_group: &'a Option<String>,
}

impl<'a> TabBar<'a> {
    pub fn new(
        workspace_root: Option<&'a std::path::Path>,
        open_documents: &'a [katana_core::document::Document],
        active_doc_idx: Option<usize>,
        recently_closed_tabs: &'a std::collections::VecDeque<(std::path::PathBuf, bool)>,
        tab_groups: &'a [crate::state::document::TabGroup],
        inline_rename_group: &'a Option<String>,
    ) -> Self {
        Self {
            workspace_root,
            open_documents,
            active_doc_idx,
            recently_closed_tabs,
            tab_groups,
            inline_rename_group,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> Option<AppAction> {
        let mut close_idx: Option<usize> = None;
        let mut tab_action: Option<AppAction> = None;
        let mut dragged_source: Option<(usize, f32)> = None;
        let mut tab_rects: Vec<(usize, egui::Rect)> = Vec::new();

        ui.style_mut().interaction.tooltip_delay = TAB_TOOLTIP_SHOW_DELAY_SECS;
        let icon_bg = self.resolve_icon_bg(ui);

        let available_panel_width = ui.available_width();

        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                let should_scroll = self.check_scroll_request(ui);
                let scroll_width = available_panel_width - TAB_NAV_BUTTONS_AREA_WIDTH;
                egui::ScrollArea::horizontal()
                    .max_width(scroll_width)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .id_salt("tab_scroll")
                    .show(ui, |ui| {
                        self.render_tab_strip(
                            ui,
                            should_scroll,
                            &mut tab_action,
                            &mut close_idx,
                            &mut dragged_source,
                            &mut tab_rects,
                        );
                    });
                if should_scroll {
                    ui.memory_mut(|mem| {
                        mem.data
                            .remove_temp::<bool>(egui::Id::new("scroll_tab_req"));
                    });
                }
                ui.separator();
                nav::TabNavButtons {
                    icon_bg,
                    doc_count: self.open_documents.len(),
                    active_doc_idx: self.active_doc_idx,
                    open_documents: self.open_documents,
                }
                .show(ui, &mut tab_action);
            })
            .show(ui);

        if let Some((src_idx, ghost_x)) = dragged_source {
            let handler = drag::TabDragHandler {
                src_idx,
                ghost_center_x: ghost_x,
                tab_rects: &tab_rects,
                open_documents: self.open_documents,
                tab_groups: self.tab_groups,
            };
            if let Some(a) = handler.resolve() {
                tab_action = Some(a);
            }
        }
        if let Some(idx) = close_idx {
            tab_action = Some(AppAction::CloseDocument(idx));
        }
        tab_action
    }

    fn resolve_icon_bg(&self, ui: &egui::Ui) -> egui::Color32 {
        if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::from_gray(LIGHT_MODE_ICON_BG)
        }
    }

    fn check_scroll_request(&self, ui: &mut egui::Ui) -> bool {
        ui.memory_mut(|mem| {
            mem.data
                .get_temp::<bool>(egui::Id::new("scroll_tab_req"))
                .unwrap_or(false)
        })
    }

    fn render_tab_strip(
        &self,
        ui: &mut egui::Ui,
        should_scroll: bool,
        tab_action: &mut Option<AppAction>,
        close_idx: &mut Option<usize>,
        dragged_source: &mut Option<(usize, f32)>,
        tab_rects: &mut Vec<(usize, egui::Rect)>,
    ) {
        let renderer = strip_renderer::DrawItemRenderer {
            open_documents: self.open_documents,
            active_doc_idx: self.active_doc_idx,
            workspace_root: self.workspace_root,
            tab_groups: self.tab_groups,
            recently_closed_tabs_empty: self.recently_closed_tabs.is_empty(),
            inline_rename_group: self.inline_rename_group,
        };
        let draw_items = items::DrawItemCollector {
            open_documents: self.open_documents,
            tab_groups: self.tab_groups,
        }
        .collect();

        let mut ghost_info_acc = None;
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                for item in draw_items {
                    renderer.render(
                        ui,
                        item,
                        should_scroll,
                        tab_action,
                        close_idx,
                        dragged_source,
                        tab_rects,
                        &mut ghost_info_acc,
                    );
                }
                strip_renderer::DropIndicator {
                    tab_rects,
                    ghost_info: ghost_info_acc,
                }
                .render(ui);
            })
            .show(ui);
    }
}
