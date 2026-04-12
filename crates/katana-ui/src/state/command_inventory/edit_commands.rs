use super::{CommandGroup, CommandInventoryItem};
use crate::app_action::MarkdownAuthoringOp;
use crate::app_state::AppAction;
use crate::i18n::I18nOps;

/* WHY: Helper — true only when the active document is an editable (non-reference) Markdown file. */
fn is_active_editable_markdown(state: &crate::app_state::AppState) -> bool {
    state
        .active_document()
        .is_some_and(|d| !d.is_reference && !d.path.to_string_lossy().starts_with("Katana://"))
}

pub struct EditCommands;

impl EditCommands {
    pub fn get() -> Vec<CommandInventoryItem> {
        vec![
            /* WHY: Edit Group — Markdown authoring commands */
            CommandInventoryItem {
                id: "edit.bold",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Bold),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_bold.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+B[editor]"],
            },
            CommandInventoryItem {
                id: "edit.italic",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Italic),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_italic.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+I[editor]"],
            },
            CommandInventoryItem {
                id: "edit.strikethrough",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Strikethrough),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_strikethrough.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.inline_code",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::InlineCode),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_inline_code.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+`[editor]"],
            },
            CommandInventoryItem {
                id: "edit.heading1",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading1),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_heading1.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.heading2",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading2),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_heading2.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.heading3",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading3),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_heading3.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.bullet_list",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::BulletList),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_bullet_list.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.numbered_list",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::NumberedList),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_numbered_list.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.blockquote",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Blockquote),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_blockquote.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.code_block",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::CodeBlock),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_code_block.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.horizontal_rule",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::HorizontalRule),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_horizontal_rule.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.insert_link",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::InsertLink),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_insert_link.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+K[editor]"],
            },
            CommandInventoryItem {
                id: "edit.insert_table",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::InsertTable),
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_author_insert_table.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            /* WHY: Image ingest commands — also Edit group since they mutate the document. */
            CommandInventoryItem {
                id: "edit.ingest_image_file",
                action: AppAction::IngestImageFile,
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_ingest_image_file.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.ingest_clipboard_image",
                action: AppAction::IngestClipboardImage,
                group: CommandGroup::Edit,
                label: || I18nOps::get().search.command_ingest_clipboard_image.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
        ]
    }
}
