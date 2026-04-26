use crate::app_action::{AppAction, MarkdownAuthoringOp};
use crate::i18n::I18nOps;
use crate::icon::{Icon, IconSize};
use crate::widgets::{AlignCenter, MenuButtonOps};
use egui::Ui;

const TOOLBAR_SEPARATOR_TEXT: &str = "|";
const TOOLBAR_SEPARATOR_WIDTH: f32 = 6.0;

pub(crate) struct EditorToolbar<'a> {
    action: &'a mut AppAction,
    has_selection: bool,
}

impl<'a> EditorToolbar<'a> {
    pub(crate) fn new(action: &'a mut AppAction, has_selection: bool) -> Self {
        Self {
            action,
            has_selection,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let action = &mut *self.action;
        let has_selection = self.has_selection;
        AlignCenter::new()
            .shrink_to_fit(true)
            .content(|ui| {
                Self::inline_group(ui, action, has_selection);
                Self::separator(ui);
                Self::heading_group(ui, action);
                Self::separator(ui);
                Self::list_group(ui, action);
                Self::separator(ui);
                Self::block_group(ui, action);
                Self::separator(ui);
                Self::image_group(ui, action);
            })
            .show(ui);
    }

    fn separator(ui: &mut Ui) {
        let size = egui::vec2(TOOLBAR_SEPARATOR_WIDTH, ui.spacing().interact_size.y);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            TOOLBAR_SEPARATOR_TEXT,
            egui::TextStyle::Button.resolve(ui.style()),
            ui.visuals().weak_text_color(),
        );
    }

    fn inline_group(ui: &mut Ui, action: &mut AppAction, has_sel: bool) {
        if ui
            .add_enabled(has_sel, Icon::Bold.button(ui, IconSize::Small))
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::Bold);
        }

        if ui
            .add_enabled(has_sel, Icon::Italic.button(ui, IconSize::Small))
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::Italic);
        }

        if ui
            .add_enabled(has_sel, Icon::Strikethrough.button(ui, IconSize::Small))
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::Strikethrough);
        }

        if ui
            .add_enabled(has_sel, Icon::Code.button(ui, IconSize::Small))
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::InlineCode);
        }
    }

    fn heading_group(ui: &mut Ui, action: &mut AppAction) {
        if ui
            .add(Icon::Heading.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().editor.toolbar.heading1.clone())
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading1);
        }

        if ui
            .add(Icon::Heading.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().editor.toolbar.heading2.clone())
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading2);
        }

        if ui
            .add(Icon::Heading.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().editor.toolbar.heading3.clone())
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading3);
        }
    }

    fn list_group(ui: &mut Ui, action: &mut AppAction) {
        if ui
            .add(Icon::List.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().editor.toolbar.bullet_list.clone())
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::BulletList);
        }

        if ui
            .add(Icon::ListOrdered.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().editor.toolbar.numbered_list.clone())
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::NumberedList);
        }

        if ui
            .add(Icon::Quote.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().editor.toolbar.blockquote.clone())
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::Blockquote);
        }
    }

    fn block_group(ui: &mut Ui, action: &mut AppAction) {
        let label = I18nOps::get().editor.toolbar.code_block.clone();
        MenuButtonOps::show(ui, label, |ui| {
            super::code_block_menu::CodeBlockMenuOps::show(ui, action);
        });
    }

    fn image_group(ui: &mut Ui, action: &mut AppAction) {
        if ui
            .add(Icon::Image.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().search.command_ingest_image_file.clone())
            .clicked()
        {
            *action = AppAction::IngestImageFile;
        }
    }
}
