use crate::app_state::AppAction;
use eframe::egui;

const ITEM_SPACING: f32 = 2.0;

pub(crate) struct HistoryPanel<'a> {
    pub histories: &'a [String],
    pub action: &'a mut AppAction,
}

impl<'a> HistoryPanel<'a> {
    pub fn new(histories: &'a [String], action: &'a mut AppAction) -> Self {
        Self { histories, action }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let panel_width = ui.available_width();
        ui.set_max_width(panel_width);
        ui.set_min_width(panel_width);
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

        let histories: Vec<String> = self.histories.to_vec();

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            /* WHY: Pad label slightly according to typical header spacing layout */
            ui.add_space(crate::shell::RECENT_WORKSPACES_HEADING_LEFT_PADDING);
            ui.label(
                egui::RichText::new(
                    crate::i18n::I18nOps::get()
                        .workspace
                        .workspace_history_title
                        .clone(),
                )
                .strong(),
            );
        });
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if histories.is_empty() {
                    ui.vertical_centered(|ui| {
                        ui.add_space(crate::shell::HISTORY_MODAL_EMPTY_BOTTOM_SPACING);
                        ui.label(crate::i18n::I18nOps::get().status.no_problems_found.clone());
                    });
                } else {
                    for path in histories.iter().rev() {
                        Self::render_history_item(ui, path, self.action);
                        ui.add_space(ITEM_SPACING);
                    }
                }
            });
    }

    fn render_history_item(ui: &mut egui::Ui, path: &str, action: &mut AppAction) {
        const MIN_AVAILABLE_WIDTH: f32 = 10.0;
        let remove_width = crate::shell::RECENT_WORKSPACES_CLOSE_BUTTON_WIDTH;
        let available_width = (ui.available_width() - remove_width - ui.spacing().item_spacing.x)
            .max(MIN_AVAILABLE_WIDTH);

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.set_min_width(available_width);
                    ui.set_max_width(available_width);

                    let display_name = std::path::Path::new(path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(path);

                    if ui
                        .add(egui::Button::new(display_name).frame(false).truncate())
                        .on_hover_text(path)
                        .clicked()
                    {
                        *action = crate::app_state::AppAction::OpenWorkspace(
                            std::path::PathBuf::from(path),
                        );
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let remove_icon = crate::Icon::Remove.button(ui, crate::icon::IconSize::Small);
                    if ui
                        .add(remove_icon)
                        .on_hover_text(crate::i18n::I18nOps::get().action.remove_workspace.clone())
                        .clicked()
                    {
                        *action =
                            crate::app_state::AppAction::RemoveWorkspaceHistory(path.to_string());
                    }
                });
            },
        );
    }
}
