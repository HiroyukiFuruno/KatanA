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
    SEARCH_MODAL_WIDTH, STATUS_BAR_ICON_SPACING, STATUS_SUCCESS_GREEN,
    TOC_HEADING_VISIBILITY_THRESHOLD, TOC_INDENT_PER_LEVEL, TOC_PANEL_DEFAULT_WIDTH,
    TOC_PANEL_MARGIN, TreeRenderContext, WORKSPACE_SPINNER_INNER_MARGIN,
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
                // WHY: allow(horizontal_layout)
                crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
                    // WHY: allow(conditional_frame) — in popup/list context; future: standardize as atom
                    let file_name_selected = search.active_tab == crate::app_state::SearchTab::FileName;
                    if ui
                        .add(
                            egui::Button::selectable(
                                file_name_selected,
                                crate::i18n::I18nOps::get().search.tab_file_name.clone(),
                            )
                            .frame_when_inactive(true)
                            ,
                        )
                        .clicked()
                    {
                        search.active_tab = crate::app_state::SearchTab::FileName;
                        search.focus_requested = false;
                    }
                    // WHY: allow(conditional_frame) — in popup/list context; future: standardize as atom
                    let md_content_selected = search.active_tab == crate::app_state::SearchTab::MarkdownContent;
                    if ui
                        .add(
                            egui::Button::selectable(
                                md_content_selected,
                                crate::i18n::I18nOps::get().search.tab_markdown_content.clone(),
                            )
                            .frame_when_inactive(true)
                            ,
                        )
                        .clicked()
                    {
                        search.active_tab = crate::app_state::SearchTab::MarkdownContent;
                        search.focus_requested = false;
                    }
                }).show(ui);
                ui.separator();

                match search.active_tab {
                    crate::app_state::SearchTab::FileName => {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut search.query)
                                .hint_text(crate::i18n::I18nOps::get().search.query_hint.clone())
                                .desired_width(f32::INFINITY),
                        );
                        if !search.focus_requested {
                            response.request_focus();
                            search.focus_requested = true;
                        }

                        let mut include_regexes = Vec::new();
                        let mut include_valid = true;
                        if !search.include_pattern.is_empty() {
                            for pat in search.include_pattern.split(',') {
                                let pat = pat.trim();
                                if !pat.is_empty() {
                                    match regex::Regex::new(pat) {
                                        Ok(re) => include_regexes.push(re),
                                        Err(_) => include_valid = false,
                                    }
                                }
                            }
                        }

                        let mut exclude_regexes = Vec::new();
                        let mut exclude_valid = true;
                        if !search.exclude_pattern.is_empty() {
                            for pat in search.exclude_pattern.split(',') {
                                let pat = pat.trim();
                                if !pat.is_empty() {
                                    match regex::Regex::new(pat) {
                                        Ok(re) => exclude_regexes.push(re),
                                        Err(_) => exclude_valid = false,
                                    }
                                }
                            }
                        }

                        let include_color = if include_valid {
                            ui.visuals().text_color()
                        } else {
                            ui.ctx()
                                .data(|d| {
                                    d.get_temp::<katana_platform::theme::ThemeColors>(
                                        egui::Id::new("katana_theme_colors"),
                                    )
                                })
                                .map_or(crate::theme_bridge::WHITE, |tc| {
                                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                                        tc.system.error_text,
                                    )
                                })
                        };

                        let exclude_color = if exclude_valid {
                            ui.visuals().text_color()
                        } else {
                            ui.ctx()
                                .data(|d| {
                                    d.get_temp::<katana_platform::theme::ThemeColors>(
                                        egui::Id::new("katana_theme_colors"),
                                    )
                                })
                                .map_or(crate::theme_bridge::WHITE, |tc| {
                                    crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                                        tc.system.error_text,
                                    )
                                })
                        };

                        ui.add(
                            egui::TextEdit::singleline(&mut search.include_pattern)
                                .hint_text(crate::i18n::I18nOps::get().search.include_pattern_hint.clone())
                                .text_color(include_color)
                                .desired_width(f32::INFINITY),
                        );

                        ui.add(
                            egui::TextEdit::singleline(&mut search.exclude_pattern)
                                .hint_text(crate::i18n::I18nOps::get().search.exclude_pattern_hint.clone())
                                .text_color(exclude_color)
                                .desired_width(f32::INFINITY),
                        );

                        let current_params = (
                            search.query.clone(),
                            search.include_pattern.clone(),
                            search.exclude_pattern.clone(),
                        );

                        if search.last_params.as_ref() != Some(&current_params) {
                            search.last_params = Some(current_params);

                            let query = search.query.to_lowercase();
                            if query.is_empty()
                                && include_regexes.is_empty()
                                && exclude_regexes.is_empty()
                            {
                                search.results.clear();
                            } else if let Some(ws) = workspace {
                                let mut results = Vec::new();
                                crate::shell_logic::ShellLogicOps::collect_matches(
                                    &ws.tree,
                                    &query,
                                    &include_regexes,
                                    &exclude_regexes,
                                    &ws.root,
                                    &mut results,
                                );
                                search.results = results;
                            }
                        }

                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                if search.results.is_empty() && !search.query.is_empty() {
                                    ui.label(crate::i18n::I18nOps::get().search.no_results.clone());
                                } else {
                                    let ws_root = workspace.map(|ws| ws.root.clone());
                                    for path in &search.results {
                                        let rel =
                                            crate::shell_logic::ShellLogicOps::relative_full_path(
                                                path,
                                                ws_root.as_deref(),
                                            );
                                        // WHY: in popup/list context; future: standardize as atom
                                        if ui.add(egui::Button::selectable(false, rel).frame_when_inactive(true)).clicked()
                                            && path.exists()
                                        {
                                            *action = AppAction::SelectDocument(path.to_path_buf());
                                        }
                                    }
                                }
                            });
                    }
                    crate::app_state::SearchTab::MarkdownContent => {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut search.md_query)
                                .hint_text(crate::i18n::I18nOps::get().search.md_query_hint.clone())
                                .desired_width(f32::INFINITY),
                        );
                        if !search.focus_requested {
                            response.request_focus();
                            search.focus_requested = true;
                        }

                        if (response.changed()
                            || response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            && search.md_last_query.as_deref() != Some(search.md_query.as_str())
                        {
                            search.md_last_query = Some(search.md_query.clone());
                            if search.md_query.is_empty() {
                                search.md_results.clear();
                            } else if let Some(ws) = workspace {
                                search.md_results = katana_core::search::WorkspaceSearchOps::search_workspace(
                                    ws,
                                    &search.md_query,
                                    MD_SEARCH_LIMIT,
                                );
                                search.md_history.push_term(
                                    search.md_query.clone(),
                                    MD_SEARCH_HISTORY_LIMIT,
                                );
                            }
                        }

                        // WHY: Recent Terms
                        if search.md_query.is_empty() && !search.md_history.recent_terms.is_empty()
                        {
                            ui.separator();
                            // WHY: allow(horizontal_layout)
                            crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
                                ui.label(
                                    egui::RichText::new(
                                        crate::i18n::I18nOps::get().search.recent_searches.clone(),
                                    )
                                    .strong(),
                                );
                                if ui
                                    .button(crate::i18n::I18nOps::get().search.clear_history.clone())
                                    .clicked()
                                {
                                    search.md_history.clear();
                                }
                            }).show(ui);
                            for term in search.md_history.recent_terms.clone() {
                                if ui.link(&term).clicked() {
                                    search.md_query = term.clone();
                                    search.md_last_query = Some(term.clone());
                                    if let Some(ws) = workspace {
                                        search.md_results = katana_core::search::WorkspaceSearchOps::search_workspace(
                                            ws,
                                            &term,
                                            MD_SEARCH_LIMIT,
                                        );
                                        search.md_history.push_term(term, MD_SEARCH_HISTORY_LIMIT);
                                    }
                                }
                            }
                        }

                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                if search.md_results.is_empty() && !search.md_query.is_empty() {
                                    ui.label(crate::i18n::I18nOps::get().search.no_results.clone());
                                } else {
                                    let ws_root = workspace.map(|ws| ws.root.clone());
                                    for result in &search.md_results {
                                        let rel =
                                            crate::shell_logic::ShellLogicOps::relative_full_path(
                                                &result.file_path,
                                                ws_root.as_deref(),
                                            );
                                        ui.group(|ui| {
                                            // WHY: allow(horizontal_layout)
                                            crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
                                                ui.label(egui::RichText::new(&rel).strong());
                                                let ln_text = format!(
                                                    "{}{}",
                                                    crate::i18n::I18nOps::get().search.ln_prefix,
                                                    result.line_number + 1
                                                );
                                                ui.label(egui::RichText::new(ln_text).weak());
                                            }).show(ui);
                                            let mut job = egui::text::LayoutJob::default();
                                            let start = result.start_col;
                                            let end = result.end_col;
                                            if start <= end && end <= result.snippet.len() {
                                                let pre = &result.snippet[..start];
                                                let matched = &result.snippet[start..end];
                                                let post = &result.snippet[end..];

                                                job.append(pre, 0.0, egui::TextFormat::default());
                                                job.append(
                                                    matched,
                                                    0.0,
                                                    egui::TextFormat {
                                                        color: ui.visuals().strong_text_color(),
                                                        background: ui.visuals().selection.bg_fill,
                                                        ..Default::default()
                                                    },
                                                );
                                                job.append(post, 0.0, egui::TextFormat::default());
                                            } else {
                                                job.append(
                                                    &result.snippet,
                                                    0.0,
                                                    egui::TextFormat::default(),
                                                );
                                            }

                                            // WHY: Make it selectable and render the layout job!
                                            // WHY: in popup/list context; future: standardize as atom
                                            let response = ui.add(egui::Button::selectable(false, job).frame_when_inactive(true));

                                            if response.clicked() {
                                                *action = AppAction::SelectDocumentAndJump {
                                                    path: result.file_path.clone(),
                                                    line: result.line_number,
                                                    byte_range: result.start_col..result.end_col,
                                                };
                                            }
                                        });
                                    }
                                }
                            });
                    }
                }
            });

        *self.is_open = local_is_open;
    }
}
