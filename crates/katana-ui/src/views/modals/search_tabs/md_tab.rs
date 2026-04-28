/* WHY: Specialized logic for Markdown search tab rendering to keep the codebase modular and organized. */

use crate::app_state::AppAction;
use eframe::egui;

const MD_SEARCH_LIMIT: usize = 50;
const MD_SEARCH_HISTORY_LIMIT: usize = 10;

pub struct MdTabOps;

impl MdTabOps {
    pub(crate) fn show_md_tab(
        ui: &mut egui::Ui,
        search: &mut crate::app_state::SearchState,
        workspace: Option<&katana_core::workspace::Workspace>,
        action: &mut AppAction,
    ) {
        let response = crate::widgets::SearchBar::new(&mut search.md_search)
            .hint_text(crate::i18n::I18nOps::get().search.md_query_hint.clone())
            .id_source("search_tabs_md_search_bar")
            .show(ui);
        let history_changed =
            super::history::SearchHistoryUiOps::apply_keyboard_navigation(ui, &response, search);
        if response.changed() && !history_changed {
            super::history::SearchHistoryUiOps::reset_navigation(search);
        }
        if !search.focus_requested {
            response.request_focus();
            search.focus_requested = true;
        }

        if (history_changed
            || response.changed()
            || response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            && search.md_last_params.as_ref() != Some(&search.md_search)
        {
            Self::refresh_results(search, workspace, true);
        }

        if search.md_search.query.is_empty() && !search.md_history.recent_terms.is_empty() {
            ui.separator();
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
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
                        super::history::SearchHistoryUiOps::reset_navigation(search);
                    }
                })
                .show(ui);
            let terms = search.md_history.recent_terms.clone();
            for action in super::history::SearchHistoryUiOps::render_recent_terms(ui, &terms) {
                match action {
                    super::history::SearchHistoryAction::Select(term) => {
                        search.md_search.query = term;
                        super::history::SearchHistoryUiOps::reset_navigation(search);
                        Self::refresh_results(search, workspace, true);
                    }
                    super::history::SearchHistoryAction::Remove(term) => {
                        search.md_history.remove_term(&term);
                        super::history::SearchHistoryUiOps::reset_navigation(search);
                    }
                }
            }
        }

        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if search.md_results.is_empty() && !search.md_search.query.is_empty() {
                    ui.label(crate::i18n::I18nOps::get().search.no_results.clone());
                } else {
                    let ws_root = workspace.map(|ws| ws.root.clone());
                    for result in &search.md_results {
                        let rel = crate::shell_logic::ShellLogicOps::relative_full_path(
                            &result.file_path,
                            ws_root.as_deref(),
                        );
                        ui.group(|ui| {
                            crate::widgets::AlignCenter::new()
                                .shrink_to_fit(true)
                                .content(|ui| {
                                    ui.label(egui::RichText::new(&rel).strong());
                                    let ln = crate::i18n::I18nOps::get().search.ln_prefix.clone()
                                        + &(result.line_number + 1).to_string();
                                    ui.label(egui::RichText::new(ln).weak());
                                })
                                .show(ui);
                            let job = super::utils::SearchUtilsOps::build_snippet_job(ui, result);
                            if ui
                                .add(egui::Button::selectable(false, job).frame_when_inactive(true))
                                .clicked()
                            {
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

    fn refresh_results(
        search: &mut crate::app_state::SearchState,
        workspace: Option<&katana_core::workspace::Workspace>,
        remember: bool,
    ) {
        search.md_last_params = Some(search.md_search.clone());
        if search.md_search.query.is_empty() {
            search.md_results.clear();
            return;
        }

        let Some(workspace) = workspace else {
            return;
        };
        search.md_results = katana_core::search::WorkspaceSearchOps::search_workspace(
            workspace,
            &search.md_search.query,
            search.md_search.match_case,
            search.md_search.match_word,
            search.md_search.use_regex,
            MD_SEARCH_LIMIT,
        );
        if remember {
            search
                .md_history
                .push_term(search.md_search.query.clone(), MD_SEARCH_HISTORY_LIMIT);
        }
    }
}
