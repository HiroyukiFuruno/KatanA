pub(super) use super::side_panel_sidebar_buttons::SidebarButton;
use super::side_panel_sidebar_policy::SidebarAvailability;
use super::side_panels::{PREVIEW_SIDE_BAR_SPACING, PreviewSidePanels};
use crate::app_state::AppAction;
use eframe::egui;

impl PreviewSidePanels<'_> {
    pub(super) fn render_sidebar_controls(
        &mut self,
        ui: &mut egui::Ui,
        availability: SidebarAvailability,
    ) {
        ui.spacing_mut().item_spacing.y = PREVIEW_SIDE_BAR_SPACING;
        self.render_toc_control(ui, availability.toc);
        self.render_primary_controls(ui, availability.refresh, availability.search);
        self.render_auxiliary_controls(ui, availability);
        self.render_info_control(ui, availability.info);
    }

    fn render_toc_control(&mut self, ui: &mut egui::Ui, available: bool) {
        if !self.app.state.config.settings.settings().layout.toc_visible {
            return;
        }
        let label = &crate::i18n::I18nOps::get().action.toggle_toc;
        let response = self.render_action_button(
            ui,
            SidebarButton {
                available,
                icon: crate::Icon::Toc,
                active: self.app.state.layout.show_toc,
                label,
                shortcut: None,
                action: AppAction::ToggleToc,
            },
        );
        self.toc_btn_rect = Some(response.rect);
        if response.clicked() {
            /* WHY: Avoid reopening the panel while the pointer remains on its button. */
            ui.ctx().data_mut(|data| {
                data.insert_temp(egui::Id::new("toc_hover_cooldown"), true);
            });
        }
    }

    fn render_primary_controls(&mut self, ui: &mut egui::Ui, refresh: bool, search: bool) {
        let i18n = crate::i18n::I18nOps::get();
        self.render_action_button(
            ui,
            SidebarButton {
                available: refresh,
                icon: crate::Icon::Refresh,
                active: false,
                label: &i18n.action.refresh_document,
                shortcut: None,
                action: AppAction::RefreshDocument { is_manual: true },
            },
        );
        let shortcut = crate::os_command::OsCommandOps::get("search_tab");
        self.render_action_button(
            ui,
            SidebarButton {
                available: search,
                icon: crate::Icon::Search,
                active: self.app.state.search.doc_search_open,
                label: &i18n.search.doc_search_title,
                shortcut: Some(&shortcut),
                action: AppAction::ToggleDocSearch,
            },
        );
    }

    fn render_auxiliary_controls(&mut self, ui: &mut egui::Ui, availability: SidebarAvailability) {
        let export = self.render_export_control(ui, availability.export);
        let story = self.render_story_control(ui, availability.story);
        let tools = self.render_tools_control(ui, availability.tools);
        self.export_btn_rect = Some(export.rect);
        self.story_btn_rect = Some(story.rect);
        self.tools_btn_rect = Some(tools.rect);
        /* WHY: Popup hover for the three auxiliary panels shares one state machine. */
        self.handle_popup_hover(
            ui,
            availability.export && export.hovered(),
            availability.story && story.hovered(),
            availability.tools && tools.hovered(),
        );
    }

    fn render_export_control(&mut self, ui: &mut egui::Ui, available: bool) -> egui::Response {
        self.render_action_button(
            ui,
            SidebarButton {
                available,
                icon: crate::Icon::Export,
                active: self.app.state.layout.show_export_panel,
                label: &crate::i18n::I18nOps::get().menu.export,
                shortcut: None,
                action: AppAction::ToggleExportPanel,
            },
        )
    }

    fn render_story_control(&mut self, ui: &mut egui::Ui, available: bool) -> egui::Response {
        self.render_action_button(
            ui,
            SidebarButton {
                available,
                icon: crate::Icon::Preview,
                active: self.app.state.layout.show_story_panel,
                label: &crate::i18n::I18nOps::get().preview.slideshow_settings,
                shortcut: None,
                action: AppAction::ToggleStoryPanel,
            },
        )
    }

    fn render_tools_control(&mut self, ui: &mut egui::Ui, available: bool) -> egui::Response {
        self.render_action_button(
            ui,
            SidebarButton {
                available,
                icon: crate::Icon::Tools,
                active: self.app.state.layout.show_tools_panel,
                label: &crate::i18n::I18nOps::get().menu.view,
                shortcut: None,
                action: AppAction::ToggleToolsPanel,
            },
        )
    }
}

#[cfg(test)]
#[path = "side_panel_sidebar_auxiliary_tests.rs"]
mod tests;
