use crate::app_state::AppAction;
use eframe::egui;

pub(super) struct EditorContextMenuImageIngestOps;

impl EditorContextMenuImageIngestOps {
    pub(super) fn render(ui: &mut egui::Ui, action: &mut AppAction) {
        let search = &crate::i18n::I18nOps::get().search;
        if ui.button(&search.command_ingest_image_file).clicked() {
            super::context_menu::EditorContextMenu::close_after_action(
                ui,
                action,
                AppAction::IngestImageFile,
            );
        }
        if Self::clipboard_image_button(ui, &search.command_ingest_clipboard_image).clicked() {
            super::context_menu::EditorContextMenu::close_after_action(
                ui,
                action,
                AppAction::IngestClipboardImage,
            );
        }
    }

    fn clipboard_image_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
        ui.add_enabled(
            crate::app::action::clipboard_image::ClipboardImageOps::has_image_payload(),
            egui::Button::new(label),
        )
    }
}
