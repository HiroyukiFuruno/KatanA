use crate::Icon;
use crate::app_state::{AppAction, AppState, ScrollSource, ViewMode};
use crate::i18n;
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell::{
    ACTIVE_FILE_HIGHLIGHT_ROUNDING, EDITOR_INITIAL_VISIBLE_ROWS, FILE_TREE_PANEL_DEFAULT_WIDTH,
    FILE_TREE_PANEL_MIN_WIDTH, NO_WORKSPACE_BOTTOM_SPACING, RECENT_WORKSPACES_ITEM_SPACING,
    RECENT_WORKSPACES_SPACING, SCROLL_SYNC_DEAD_ZONE, TAB_DROP_ANIMATION_TIME,
    TAB_DROP_INDICATOR_WIDTH, TAB_INTER_ITEM_SPACING, TAB_NAV_BUTTONS_AREA_WIDTH,
    TAB_TOOLTIP_SHOW_DELAY_SECS, TREE_LABEL_HOFFSET, TREE_ROW_HEIGHT,
};
use crate::shell_ui::{
    LIGHT_MODE_ICON_ACTIVE_BG, LIGHT_MODE_ICON_BG, PREVIEW_CONTENT_PADDING, SEARCH_MODAL_HEIGHT,
    SEARCH_MODAL_WIDTH, STATUS_BAR_ICON_SPACING, STATUS_SUCCESS_GREEN, TOC_INDENT_PER_LEVEL,
    TOC_PANEL_DEFAULT_WIDTH, TOC_PANEL_MARGIN, TreeRenderContext, WORKSPACE_SPINNER_INNER_MARGIN,
    WORKSPACE_SPINNER_OUTER_MARGIN, WORKSPACE_SPINNER_TEXT_MARGIN,
};
use crate::theme_bridge;
use eframe::egui;
use katana_core::workspace::{TreeEntry, Workspace};
use std::path::{Path, PathBuf};

const MD_SEARCH_LIMIT: usize = 50;
const MD_SEARCH_HISTORY_LIMIT: usize = 10;

pub(crate) struct SearchModal<'a> {
    pub search: &'a mut crate::app_state::SearchState,
    pub workspace: Option<&'a katana_core::workspace::Workspace>,
    pub is_open: &'a mut bool,
    pub action: &'a mut AppAction,
}

impl<'a> SearchModal<'a> {
    pub fn new(
        search: &'a mut crate::app_state::SearchState,
        workspace: Option<&'a katana_core::workspace::Workspace>,
        is_open: &'a mut bool,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            search,
            workspace,
            is_open,
            action,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        let search = self.search;
        let workspace = self.workspace;
        let action = self.action;
        let mut local_is_open = *self.is_open;
        egui::Window::new(crate::i18n::I18nOps::get().search.modal_title.clone())
            .open(&mut local_is_open)
            .collapsible(false)
            .resizable(true)
            .default_size(egui::vec2(SEARCH_MODAL_WIDTH, SEARCH_MODAL_HEIGHT))
            .show(ctx, |ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        let file_name_selected =
                            search.active_tab == crate::app_state::SearchTab::FileName;
                        if ui
                            .add(
                                egui::Button::selectable(
                                    file_name_selected,
                                    crate::i18n::I18nOps::get().search.tab_file_name.clone(),
                                )
                                .frame_when_inactive(true),
                            )
                            .clicked()
                        {
                            search.active_tab = crate::app_state::SearchTab::FileName;
                            search.focus_requested = false;
                        }
                        let md_content_selected =
                            search.active_tab == crate::app_state::SearchTab::MarkdownContent;
                        if ui
                            .add(
                                egui::Button::selectable(
                                    md_content_selected,
                                    crate::i18n::I18nOps::get()
                                        .search
                                        .tab_markdown_content
                                        .clone(),
                                )
                                .frame_when_inactive(true),
                            )
                            .clicked()
                        {
                            search.active_tab = crate::app_state::SearchTab::MarkdownContent;
                            search.focus_requested = false;
                        }
                    })
                    .show(ui);
                ui.separator();

                match search.active_tab {
                    crate::app_state::SearchTab::FileName => {
                        super::search_tabs::show_filename_tab(ui, search, workspace, action);
                    }
                    crate::app_state::SearchTab::MarkdownContent => {
                        super::search_tabs::show_md_tab(ui, search, workspace, action);
                    }
                }
            });

        *self.is_open = local_is_open;
    }
}
