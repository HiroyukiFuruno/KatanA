use egui;
use std::path::PathBuf;

const MOVE_MODAL_WIDTH: f32 = 360.0;
const MOVE_MODAL_SPACING: f32 = 8.0;

pub(crate) struct MoveModal<'a> {
    pub modal_data: &'a (PathBuf, PathBuf),
    pub ws_root: Option<&'a std::path::Path>,
    pub pending_action: &'a mut crate::app_state::AppAction,
}

impl<'a> MoveModal<'a> {
    pub fn new(
        modal_data: &'a (PathBuf, PathBuf),
        ws_root: Option<&'a std::path::Path>,
        pending_action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self {
            modal_data,
            ws_root,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let (source_path, target_dir) = self.modal_data;
        let Some(file_name) = source_path.file_name() else {
            return true;
        };
        let target_path = target_dir.join(file_name);
        let mut close = false;
        let mut is_open = true;
        egui::Window::new(crate::i18n::I18nOps::get().dialog.move_title.clone())
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .max_width(MOVE_MODAL_WIDTH)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let source =
                    crate::views::panels::explorer::drag::ExplorerDragUi::relative_display_path(
                        source_path,
                        self.ws_root,
                        false,
                    );
                let target =
                    crate::views::panels::explorer::drag::ExplorerDragUi::relative_display_path(
                        &target_path,
                        self.ws_root,
                        false,
                    );
                ui.label(crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().dialog.move_confirm_msg,
                    &[("source", source.as_str()), ("target", target.as_str())],
                ));
                ui.add_space(MOVE_MODAL_SPACING);
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        if ui
                            .button(crate::i18n::I18nOps::get().action.cancel.clone())
                            .clicked()
                        {
                            close = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(crate::i18n::I18nOps::get().action.confirm.clone())
                                .clicked()
                            {
                                *self.pending_action = crate::app_state::AppAction::MoveFsNode {
                                    source_path: source_path.clone(),
                                    target_path,
                                };
                                close = true;
                            }
                        });
                    })
                    .show(ui);
            });
        close || !is_open
    }
}
