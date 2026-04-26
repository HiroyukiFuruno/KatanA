use crate::app_action::{AppAction, CodeBlockKind, MarkdownAuthoringOp};
use eframe::egui;

pub(crate) struct CodeBlockMenuOps;

impl CodeBlockMenuOps {
    pub(crate) fn show(ui: &mut egui::Ui, action: &mut AppAction) {
        for kind in CodeBlockKind::all() {
            if ui.button(kind.display_label()).clicked() {
                *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::CodeBlock(*kind));
                ui.close();
            }
        }
    }
}
