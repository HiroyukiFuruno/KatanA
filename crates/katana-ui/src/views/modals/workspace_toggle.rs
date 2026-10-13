use crate::app_state::AppAction;
use crate::widgets::Modal;
use eframe::egui;

pub(crate) struct WorkspaceToggleModal<'a> {
    pub title: &'a str,
    pub paths: &'a [String],
    pub current_root: Option<String>,
    pub action: &'a mut AppAction,
    pub is_open: &'a mut bool,
    pub is_history: bool,
    pub pos_y: f32,
}

impl<'a> WorkspaceToggleModal<'a> {
    pub fn new(
        title: &'a str,
        paths: &'a [String],
        current_root: Option<String>,
        action: &'a mut AppAction,
        is_open: &'a mut bool,
        is_history: bool,
        pos_y: f32,
    ) -> Self {
        Self {
            title,
            paths,
            current_root,
            action,
            is_open,
            is_history,
            pos_y,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        if !*self.is_open {
            return;
        }

        let mut close_modal = false;

        const MAX_MODAL_WIDTH: f32 = 400.0;
        const MAX_MODAL_HEIGHT: f32 = 400.0;
        const DEFAULT_WINDOW_POS_X: f32 = 60.0;

        let response = egui::Window::new(self.title)
            .id(egui::Id::new("workspace_toggle_modal"))
            .open(self.is_open)
            .collapsible(false)
            .resizable(false)
            .default_width(MAX_MODAL_WIDTH)
            .default_pos([DEFAULT_WINDOW_POS_X, self.pos_y])
            .show(ctx, |ui| {
                /* WHY: Provide some vertical space restraint since the list can grow */
                egui::ScrollArea::vertical()
                    .max_height(MAX_MODAL_HEIGHT)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        if self.paths.is_empty() {
                            let text = if self.is_history {
                                crate::i18n::I18nOps::get()
                                    .workspace
                                    .no_recent_workspaces
                                    .clone()
                            } else {
                                crate::i18n::I18nOps::get()
                                    .workspace
                                    .no_saved_workspaces
                                    .clone()
                            };
                            ui.label(text);
                        } else {
                            let mut paths = self.paths.to_vec();
                            paths.reverse();
                            for path in paths {
                                Self::render_history_item(
                                    ui,
                                    &path,
                                    self.action,
                                    &mut close_modal,
                                    self.is_history,
                                );
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

        if let Some(res) = response
            && res.response.clicked_elsewhere()
        {
            close_modal = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            close_modal = true;
        }

        if close_modal {
            *self.is_open = false;
        }
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
        is_history: bool,
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
                    /* WHY: Show directory name only; full path is shown in tooltip. */
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
                        /* WHY: History panel only removes from history; workspace panel removes from persisted list */
                        if is_history {
                            *action = crate::app_state::AppAction::RemoveWorkspaceHistory(
                                path.to_string(),
                            );
                        } else {
                            *action =
                                crate::app_state::AppAction::RemoveWorkspace(path.to_string());
                        }
                    }
                });
            },
        );
    }
}
