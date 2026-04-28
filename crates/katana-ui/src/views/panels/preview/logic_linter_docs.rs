use super::types::*;
use eframe::egui;

impl PreviewLogicOps {
    /// FB25: Render a "View on GitHub" button at the top-right of the preview
    /// pane when viewing a LinterDocs virtual document.
    pub fn render_linter_docs_github_button(ui: &mut egui::Ui, doc_path: &std::path::Path) {
        let Some(identity) = crate::linter_docs::LinterDocIdentity::from_virtual_path(doc_path)
        else { return };

        const GITHUB_BTN_SIZE: f32 = 28.0;
        const GITHUB_BTN_MARGIN_TOP: f32 = 6.0;
        const GITHUB_BTN_MARGIN_RIGHT: f32 = 26.0;
        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(
                ui.max_rect().right() - GITHUB_BTN_MARGIN_RIGHT - GITHUB_BTN_SIZE,
                ui.max_rect().top() + GITHUB_BTN_MARGIN_TOP,
            ),
            egui::vec2(GITHUB_BTN_SIZE, GITHUB_BTN_SIZE),
        );
        let mut overlay_ui = ui.new_child(egui::UiBuilder::new().max_rect(btn_rect).layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        ));
        let tooltip = crate::i18n::I18nOps::get().linter.view_on_github.clone();
        let github_url = identity.github_url();
        let btn = crate::icon::Icon::Github.button(&overlay_ui, crate::icon::IconSize::Small);
        if overlay_ui.add(btn).on_hover_text(&tooltip).clicked() {
            let _ = open::that(&github_url);
        }
    }
}
