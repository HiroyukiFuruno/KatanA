/* WHY: Refactored tab item orchestration to maintain a clean structure and manage architectural complexity via specialized sub-modules. */

use crate::app_state::AppAction;
use crate::state::document::{TabGroup, VirtualPathExt};
use crate::views::top_bar::tab_border::TabBorderOps;
use crate::views::top_bar::types::TopBarOps;
use eframe::egui;
use katana_core::document::Document;

mod close;
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
    pub show_dirty_indicator: bool,
    pub row_top: f32,
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
        let is_demo = self.doc.path.is_demo_path();
        let is_lint_review = crate::app::LintFixReviewPath::is_review_path(&self.doc.path);
        let original_filename = if is_lint_review {
            crate::i18n::I18nOps::get().diff_review.title.clone()
        } else {
            self.doc.file_name().unwrap_or("untitled").to_string()
        };
        let title = TopBarOps::tab_display_title(
            &original_filename,
            is_changelog,
            is_demo,
            self.doc.is_dirty,
            self.show_dirty_indicator,
        );
        let tooltip_path =
            crate::shell_logic::ShellLogicOps::relative_full_path(&self.doc.path, self.ws_root);

        let (tab_rect, close_resp, tab_interact) =
            self.render_parent_tab(ui, &title, is_changelog, is_demo);
        let tab_hovered = TabBorderOps::rect_contains_pointer(ui, tab_rect);
        self.set_close_visible(
            ui,
            Self::close_area_visible(self.doc.is_pinned, tab_hovered),
        );
        TabBorderOps::paint(ui, tab_rect, tab_hovered);
        self.draw_group_underline(ui, tab_rect);

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

        let ghost_info = self.handle_drag(ui, &tab_interact, tab_rect, &title, is_changelog);
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
            rect: tab_rect,
            close_idx: close_ret,
            ghost_info,
            dragged_source,
        })
    }

    fn render_parent_tab(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        is_changelog: bool,
        is_demo: bool,
    ) -> (egui::Rect, Option<egui::Response>, egui::Response) {
        let mut title_response = None;
        let mut close_response = None;
        let mut tab_interact = None;
        let parent_response = ui
            .push_id(format!("tab_{}", self.idx), |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                let close_visible = self.is_close_visible(ui);
                let tab_width = self.parent_tab_width(ui, title, is_changelog, close_visible);
                let row_height = self.parent_tab_height(ui);
                let (parent_rect, parent_response) =
                    ui.allocate_exact_size(egui::vec2(tab_width, row_height), egui::Sense::hover());
                let parent_rect = self.normalize_parent_tab_rect(parent_rect);
                let close_width = Self::close_width(ui);
                let close_rect = Self::close_rect(parent_rect, close_width);
                let title_rect = if close_visible {
                    egui::Rect::from_min_max(
                        parent_rect.min,
                        egui::pos2(
                            close_rect.left() - Self::title_close_gap(),
                            parent_rect.bottom(),
                        ),
                    )
                } else {
                    parent_rect
                };
                tab_interact = Some(ui.interact(
                    parent_rect,
                    egui::Id::new("tab_interact").with(self.idx),
                    if is_demo {
                        egui::Sense::click()
                    } else {
                        egui::Sense::click_and_drag()
                    },
                ));
                title_response =
                    Some(self.render_title_button_at(ui, title_rect, title, is_changelog));
                close_response = close_visible
                    .then(|| self.render_close_button_at(ui, close_rect))
                    .flatten();
                parent_response
            })
            .inner;
        let _ = title_response.expect("document tab title response");
        let tab_interact = tab_interact.expect("document tab interaction response");
        let tab_rect = Self::resolved_parent_tab_rect(
            parent_response.rect,
            close_response.as_ref().map(|response| response.rect),
        );
        let tab_rect = self.normalize_parent_tab_rect(tab_rect);
        (tab_rect, close_response, tab_interact)
    }

    pub(crate) fn resolved_parent_tab_rect(
        parent_rect: egui::Rect,
        _close_response_rect: Option<egui::Rect>,
    ) -> egui::Rect {
        parent_rect
    }

    fn normalize_parent_tab_rect(&self, rect: egui::Rect) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(rect.left(), self.row_top),
            egui::vec2(rect.width(), rect.height()),
        )
    }
}
