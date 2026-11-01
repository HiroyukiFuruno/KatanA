use eframe::egui;

pub(super) struct ExplorerHeaderFilter;

impl ExplorerHeaderFilter {
    pub(super) fn show(
        ui: &mut egui::Ui,
        workspace: &crate::app_state::WorkspaceState,
        search: &mut crate::app_state::SearchState,
    ) {
        if workspace.data.is_none() || !search.filter_enabled {
            return;
        }

        let text_color = Self::filter_text_color(ui, &search.filter);
        /* WHY: filter bar does not show the search icon - icon is only for the file-name tab. */
        let resp = crate::widgets::SearchBar::new(&mut search.filter)
            .text_color(text_color)
            .hint_text(crate::i18n::I18nOps::get().workspace.filter_hint.clone())
            .show_search_icon(false)
            .id_source("workspace_filter_bar")
            .show(ui);
        let focus_requested = ui.ctx().data_mut(|d| {
            d.remove_temp::<bool>(egui::Id::new("filter_newly_enabled"))
                .unwrap_or(false)
        });
        if focus_requested {
            resp.request_focus();
        }
    }

    fn filter_text_color(
        ui: &egui::Ui,
        params: &crate::state::search::SearchParams,
    ) -> egui::Color32 {
        if super::filter_pattern::FilterPattern::compile(params).is_ok() {
            return ui.visuals().text_color();
        }
        ui.ctx()
            .data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            })
            .map_or(crate::theme_bridge::WHITE, |tc| {
                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.system.error_text)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::super::filter_pattern::FilterPattern;
    use crate::state::search::SearchParams;

    #[test]
    fn literal_filter_bracket_is_valid() {
        let params = SearchParams {
            query: "[".into(),
            ..Default::default()
        };
        assert!(FilterPattern::compile(&params).is_ok());
    }

    #[test]
    fn negated_regex_validates_the_stripped_pattern() {
        let params = SearchParams {
            query: "!*".into(),
            use_regex: true,
            ..Default::default()
        };
        assert!(FilterPattern::compile(&params).is_err());
    }
}
