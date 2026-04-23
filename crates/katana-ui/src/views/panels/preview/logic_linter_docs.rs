use super::types::*;
use eframe::egui;

impl PreviewLogicOps {
    /// FB25: Render a "View on GitHub" button at the top-right of the preview
    /// pane when viewing a LinterDocs virtual document.
    pub fn render_linter_docs_github_button(ui: &mut egui::Ui, doc_path: &std::path::Path) {
        let p = doc_path.to_string_lossy();
        if !p.starts_with("Katana://LinterDocs/") {
            return;
        }
        let Some(rule_id) = p
            .strip_prefix("Katana://LinterDocs/")
            .and_then(|s| s.strip_suffix(".md"))
            .map(|s| s.to_ascii_lowercase())
        else {
            return;
        };

        const GITHUB_BTN_SIZE: f32 = 28.0;
        const GITHUB_BTN_MARGIN: f32 = 16.0;
        let btn_rect = egui::Rect::from_min_size(
            egui::pos2(
                ui.max_rect().right() - GITHUB_BTN_MARGIN - GITHUB_BTN_SIZE,
                ui.max_rect().top() + GITHUB_BTN_MARGIN,
            ),
            egui::vec2(GITHUB_BTN_SIZE, GITHUB_BTN_SIZE),
        );
        let mut overlay_ui = ui.new_child(egui::UiBuilder::new().max_rect(btn_rect).layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
        ));
        let tooltip = crate::i18n::I18nOps::get().linter.view_on_github.clone();
        let github_url =
            format!("https://github.com/DavidAnson/markdownlint/blob/main/doc/{rule_id}.md",);
        let btn = crate::icon::Icon::Github.button(&overlay_ui, crate::icon::IconSize::Small);
        if overlay_ui.add(btn).on_hover_text(&tooltip).clicked() {
            let _ = open::that(&github_url);
        }
    }
}
