/* WHY: Encapsulated filename search tab rendering logic to manage UI complexity and maintain modularity. */

use crate::app_state::AppAction;
use eframe::egui;

pub struct FilenameTabOps;

impl FilenameTabOps {
    pub(crate) fn show_filename_tab(
        ui: &mut egui::Ui,
        search: &mut crate::app_state::SearchState,
        workspace: Option<&katana_core::workspace::Workspace>,
        action: &mut AppAction,
    ) {
        let response = crate::widgets::SearchBar::new(&mut search.file_search)
            .hint_text(crate::i18n::I18nOps::get().search.query_hint.clone())
            .id_source("search_tabs_file_name_bar")
            .show(ui);
        if !search.focus_requested {
            response.request_focus();
            search.focus_requested = true;
        }

        let (include_regexes, include_valid) =
            super::utils::SearchUtilsOps::build_regexes(&search.include_pattern);
        let (exclude_regexes, exclude_valid) =
            super::utils::SearchUtilsOps::build_regexes(&search.exclude_pattern);

        let err_color = super::utils::SearchUtilsOps::get_error_color(ui);
        let include_color = if include_valid {
            ui.visuals().text_color()
        } else {
            err_color
        };
        let exclude_color = if exclude_valid {
            ui.visuals().text_color()
        } else {
            err_color
        };

        crate::widgets::SearchBar::simple(&mut search.include_pattern)
            .hint_text(
                crate::i18n::I18nOps::get()
                    .search
                    .include_pattern_hint
                    .clone(),
            )
            .id_source("search_tabs_include_bar")
            .text_color(include_color)
            .show(ui);

        crate::widgets::SearchBar::simple(&mut search.exclude_pattern)
            .hint_text(
                crate::i18n::I18nOps::get()
                    .search
                    .exclude_pattern_hint
                    .clone(),
            )
            .id_source("search_tabs_exclude_bar")
            .text_color(exclude_color)
            .show(ui);

        let current_params = (
            search.file_search.clone(),
            search.include_pattern.clone(),
            search.exclude_pattern.clone(),
        );
        if search.last_params.as_ref() != Some(&current_params) {
            search.last_params = Some(current_params);
            let query = search.file_search.query.clone();
            if query.is_empty() && include_regexes.is_empty() && exclude_regexes.is_empty() {
                search.results.clear();
            } else if let Some(ws) = workspace {
                let mut results = Vec::new();
                crate::shell_logic::ShellLogicOps::collect_matches(
                    &ws.tree,
                    &query,
                    &include_regexes,
                    &exclude_regexes,
                    &ws.root,
                    search.file_search.match_case,
                    search.file_search.match_word,
                    search.file_search.use_regex,
                    &mut results,
                );
                search.results = results;
            }
        }

        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                if search.results.is_empty() && !search.file_search.query.is_empty() {
                    ui.label(crate::i18n::I18nOps::get().search.no_results.clone());
                } else {
                    let ws_root = workspace.map(|ws| ws.root.clone());
                    for path in &search.results {
                        let rel = crate::shell_logic::ShellLogicOps::relative_full_path(
                            path,
                            ws_root.as_deref(),
                        );
                        if ui
                            .add(egui::Button::selectable(false, rel).frame_when_inactive(true))
                            .clicked()
                            && path.exists()
                        {
                            *action = AppAction::SelectDocument(path.to_path_buf());
                        }
                    }
                }
            });
    }
}
