use crate::app_state::AppAction;
use eframe::egui;

const MD_SEARCH_LIMIT: usize = 50;
const MD_SEARCH_HISTORY_LIMIT: usize = 10;

pub(super) fn show_filename_tab(
    ui: &mut egui::Ui,
    search: &mut crate::app_state::SearchState,
    workspace: Option<&katana_core::workspace::Workspace>,
    action: &mut AppAction,
) {
    let response = crate::widgets::SearchBar::new(&mut search.file_search)
        .hint_text(crate::i18n::I18nOps::get().search.query_hint.clone())
        .show(ui);
    if !search.focus_requested {
        response.request_focus();
        search.focus_requested = true;
    }

    let (include_regexes, include_valid) = build_regexes(&search.include_pattern);
    let (exclude_regexes, exclude_valid) = build_regexes(&search.exclude_pattern);

    let err_color = get_error_color(ui);
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
        .text_color(include_color)
        .show_search_icon(true)
        .show(ui);

    crate::widgets::SearchBar::simple(&mut search.exclude_pattern)
        .hint_text(
            crate::i18n::I18nOps::get()
                .search
                .exclude_pattern_hint
                .clone(),
        )
        .text_color(exclude_color)
        .show_search_icon(true)
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

pub(super) fn show_md_tab(
    ui: &mut egui::Ui,
    search: &mut crate::app_state::SearchState,
    workspace: Option<&katana_core::workspace::Workspace>,
    action: &mut AppAction,
) {
    let response = crate::widgets::SearchBar::new(&mut search.md_search)
        .hint_text(crate::i18n::I18nOps::get().search.md_query_hint.clone())
        .show(ui);
    if !search.focus_requested {
        response.request_focus();
        search.focus_requested = true;
    }

    if (response.changed()
        || response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
        && search.md_last_params.as_ref() != Some(&search.md_search)
    {
        search.md_last_params = Some(search.md_search.clone());
        if search.md_search.query.is_empty() {
            search.md_results.clear();
        } else if let Some(ws) = workspace {
            /* WHY: WorkspaceSearchOps::search_workspace needs to accept params now, but for now we pass query and other fields */
            search.md_results = katana_core::search::WorkspaceSearchOps::search_workspace(
                ws,
                &search.md_search.query,
                search.md_search.match_case,
                search.md_search.match_word,
                search.md_search.use_regex,
                MD_SEARCH_LIMIT,
            );
            search
                .md_history
                .push_term(search.md_search.query.clone(), MD_SEARCH_HISTORY_LIMIT);
        }
    }

    if search.md_search.query.is_empty() && !search.md_history.recent_terms.is_empty() {
        ui.separator();
        crate::widgets::AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                ui.label(
                    egui::RichText::new(crate::i18n::I18nOps::get().search.recent_searches.clone())
                        .strong(),
                );
                if ui
                    .button(crate::i18n::I18nOps::get().search.clear_history.clone())
                    .clicked()
                {
                    search.md_history.clear();
                }
            })
            .show(ui);
        for term in search.md_history.recent_terms.clone() {
            if !ui.link(&term).clicked() {
                continue;
            }
            search.md_search.query = term.clone();
            let params = search.md_search.clone();
            search.md_last_params = Some(params);
            if let Some(ws) = workspace {
                search.md_results = katana_core::search::WorkspaceSearchOps::search_workspace(
                    ws,
                    &term,
                    search.md_search.match_case,
                    search.md_search.match_word,
                    search.md_search.use_regex,
                    MD_SEARCH_LIMIT,
                );
                search.md_history.push_term(term, MD_SEARCH_HISTORY_LIMIT);
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
                        let job = build_snippet_job(ui, result);
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

fn build_regexes(pattern: &str) -> (Vec<regex::Regex>, bool) {
    let mut regexes = Vec::new();
    let mut valid = true;
    if pattern.is_empty() {
        return (regexes, valid);
    }
    /* WHY: Precompile query regex for efficiency so we don't compile inside the loop */
    for pat in pattern.split(',') {
        let pat = pat.trim();
        if pat.is_empty() {
            continue;
        }
        match regex::Regex::new(pat) {
            Ok(re) => regexes.push(re),
            Err(_) => valid = false,
        }
    }
    (regexes, valid)
}

fn get_error_color(ui: &egui::Ui) -> egui::Color32 {
    ui.ctx()
        .data(|d| {
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        })
        .map_or(crate::theme_bridge::WHITE, |tc| {
            crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.system.error_text)
        })
}

fn build_snippet_job(
    ui: &egui::Ui,
    result: &katana_core::search::SearchResult,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let (start, end) = (result.start_col, result.end_col);
    if start <= end && end <= result.snippet.len() {
        job.append(&result.snippet[..start], 0.0, egui::TextFormat::default());
        job.append(
            &result.snippet[start..end],
            0.0,
            egui::TextFormat {
                color: ui.visuals().strong_text_color(),
                background: ui.visuals().selection.bg_fill,
                ..Default::default()
            },
        );
        job.append(&result.snippet[end..], 0.0, egui::TextFormat::default());
    } else {
        job.append(&result.snippet, 0.0, egui::TextFormat::default());
    }
    job
}
