use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct WorkspaceHistoryModal<'a> {
    pub is_open: &'a mut bool,
    pub recent_paths: &'a [String],
    pub action: &'a mut AppAction,
}

impl<'a> WorkspaceHistoryModal<'a> {
    pub fn new(
        is_open: &'a mut bool,
        recent_paths: &'a [String],
        action: &'a mut AppAction,
    ) -> Self {
        Self {
            is_open,
            recent_paths,
            action,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        let mut local_is_open = *self.is_open;
        let mut close_modal = false;

        const DEFAULT_MODAL_WIDTH: f32 = 500.0;

        egui::Window::new(
            crate::i18n::I18nOps::get()
                .workspace
                .recent_workspaces
                .clone(),
        )
        .open(&mut local_is_open)
        .collapsible(false)
        .resizable(false)
        .default_width(DEFAULT_MODAL_WIDTH)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    if self.recent_paths.is_empty() {
                        ui.label(
                            crate::i18n::I18nOps::get()
                                .workspace
                                .recent_workspaces
                                .clone(),
                        );
                    } else {
                        let mut paths = self.recent_paths.to_vec();
                        paths.reverse();
                        for path in paths {
                            Self::render_history_item(ui, &path, self.action, &mut close_modal);
                            ui.add_space(2.0);
                        }
                    }

                    ui.separator();
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            Self::render_history_open_workspace_btn(
                                ui,
                                self.action,
                                &mut close_modal,
                            );
                        },
                    );
                });
        });

        if close_modal {
            local_is_open = false;
        }
        *self.is_open = local_is_open;
    }

    fn render_history_open_workspace_btn(
        ui: &mut egui::Ui,
        action: &mut AppAction,
        close_modal: &mut bool,
    ) {
        if ui
            .button(crate::i18n::I18nOps::get().menu.open_workspace.clone())
            .clicked()
            && let Some(path) = crate::shell_ui::ShellUiOps::open_folder_dialog()
        {
            *action = crate::app_state::AppAction::OpenWorkspace(path);
            *close_modal = true;
        }
    }

    fn render_history_item(
        ui: &mut egui::Ui,
        path: &str,
        action: &mut AppAction,
        close_modal: &mut bool,
    ) {
        const MIN_AVAILABLE_WIDTH: f32 = 10.0;

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let remove_width =
                    crate::icon::IconSize::Small.to_vec2().x + ui.spacing().button_padding.x * 2.0;
                let available_width =
                    (ui.available_width() - remove_width - ui.spacing().item_spacing.x)
                        .max(MIN_AVAILABLE_WIDTH);

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.set_min_width(available_width);
                    ui.set_max_width(available_width);
                    if ui
                        .add(egui::Button::new(path).frame(false).truncate())
                        .on_hover_text(path)
                        .clicked()
                    {
                        *action = crate::app_state::AppAction::OpenWorkspace(
                            std::path::PathBuf::from(path),
                        );
                        *close_modal = true;
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
                        *close_modal = true;
                    }
                });
            },
        );
    }
}
