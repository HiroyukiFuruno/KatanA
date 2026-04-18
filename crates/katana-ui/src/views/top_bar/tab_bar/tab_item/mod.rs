/* WHY: Refactored tab item orchestration to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

use crate::app_state::AppAction;
use crate::state::document::TabGroup;
use crate::views::top_bar::types::TopBarOps;
use eframe::egui;
use katana_core::document::Document;

mod drag;
mod render;

const TAB_MAX_WIDTH: f32 = 200.0;

pub(crate) struct TabItemResult {
    pub rect: egui::Rect,
    pub close_idx: Option<usize>,
    pub ghost_info: Option<(egui::Rect, egui::Rangef)>,
    pub dragged_source: Option<(usize, f32)>,
}

pub(crate) struct TabItem<'a> {
    pub idx: usize,
    pub doc: &'a Document,
    pub is_active: bool,
    pub group: Option<&'a TabGroup>,
    pub ws_root: Option<&'a std::path::Path>,
    pub tab_groups: &'a [TabGroup],
    pub recently_closed_tabs_empty: bool,
    pub should_scroll: bool,
}

impl<'a> TabItem<'a> {
    pub fn show(
        self,
        ui: &mut egui::Ui,
        tab_action: &mut Option<AppAction>,
    ) -> Option<TabItemResult> {
        if self.group.is_some_and(|g| g.collapsed) {
            return None;
        }
        let is_changelog = self
            .doc
            .path
            .to_string_lossy()
            .starts_with("Katana://ChangeLog");
        let title = TopBarOps::tab_display_title(
            self.doc.file_name().unwrap_or("untitled"),
            is_changelog,
            self.doc.is_dirty,
            self.doc.is_pinned,
        );
        let tooltip_path =
            crate::shell_logic::ShellLogicOps::relative_full_path(&self.doc.path, self.ws_root);

        let (title_resp, close_resp) = ui
            .push_id(format!("tab_{}", self.idx), |ui| {
                ui.set_max_width(TAB_MAX_WIDTH);
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                let t = self.render_title_button(ui, &title, is_changelog);
                let c = self.render_close_button(ui);
                (t, c)
            })
            .inner;

        let full_rect = close_resp
            .as_ref()
            .map_or(title_resp.rect, |c| title_resp.rect.union(c.rect));
        self.draw_group_underline(ui, full_rect);

        use crate::state::document::VirtualPathExt;
        let is_demo = self.doc.path.is_demo_path();
        let tab_interact = ui.interact(
            title_resp.rect,
            egui::Id::new("tab_interact").with(self.idx),
            if is_demo {
                egui::Sense::click()
            } else {
                egui::Sense::click_and_drag()
            },
        );
        let mut clicked_tab = tab_interact.clicked();
        let mut close_ret = None;
        if let Some(c) = &close_resp
            && c.clicked()
        {
            if self.doc.is_pinned {
                *tab_action = Some(AppAction::TogglePinDocument(self.idx));
            } else {
                close_ret = Some(self.idx);
            }
            clicked_tab = false;
        }

        let ghost_info = self.handle_drag(ui, &tab_interact, full_rect, &title, is_changelog);
        if self.is_active && self.should_scroll {
            tab_interact.scroll_to_me(Some(egui::Align::Center));
        }
        let dragged_source = self.check_drag_stopped(ui, &tab_interact);

        let tab_groups = self.tab_groups;
        let recently_closed_tabs_empty = self.recently_closed_tabs_empty;
        let idx = self.idx;
        let doc = self.doc;
        tab_interact
            .on_hover_text(&tooltip_path)
            .context_menu(|ui| {
                super::tab_context_menu::TabContextMenu {
                    idx,
                    doc,
                    tab_groups,
                    recently_closed_tabs_empty,
                }
                .show(ui, tab_action);
            });

        if clicked_tab && !self.is_active {
            *tab_action = Some(AppAction::SelectDocument(self.doc.path.clone()));
        }
        Some(TabItemResult {
            rect: full_rect,
            close_idx: close_ret,
            ghost_info,
            dragged_source,
        })
    }
}
