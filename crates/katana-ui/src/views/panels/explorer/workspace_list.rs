use crate::app_state::AppAction;
use eframe::egui;

const WORKSPACE_LIST_ITEM_SPACING: f32 = 2.0;
const MIN_AVAILABLE_WIDTH: f32 = 10.0;

pub(crate) struct WorkspaceList<'a> {
    pub persisted: &'a [String],
    pub current_root: Option<String>,
    pub action: &'a mut AppAction,
}

impl<'a> WorkspaceList<'a> {
    pub fn new(
        persisted: &'a [String],
        current_root: Option<String>,
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            persisted,
            current_root,
            action,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        if self.persisted.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(crate::shell::HISTORY_MODAL_EMPTY_BOTTOM_SPACING);
                ui.label(crate::i18n::I18nOps::get().status.no_problems_found.clone());
            });
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for path in self.persisted.iter().rev() {
                    Self::render_item(ui, path, &self.current_root, self.action);
                    ui.add_space(WORKSPACE_LIST_ITEM_SPACING);
                }
            });
    }

    fn render_item(
        ui: &mut egui::Ui,
        path: &str,
        current_root: &Option<String>,
        action: &mut AppAction,
    ) {
        let is_active = current_root.as_ref().is_some_and(|root| root == path);
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

                    let text = if is_active {
                        egui::RichText::new(display_name).strong()
                    } else {
                        egui::RichText::new(display_name)
                    };
                    let btn = egui::Button::new(text).frame(false).truncate();
                    if ui.add(btn).on_hover_text(path).clicked() && !is_active {
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
                        *action = crate::app_state::AppAction::RemoveWorkspace(path.to_string());
                    }
                });
            },
        );
    }
}
