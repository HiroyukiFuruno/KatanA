use crate::app_state::AppAction;
use crate::shell::{RECENT_WORKSPACES_HEADING_LEFT_PADDING, RECENT_WORKSPACES_LIST_LEFT_PADDING};
use eframe::egui;

const SPACING_TOP_MARGIN: f32 = 60.0;
const SPACING_ICON_BOTTOM: f32 = 24.0;
const SPACING_HEADING_BOTTOM: f32 = 8.0;
const SPACING_HINT_BOTTOM: f32 = 20.0;
const BUTTON_TEXT_SIZE: f32 = 16.0;
const SPACING_HISTORY_TOP: f32 = 80.0;
const SPACING_HISTORY_HEADING_BOTTOM: f32 = 10.0;

pub(crate) struct EmptyWorkspaceView<'a> {
    pub recent_paths: &'a [String],
    pub action: &'a mut AppAction,
}

impl<'a> EmptyWorkspaceView<'a> {
    pub fn new(recent_paths: &'a [String], action: &'a mut AppAction) -> Self {
        Self {
            recent_paths,
            action,
        }
    }

    pub fn show(mut self, ui: &mut egui::Ui) {
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

        if !self.recent_paths.is_empty() {
            ui.add_space(SPACING_HISTORY_TOP); // Large gap before history

            crate::widgets::AlignCenter::new()
                .left(|ui| {
                    ui.add_space(RECENT_WORKSPACES_HEADING_LEFT_PADDING);
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

            ui.add_space(SPACING_HISTORY_HEADING_BOTTOM);
            self.show_recent_paths(ui);
        }
    }

    fn show_recent_paths(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("recent_workspaces_scroll")
            .show(ui, |ui| {
                for path in self.recent_paths.iter().rev() {
                    let mut open_clicked = false;
                    let mut remove_clicked = false;

                    let label_path = path.clone();
                    let remove_text = crate::i18n::I18nOps::get().action.remove_workspace.clone();

                    let resp = crate::widgets::AlignCenter::new()
                        .interactive(true)
                        .left(|ui| {
                            ui.add_space(RECENT_WORKSPACES_LIST_LEFT_PADDING);
                            ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                        })
                        .left(|ui| ui.add(egui::Label::new(label_path.as_str()).truncate()))
                        .right(|ui| {
                            let resp = ui.button("×").on_hover_text(&remove_text);
                            if resp.clicked() {
                                remove_clicked = true;
                            }
                            resp
                        })
                        .show(ui);

                    if resp.clicked() {
                        open_clicked = true;
                    }

                    if open_clicked {
                        *self.action = crate::app_state::AppAction::OpenWorkspace(
                            std::path::PathBuf::from(path.clone()),
                        );
                    }
                    if remove_clicked {
                        *self.action =
                            crate::app_state::AppAction::RemoveWorkspaceHistory(path.clone());
                    }
                }
            });
    }
}
