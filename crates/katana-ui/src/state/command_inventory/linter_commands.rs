use super::{CommandGroup, CommandInventoryItem};
use crate::app_state::AppAction;
use crate::state::shortcut_context::ShortcutContext;

pub struct LinterCommands;

macro_rules! linter_cmd {
    ($id:ident, $code:expr, $name:expr) => {
        CommandInventoryItem {
            id: concat!("linter.toggle.", $code),
            action: AppAction::ToggleLintRule($code.to_string()),
            group: CommandGroup::Behavior,
            context: ShortcutContext::Global,
            label: || {
                crate::i18n::I18nOps::get()
                    .linter
                    .rule_toggle
                    .replace("{rule_code}", $code)
                    .replace("{rule_name}", $name)
            },
            is_available: |_| true,
            default_shortcuts: &[],
        }
    };
}

impl LinterCommands {
    pub fn get() -> Vec<CommandInventoryItem> {
        vec![
            linter_cmd!(Cmd001, "MD001", "heading-increment"),
            linter_cmd!(Cmd003, "MD003", "heading-style"),
            linter_cmd!(Cmd004, "MD004", "ul-style"),
            linter_cmd!(Cmd005, "MD005", "list-indent"),
            linter_cmd!(Cmd007, "MD007", "ul-indent"),
            linter_cmd!(Cmd009, "MD009", "trailing-spaces"),
            linter_cmd!(Cmd010, "MD010", "hard-tabs"),
            linter_cmd!(Cmd011, "MD011", "reversed-links"),
            linter_cmd!(Cmd012, "MD012", "multiple-blanks"),
            linter_cmd!(Cmd013, "MD013", "line-length"),
            linter_cmd!(Cmd014, "MD014", "commands-show-output"),
            linter_cmd!(Cmd018, "MD018", "no-missing-space-atx"),
            linter_cmd!(Cmd019, "MD019", "no-multiple-space-atx"),
            linter_cmd!(Cmd020, "MD020", "no-missing-space-closed-atx"),
            linter_cmd!(Cmd021, "MD021", "no-multiple-space-closed-atx"),
            linter_cmd!(Cmd022, "MD022", "blanks-around-headings"),
            linter_cmd!(Cmd023, "MD023", "heading-start-left"),
            linter_cmd!(Cmd024, "MD024", "no-duplicate-heading"),
            linter_cmd!(Cmd025, "MD025", "single-title"),
            linter_cmd!(Cmd026, "MD026", "no-trailing-punctuation"),
            linter_cmd!(Cmd027, "MD027", "no-multiple-space-blockquote"),
            linter_cmd!(Cmd028, "MD028", "no-blanks-blockquote"),
            linter_cmd!(Cmd029, "MD029", "ol-prefix"),
            linter_cmd!(Cmd030, "MD030", "list-marker-space"),
            linter_cmd!(Cmd031, "MD031", "blanks-around-lists"),
            linter_cmd!(Cmd032, "MD032", "blanks-around-lists"),
            linter_cmd!(Cmd033, "MD033", "no-inline-html"),
            linter_cmd!(Cmd034, "MD034", "no-bare-urls"),
            linter_cmd!(Cmd035, "MD035", "hr-style"),
            linter_cmd!(Cmd036, "MD036", "no-emphasis-as-heading"),
            linter_cmd!(Cmd037, "MD037", "no-space-in-emphasis"),
            linter_cmd!(Cmd038, "MD038", "no-space-in-code"),
            linter_cmd!(Cmd039, "MD039", "no-space-in-links"),
            linter_cmd!(Cmd040, "MD040", "fenced-code-language"),
            linter_cmd!(Cmd041, "MD041", "first-line-heading"),
            linter_cmd!(Cmd042, "MD042", "no-empty-links"),
            linter_cmd!(Cmd043, "MD043", "required-headings"),
            linter_cmd!(Cmd044, "MD044", "proper-names"),
            linter_cmd!(Cmd045, "MD045", "no-alt-text"),
            linter_cmd!(Cmd046, "MD046", "code-block-style"),
            linter_cmd!(Cmd047, "MD047", "single-trailing-newline"),
            linter_cmd!(Cmd048, "MD048", "code-fence-style"),
            linter_cmd!(Cmd049, "MD049", "emphasis-style"),
            linter_cmd!(Cmd050, "MD050", "strong-style"),
            linter_cmd!(Cmd051, "MD051", "link-fragments"),
            linter_cmd!(Cmd052, "MD052", "reference-links-images"),
            linter_cmd!(Cmd053, "MD053", "link-image-reference-definitions"),
            linter_cmd!(Cmd054, "MD054", "link-image-style"),
            linter_cmd!(Cmd055, "MD055", "table-pipe-style"),
            linter_cmd!(Cmd056, "MD056", "table-column-count"),
            linter_cmd!(Cmd058, "MD058", "table-sync"),
            linter_cmd!(Cmd059, "MD059", "table-sync"),
            linter_cmd!(Cmd060, "MD060", "table-sync"),
        ]
    }
}
