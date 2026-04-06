use crate::app_state::AppAction;
use eframe::egui;

const SPACING_TOP_MARGIN: f32 = 60.0;
const SPACING_ICON_BOTTOM: f32 = 24.0;
const SPACING_HEADING_BOTTOM: f32 = 8.0;
const SPACING_HINT_BOTTOM: f32 = 12.0;
const BUTTON_TEXT_SIZE: f32 = 16.0;
const SPACING_RECENT_SEP_TOP: f32 = 20.0;
const SPACING_RECENT_SEP_BOTTOM: f32 = 10.0;
const SPACING_RECENT_LIST_TOP: f32 = 4.0;

pub(crate) struct EmptyWorkspaceView<'a> {
    pub histories: &'a [String],
    pub action: &'a mut AppAction,
}

impl<'a> EmptyWorkspaceView<'a> {
    pub fn new(histories: &'a [String], action: &'a mut AppAction) -> Self {
        Self { histories, action }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        ui.add_space(SPACING_TOP_MARGIN); // Push content down a bit for aesthetics

        ui.vertical_centered(|ui| {
            ui.add(crate::Icon::Explorer.ui_image(ui, crate::icon::IconSize::Large));
            ui.add_space(SPACING_ICON_BOTTOM);

            ui.heading(
                egui::RichText::new(
                    crate::i18n::I18nOps::get()
                        .workspace
                        .no_workspace_open
                        .clone(),
                )
                .strong(),
            );
            ui.add_space(SPACING_HEADING_BOTTOM);

            ui.label(
                egui::RichText::new(
                    crate::i18n::I18nOps::get()
                        .workspace
                        .open_folder_hint
                        .clone(),
                )
                .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(SPACING_HINT_BOTTOM);

            if ui
                .add(egui::Button::new(
                    egui::RichText::new(crate::i18n::I18nOps::get().menu.open_workspace.clone())
                        .size(BUTTON_TEXT_SIZE),
                ))
                .clicked()
            {
                *self.action = crate::shell_ui::ShellUiOps::pick_open_workspace();
            }
        });

        ui.add_space(SPACING_RECENT_SEP_TOP);
        ui.separator();
        ui.add_space(SPACING_RECENT_SEP_BOTTOM);

        crate::widgets::AlignCenter::new()
            .left(|ui| {
                ui.add_space(crate::shell::RECENT_WORKSPACES_HEADING_LEFT_PADDING);
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .left(|ui| {
                ui.label(
                    egui::RichText::new(
                        crate::i18n::I18nOps::get()
                            .workspace
                            .recent_workspaces
                            .clone(),
                    )
                    .strong()
                    .color(ui.visuals().weak_text_color()),
                )
            })
            .show(ui);
        ui.add_space(SPACING_RECENT_LIST_TOP);

        let histories: Vec<String> = self.histories.to_vec();
        crate::views::panels::explorer::shared::SharedPathListRenderer::render_with_scroll(
            ui,
            &histories,
            None,
            self.action,
            false,
        );
    }
}
