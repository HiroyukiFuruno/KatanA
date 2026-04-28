use crate::app_state::{AppAction, WorkspaceState};
use eframe::egui;
use std::path::PathBuf;

const FLAT_VIEW_TOGGLE_MARGIN: f32 = 8.0;

pub(crate) struct ExplorerHeaderMenu<'a> {
    pub workspace: &'a mut WorkspaceState,
    pub action: &'a mut AppAction,
    pub ws_root: PathBuf,
    pub is_flat: bool,
}

impl<'a> ExplorerHeaderMenu<'a> {
    pub(crate) fn show(mut self, ui: &mut egui::Ui) {
        let more_img = crate::Icon::More.ui_image(ui, crate::icon::IconSize::Small);
        ui.menu_image_button(more_img, |ui| {
            self.show_open_items(ui);
            self.show_workspace_items(ui);
        });
    }

    fn show_open_items(&mut self, ui: &mut egui::Ui) {
        if ui
            .button(crate::i18n::I18nOps::get().menu.open_workspace.clone())
            .clicked()
        {
            *self.action = crate::shell_ui::ShellUiOps::pick_open_workspace();
            ui.close();
        }
        if ui
            .button(crate::i18n::I18nOps::get().action.open_file.clone())
            .clicked()
        {
            *self.action = AppAction::PickOpenFileInCurrentWorkspace;
            ui.close();
        }
    }

    fn show_workspace_items(self, ui: &mut egui::Ui) {
        if self.workspace.data.is_none() {
            return;
        }

        ui.separator();
        let mut is_flat_mut = self.is_flat;
        let flat_view_label = crate::i18n::I18nOps::get().workspace.flat_view.clone();
        if ui
            .add(
                crate::widgets::LabeledToggle::new(&flat_view_label, &mut is_flat_mut).alignment(
                    crate::widgets::ToggleAlignment::Attached(FLAT_VIEW_TOGGLE_MARGIN),
                ),
            )
            .changed()
        {
            self.workspace.set_flat_view(self.ws_root, is_flat_mut);
            ui.close();
        }

        ui.separator();
        if ui
            .button(crate::i18n::I18nOps::get().menu.close_workspace.clone())
            .clicked()
        {
            *self.action = AppAction::CloseWorkspace;
            ui.close();
        }
    }
}
