use super::{CommandGroup, CommandInventoryItem};
use crate::app_action::MarkdownAuthoringOp;
use crate::app_state::AppAction;
use crate::i18n::I18nOps;
use crate::state::shortcut_context::ShortcutContext;

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
            /* WHY: Edit Group — Markdown authoring commands.
            All edit commands use ShortcutContext::Editor so they only fire
            when the text editor pane has keyboard focus, preventing conflicts
            with Global shortcuts that share the same key combinations. */
            CommandInventoryItem {
                id: "edit.bold",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Bold),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_bold.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+Shift+B"],
            },
            CommandInventoryItem {
                id: "edit.italic",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Italic),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_italic.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+I"],
            },
            CommandInventoryItem {
                id: "edit.strikethrough",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Strikethrough),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_strikethrough.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+Shift+X"],
            },
            CommandInventoryItem {
                id: "edit.inline_code",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::InlineCode),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_inline_code.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+`"],
            },
            CommandInventoryItem {
                id: "edit.heading1",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading1),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_heading1.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+1"],
            },
            CommandInventoryItem {
                id: "edit.heading2",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading2),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_heading2.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+2"],
            },
            CommandInventoryItem {
                id: "edit.heading3",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Heading3),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_heading3.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+3"],
            },
            CommandInventoryItem {
                id: "edit.bullet_list",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::BulletList),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_bullet_list.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+Shift+8"],
            },
            CommandInventoryItem {
                id: "edit.numbered_list",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::NumberedList),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_numbered_list.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+Shift+7"],
            },
            CommandInventoryItem {
                id: "edit.blockquote",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::Blockquote),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_blockquote.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+Shift+9"],
            },
            CommandInventoryItem {
                id: "edit.code_block",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::CodeBlock),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_code_block.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+Shift+C"],
            },
            CommandInventoryItem {
                id: "edit.horizontal_rule",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::HorizontalRule),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_horizontal_rule.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+Shift+H"],
            },
            CommandInventoryItem {
                id: "edit.insert_link",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::InsertLink),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_insert_link.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+K"],
            },
            CommandInventoryItem {
                id: "edit.insert_table",
                action: AppAction::AuthorMarkdown(MarkdownAuthoringOp::InsertTable),
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_author_insert_table.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &["primary+alt+T"],
            },
            /* WHY: Image ingest commands — also Edit group since they mutate the document. */
            CommandInventoryItem {
                id: "edit.ingest_image_file",
                action: AppAction::IngestImageFile,
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_ingest_image_file.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
            CommandInventoryItem {
                id: "edit.ingest_clipboard_image",
                action: AppAction::IngestClipboardImage,
                group: CommandGroup::Edit,
                context: ShortcutContext::Editor,
                label: || I18nOps::get().search.command_ingest_clipboard_image.clone(),
                is_available: |state| is_active_editable_markdown(state),
                default_shortcuts: &[],
            },
        ]
    }
}
