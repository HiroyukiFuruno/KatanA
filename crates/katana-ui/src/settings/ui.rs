use super::types::*;
use crate::app_state::{AppAction, SettingsTab};
use crate::preview_pane::PreviewPane;
use eframe::egui;

impl<'a> SettingsWindow<'a> {
    pub fn new(
        state: &'a mut crate::app_state::AppState,
        preview_pane: &'a mut PreviewPane,
    ) -> Self {
        Self {
            state,
            preview_pane,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> Option<AppAction> {
        let state = self.state;
        let preview_pane = self.preview_pane;

        if !state.layout.show_settings {
            return None;
        }

        let mut triggered_action: Option<AppAction> = None;

        if preview_pane.sections.is_empty() {
            preview_pane.update_markdown_sections(
                SAMPLE_MARKDOWN,
                std::path::Path::new("/settings-preview.md"),
            );
        }

        let mut open = state.layout.show_settings;
        egui::Window::new(crate::i18n::I18nOps::get().settings.title.clone())
            .open(&mut open)
            .fixed_size(egui::vec2(
                SETTINGS_WINDOW_DEFAULT_WIDTH,
                SETTINGS_WINDOW_DEFAULT_HEIGHT,
            ))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(SETTINGS_WINDOW_DEFAULT_WIDTH);
                ui.set_min_height(SETTINGS_WINDOW_DEFAULT_HEIGHT);

                egui::Panel::left("settings_left_panel")
                    .resizable(false)
                    .min_size(SETTINGS_SIDE_PANEL_DEFAULT_WIDTH)
                    .max_size(SETTINGS_SIDE_PANEL_DEFAULT_WIDTH)
                    .show_inside(ui, |ui| {
                        crate::widgets::AlignCenter::new().shrink_to_fit(true).content(|ui| {
                            const TAB_SPACING: f32 = 4.0;
                            ui.add_space(TAB_SPACING);
                            if ui
                                .add(
                                    crate::Icon::ExpandAll
                                        .button(ui, crate::icon::IconSize::Small)
                                )
                                .on_hover_text(
                                    crate::i18n::I18nOps::get().action.expand_all.clone(),
                                )
                                .clicked()
                            {
                                state.config.settings_tree_force_open = Some(true);
                            }
                            if ui
                                .add(
                                    crate::Icon::CollapseAll
                                        .button(ui, crate::icon::IconSize::Small)
                                )
                                .on_hover_text(
                                    crate::i18n::I18nOps::get().action.collapse_all.clone(),
                                )
                                .clicked()
                            {
                                state.config.settings_tree_force_open = Some(false);
                            }
                        }).show(ui);
                        const TAB_SPACING: f32 = 4.0;
                        ui.add_space(TAB_SPACING);
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .id_salt("settings_nav_scroll")
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                SettingsOps::render_settings_tree(ui, state);
                            });
                    });

                let show_preview = matches!(
                    state.config.active_settings_tab,
                    SettingsTab::Theme | SettingsTab::Font | SettingsTab::Layout
                );

                if show_preview {
                    egui::Panel::right("settings_right_panel")
                        .resizable(false)
                        .min_size(SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH)
                        .max_size(SETTINGS_PREVIEW_PANEL_DEFAULT_WIDTH)
                        .show_inside(ui, |ui| {
                            SettingsOps::section_header(
                                ui,
                                &crate::i18n::I18nOps::get().settings.preview.title,
                            );
                            preview_pane.show(ui);
                        });
                }

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    let title = super::settings_helpers::active_tab_title(&state.config.active_settings_tab);
                    SettingsOps::section_header(ui, &title);

                    match state.config.active_settings_tab {
                        SettingsTab::Icons => {
                            crate::settings::tabs::IconsTabOps::render_icons_tab(ui, state)
                        }
                        _ => {
                            egui::ScrollArea::vertical()
                                .id_salt("settings_form_scroll")
                                .auto_shrink(false)
                                .show(ui, |ui| {
                                    egui::Frame::NONE
                                        .inner_margin(INNER_MARGIN)
                                        .show(ui, |ui| match state.config.active_settings_tab {
                                            SettingsTab::Theme => {
                                                crate::settings::tabs::ThemeTabOps::render_theme_tab(ui, state)
                                            }
                                            SettingsTab::Font => {
                                                crate::settings::tabs::FontTabOps::render_font_tab(ui, state)
                                            }
                                            SettingsTab::Layout => {
                                                crate::settings::tabs::LayoutTabOps::render_layout_tab(
                                                    ui, state,
                                                );
                                            }
                                            SettingsTab::Workspace => {
                                                crate::settings::tabs::WorkspaceTabOps::render_workspace_tab(
                                                    ui, state,
                                                );
                                            }
                                            SettingsTab::Updates => {
                                                triggered_action = crate::settings::tabs::UpdatesTabOps::render_updates_tab(
                                                    ui, state,
                                                    state.config.get_plantuml_jar_path_if_exists(),
                                                    state.config.get_drawio_js_path_if_exists(),
                                                    state.config.get_mermaid_js_path_if_exists(),
                                                );
                                            }
                                            SettingsTab::Behavior => {
                                                triggered_action =
                                                    crate::settings::tabs::BehaviorTabOps::render_behavior_tab(
                                                        ui, state,
                                                    );
                                            }
                                            SettingsTab::Shortcuts => {
                                                crate::settings::tabs::ShortcutsTabOps::render_shortcuts_tab(
                                                    ui, state,
                                                );
                                            }
                                            SettingsTab::Linter => {
                                                if let Some(action) = crate::settings::tabs::LinterTabOps::render_linter_tab(ui, state) {
                                                    triggered_action = Some(action);
                                                }
                                            }
                                            _ => {}
                                        })
                                });
                        }
                    }
                });

                if state.config.settings_tree_force_open.is_some() {
                    state.config.settings_tree_force_open = None;
                }
            });
        state.layout.show_settings = open;
        triggered_action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_state::SettingsTab;
    use egui_kittest::Harness;
    use egui_kittest::kittest::Queryable;
    use katana_core::ai::AiProviderRegistry;
    use katana_core::plugin::PluginRegistry;

    #[test]
    fn advanced_settings_table_is_rendered() {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(1024.0, 768.0))
            .build_ui(|ui| {
                let mut state = crate::app_state::AppState::new(
                    AiProviderRegistry::new(),
                    PluginRegistry::new(),
                    katana_platform::SettingsService::default(),
                    std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
                );

                state.layout.show_settings = true;
                state.config.active_settings_tab = SettingsTab::Icons;

                ui.data_mut(|d| d.insert_temp(egui::Id::new("icons_advanced_is_open"), true));

                let mut preview_pane = PreviewPane::default();
                SettingsWindow::new(&mut state, &mut preview_pane).show(ui.ctx());
            });

        harness.run_steps(10);

        let i18n = crate::i18n::I18nOps::get();
        harness.get_by_label(&i18n.common.advanced_settings);
        harness.get_by_label(&i18n.settings.icons.table_header_vendor);
    }
}
