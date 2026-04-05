use crate::shell::KatanaApp;
use eframe::egui;

pub(crate) struct WorkspaceSidebarHistory;

impl WorkspaceSidebarHistory {
    pub(crate) fn render_history_popup(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        idx: usize,
        rect: egui::Rect,
        history_menu_id: egui::Id,
        interact_resp: egui::Response,
        recent_paths: &[String],
    ) {
        const HISTORY_MENU_X_OFFSET: f32 = 4.0;
        const HISTORY_MENU_MAX_WIDTH: f32 = 480.0;

        let popup_area = egui::Area::new(egui::Id::new("history_menu_area").with(idx))
            .order(egui::Order::Foreground)
            .fixed_pos(rect.right_top() + egui::vec2(HISTORY_MENU_X_OFFSET, 0.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_min_width(HISTORY_MENU_MAX_WIDTH);
                    ui.set_max_width(HISTORY_MENU_MAX_WIDTH);

                    for path in recent_paths.iter().rev() {
                        Self::render_history_item(ui, app, path, history_menu_id);
                    }

                    ui.separator();
                    ui.with_layout(
                        egui::Layout::top_down_justified(egui::Align::Center),
                        |ui| {
                            Self::render_history_open_workspace_btn(ui, app, history_menu_id);
                        },
                    );
                });
            });

        Self::check_history_popup_interaction(
            ui,
            popup_area.response.rect,
            interact_resp.rect,
            history_menu_id,
        );
    }

    fn render_history_open_workspace_btn(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        history_menu_id: egui::Id,
    ) {
        if ui
            .button(crate::i18n::I18nOps::get().menu.open_workspace.clone())
            .clicked()
        {
            if let Some(path) = crate::shell_ui::ShellUiOps::open_folder_dialog() {
                app.pending_action = crate::app_state::AppAction::OpenWorkspace(path);
                ui.data_mut(|data| data.insert_temp(history_menu_id, false));
            }
        }
    }

    fn check_history_popup_interaction(
        ui: &mut egui::Ui,
        popup_rect: egui::Rect,
        btn_rect: egui::Rect,
        history_menu_id: egui::Id,
    ) {
        let clicked_elsewhere = ui.input(|i| {
            if i.pointer.any_click() {
                let inter_pos = match i.pointer.interact_pos() {
                    Some(p) => p,
                    None => return true,
                };
                !popup_rect.contains(inter_pos) && !btn_rect.contains(inter_pos)
            } else {
                false
            }
        });

        if clicked_elsewhere || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.data_mut(|data| data.insert_temp(history_menu_id, false));
        }
    }

    fn render_history_item(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        path: &str,
        history_menu_id: egui::Id,
    ) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let remove_width =
                crate::icon::IconSize::Small.to_vec2().x + ui.spacing().button_padding.x * 2.0;
            let btn_spacing = ui.spacing().item_spacing.x;
            let available = ui.available_width();
            const MIN_LABEL_WIDTH: f32 = 10.0;
            let label_width = (available - remove_width - btn_spacing).max(MIN_LABEL_WIDTH);

            Self::render_history_path_btn(ui, app, path, history_menu_id, label_width);
            Self::render_history_remove_btn(ui, app, path, history_menu_id);
        });
    }

    fn render_history_path_btn(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        path: &str,
        history_menu_id: egui::Id,
        label_width: f32,
    ) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_min_width(label_width);
            ui.set_max_width(label_width);

            if ui
                .add(egui::Button::new(path).frame(false).truncate())
                .on_hover_text(path)
                .clicked()
            {
                app.pending_action =
                    crate::app_state::AppAction::OpenWorkspace(std::path::PathBuf::from(path));
                ui.data_mut(|data| data.insert_temp(history_menu_id, false));
            }
        });
    }

    fn render_history_remove_btn(
        ui: &mut egui::Ui,
        app: &mut KatanaApp,
        path: &str,
        history_menu_id: egui::Id,
    ) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let remove_icon = crate::Icon::Remove.button(ui, crate::icon::IconSize::Small);
            if ui
                .add(remove_icon)
                .on_hover_text(crate::i18n::I18nOps::get().action.remove_workspace.clone())
                .clicked()
            {
                app.pending_action = crate::app_state::AppAction::RemoveWorkspace(path.to_string());
                ui.data_mut(|data| data.insert_temp(history_menu_id, false));
            }
        });
    }
}
