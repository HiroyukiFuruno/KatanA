use serde::{Deserialize, Serialize};

pub mod action;
pub mod chat;
pub mod common;
pub mod dashboard;
pub mod error;
pub mod linter;
pub mod menu;
pub mod meta;
pub mod preview;
pub mod search;
pub mod settings;
pub mod settings_ai;
pub mod settings_color;
pub mod status;
pub mod tab;
pub mod workspace;

pub use action::*;
pub use chat::*;
pub use common::*;
pub use dashboard::*;
pub use error::*;
pub use linter::*;
pub use menu::*;
pub use meta::*;
pub use preview::*;
pub use search::*;
pub use settings::*;
pub use settings_ai::*;
pub use status::*;
pub use tab::*;
pub use workspace::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nMessages {
    pub menu: MenuMessages,
    pub about: AboutMessages,
    pub update: UpdateMessages,
    pub workspace: WorkspaceMessages,
    pub preview: PreviewMessages,
    pub plantuml: PlantUmlMessages,
    pub view_mode: ViewModeMessages,
    pub split_toggle: SplitToggleMessages,
    pub error: ErrorMessages,
    pub status: StatusMessages,
    pub action: ActionMessages,
    pub ai: AiMessages,
    #[serde(default)]
    pub chat: ChatMessages,
    pub tool: ToolMessages,
    pub settings: SettingsMessages,
    pub tab: TabMessages,
    pub search: SearchMessages,
    pub toc: TocMessages,
    pub export: ExportMessages,
    pub terms: TermsMessages,
    pub dialog: DialogMessages,
    pub markdown: MarkdownMessages,
    pub common: CommonMessages,
    pub meta_info: MetaInfoMessages,
    pub help: HelpMessages,
    pub dashboard: DashboardMessages,
    #[serde(default)]
    pub linter: LinterTranslations,
}
