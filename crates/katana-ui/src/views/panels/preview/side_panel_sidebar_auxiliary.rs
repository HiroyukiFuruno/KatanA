use super::side_panels::{PREVIEW_SIDE_BAR_SPACING, PreviewSidePanels};
use crate::app_state::AppAction;
use eframe::egui;

#[derive(Clone, Copy)]
pub(super) struct SidebarAvailability {
    export: bool,
    story: bool,
    tools: bool,
}

struct SidebarButton<'a> {
    available: bool,
    icon: crate::Icon,
    active: bool,
    label: &'a str,
    shortcut: Option<&'a str>,
    action: AppAction,
}

impl PreviewSidePanels<'_> {
    pub(super) fn sidebar_availability(&self) -> SidebarAvailability {
        let path = self
            .app
            .state
            .active_document()
            .map(|document| document.path.as_path());
        SidebarAvailability {
            export: super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Export),
            story: super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Story),
            tools: super::PreviewMenuAvailability::for_path(path, super::PreviewMenu::Tools),
        }
    }

    pub(super) fn render_sidebar_controls(
        &mut self,
        ui: &mut egui::Ui,
        availability: SidebarAvailability,
    ) {
        ui.spacing_mut().item_spacing.y = PREVIEW_SIDE_BAR_SPACING;
        self.render_toc_control(ui);
        self.render_primary_controls(ui);
        self.render_auxiliary_controls(ui, availability);
        self.render_info_control(ui);
    }

    fn render_toc_control(&mut self, ui: &mut egui::Ui) {
        if !self.app.state.config.settings.settings().layout.toc_visible {
            return;
        }
        let path = self
            .app
            .state
            .active_document()
            .map(|document| document.path.as_path());
        let available = super::TocAvailability::for_path(path);
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

    fn render_primary_controls(&mut self, ui: &mut egui::Ui) {
        let i18n = crate::i18n::I18nOps::get();
        self.render_action_button(
            ui,
            SidebarButton {
                available: true,
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
                available: true,
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

    fn render_info_control(&mut self, ui: &mut egui::Ui) {
        let response = self.render_toggle_button(
            ui,
            crate::Icon::Info,
            false,
            &crate::i18n::I18nOps::get().meta_info.title,
            None,
        );
        if response.clicked()
            && let Some(document) = self.app.state.active_document()
        {
            self.app.pending_action = AppAction::ShowMetaInfo(document.path.clone());
        }
    }

    fn render_action_button(
        &mut self,
        ui: &mut egui::Ui,
        button: SidebarButton<'_>,
    ) -> egui::Response {
        let response = ui
            .add_enabled_ui(button.available, |ui| {
                self.render_toggle_button(
                    ui,
                    button.icon,
                    button.active,
                    button.label,
                    button.shortcut,
                )
            })
            .inner;
        if response.clicked() {
            self.app.pending_action = button.action;
        }
        response
    }
}
