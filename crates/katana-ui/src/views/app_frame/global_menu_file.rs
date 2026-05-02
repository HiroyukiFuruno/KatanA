use super::global_menu_context::GlobalMenuContext;
use crate::app_state::AppAction;
use eframe::egui;

pub(super) struct GlobalFileMenu;

impl GlobalFileMenu {
    pub(super) fn render(ui: &mut egui::Ui, context: &mut GlobalMenuContext<'_>) {
        let menu_label = context.i18n().menu.file.clone();
        crate::widgets::MenuButtonOps::show(ui, &menu_label, |ui| {
            let open_workspace = context.i18n().menu.open_workspace.clone();
            context.shortcut_action_item(
                ui,
                "file.open_workspace",
                &open_workspace,
                "open_workspace",
                AppAction::PickOpenWorkspace,
            );
            let open_file = context.i18n().action.open_file.clone();
            context.action_item(
                ui,
                "file.open_file_current_workspace",
                &open_file,
                AppAction::PickOpenFileInCurrentWorkspace,
            );
            let close_workspace = context.i18n().menu.close_workspace.clone();
            context.action_item(
                ui,
                "file.close_workspace",
                &close_workspace,
                AppAction::CloseWorkspace,
            );
            ui.separator();
            let save = context.i18n().menu.save.clone();
            context.shortcut_action_item(
                ui,
                "file.save",
                &save,
                "save_document",
                AppAction::SaveDocument,
            );
            let ingest_clipboard_image =
                context.i18n().search.command_ingest_clipboard_image.clone();
            context.action_item(
                ui,
                "edit.ingest_clipboard_image",
                &ingest_clipboard_image,
                AppAction::IngestClipboardImage,
            );
        });
    }
}
