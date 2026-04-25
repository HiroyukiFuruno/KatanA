use crate::app_action::MarkdownAuthoringOp;
use crate::app_state::AppAction;
use crate::widgets::MenuButtonOps;
use eframe::egui;

pub(crate) struct EditorContextMenu;

impl EditorContextMenu {
    pub(crate) fn render(
        response: &egui::Response,
        action: &mut AppAction,
        cursor_range: Option<egui::text::CCursorRange>,
    ) {
        let has_selection = cursor_range.is_some_and(|r| r.primary.index != r.secondary.index);
        response.context_menu(|ui| {
            let i18n = crate::i18n::I18nOps::get();
            if ui.button(i18n.action.save.as_str()).clicked() {
                *action = AppAction::SaveDocument;
                ui.close();
            }
            ui.separator();
            MenuButtonOps::show(ui, i18n.settings.shortcuts.edit.clone(), |ui| {
                Self::render_inline(ui, action, has_selection);
                ui.separator();
                Self::render_structure(ui, action);
                ui.separator();
                Self::render_insert(ui, action);
            });
            MenuButtonOps::show(
                ui,
                i18n.settings.behavior.ingest_section_title.clone(),
                |ui| {
                    Self::render_image_ingest(ui, action);
                },
            );
        });
    }

    fn render_inline(ui: &mut egui::Ui, action: &mut AppAction, has_selection: bool) {
        let s = &crate::i18n::I18nOps::get().search;
        Self::author_button(
            ui,
            action,
            &s.command_author_bold,
            MarkdownAuthoringOp::Bold,
            has_selection,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_italic,
            MarkdownAuthoringOp::Italic,
            has_selection,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_strikethrough,
            MarkdownAuthoringOp::Strikethrough,
            has_selection,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_inline_code,
            MarkdownAuthoringOp::InlineCode,
            has_selection,
        );
    }

    fn render_structure(ui: &mut egui::Ui, action: &mut AppAction) {
        Self::render_headings(ui, action);
        ui.separator();
        Self::render_blocks(ui, action);
    }

    fn render_headings(ui: &mut egui::Ui, action: &mut AppAction) {
        let s = &crate::i18n::I18nOps::get().search;
        Self::author_button(
            ui,
            action,
            &s.command_author_heading1,
            MarkdownAuthoringOp::Heading1,
            true,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_heading2,
            MarkdownAuthoringOp::Heading2,
            true,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_heading3,
            MarkdownAuthoringOp::Heading3,
            true,
        );
    }

    fn render_blocks(ui: &mut egui::Ui, action: &mut AppAction) {
        let s = &crate::i18n::I18nOps::get().search;
        Self::author_button(
            ui,
            action,
            &s.command_author_bullet_list,
            MarkdownAuthoringOp::BulletList,
            true,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_numbered_list,
            MarkdownAuthoringOp::NumberedList,
            true,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_blockquote,
            MarkdownAuthoringOp::Blockquote,
            true,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_code_block,
            MarkdownAuthoringOp::CodeBlock,
            true,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_horizontal_rule,
            MarkdownAuthoringOp::HorizontalRule,
            true,
        );
    }

    fn render_insert(ui: &mut egui::Ui, action: &mut AppAction) {
        let s = &crate::i18n::I18nOps::get().search;
        Self::author_button(
            ui,
            action,
            &s.command_author_insert_link,
            MarkdownAuthoringOp::InsertLink,
            true,
        );
        Self::author_button(
            ui,
            action,
            &s.command_author_insert_table,
            MarkdownAuthoringOp::InsertTable,
            true,
        );
    }

    fn render_image_ingest(ui: &mut egui::Ui, action: &mut AppAction) {
        let s = &crate::i18n::I18nOps::get().search;
        if ui.button(&s.command_ingest_image_file).clicked() {
            *action = AppAction::IngestImageFile;
            ui.close();
        }
        if ui.button(&s.command_ingest_clipboard_image).clicked() {
            *action = AppAction::IngestClipboardImage;
            ui.close();
        }
    }

    fn author_button(
        ui: &mut egui::Ui,
        action: &mut AppAction,
        label: &str,
        op: MarkdownAuthoringOp,
        enabled: bool,
    ) {
        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
            *action = AppAction::AuthorMarkdown(op);
            ui.close();
        }
    }
}
