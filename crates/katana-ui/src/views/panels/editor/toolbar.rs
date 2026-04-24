use crate::app_action::{AppAction, MarkdownAuthoringOp};
use crate::i18n::I18nOps;
use crate::icon::{Icon, IconSize};
use crate::widgets::AlignCenter;
use egui::Ui;

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

    pub(crate) fn render(
        ui: &mut Ui,
        action: &'a mut AppAction,
        cursor_range: &Option<egui::text::CCursorRange>,
    ) {
        let has_sel = cursor_range
            .as_ref()
            .map(|r| r.primary.index != r.secondary.index)
            .unwrap_or(false);
        Self::new(action, has_sel).show(ui);
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let action = &mut *self.action;
        let has_selection = self.has_selection;
        AlignCenter::new()
            .content(|ui| {
                Self::inline_group(ui, action, has_selection);
                ui.separator();
                Self::heading_group(ui, action);
                ui.separator();
                Self::list_group(ui, action);
                ui.separator();
                Self::block_group(ui, action);
            })
            .show(ui);
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
        if ui
            .add(Icon::Code.button(ui, IconSize::Small))
            .on_hover_text(I18nOps::get().editor.toolbar.code_block.clone())
            .clicked()
        {
            *action = AppAction::AuthorMarkdown(MarkdownAuthoringOp::CodeBlock);
        }
    }
}
