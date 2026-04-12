use crate::app_state::{AppAction, AppState};
use crate::preview_pane::PreviewPane;
use eframe::egui;
use katana_platform::FilesystemService;
use katana_platform::theme::ThemeColors;

pub(crate) struct TabPreviewCache {
    pub path: std::path::PathBuf,
    pub pane: PreviewPane,
    pub hash: u64,
}

pub(crate) enum ExplorerLoadType {
    Open,
    Refresh,
}

pub(crate) type ExplorerLoadResult =
    Result<katana_core::workspace::Workspace, katana_core::workspace::WorkspaceError>;
pub(crate) type ExplorerLoadMessage = (ExplorerLoadType, std::path::PathBuf, ExplorerLoadResult);

pub(crate) struct ExportTask {
    pub filename: String,
    pub rx: std::sync::mpsc::Receiver<Result<std::path::PathBuf, String>>,
    pub open_on_complete: bool,
}

pub enum UpdateInstallEvent {
    Progress(katana_core::update::UpdateProgress),
    Finished(Result<katana_core::update::UpdatePreparation, String>),
}

pub struct KatanaApp {
    pub(crate) state: AppState,
    pub(crate) fs: FilesystemService,
    pub(crate) pending_action: AppAction,
    pub(crate) tab_previews: Vec<TabPreviewCache>,
    pub(crate) download_rx: Option<std::sync::mpsc::Receiver<Result<(), String>>>,
    pub(crate) explorer_rx: Option<std::sync::mpsc::Receiver<ExplorerLoadMessage>>,
    pub(crate) update_rx:
        Option<std::sync::mpsc::Receiver<Result<Option<katana_core::update::ReleaseInfo>, String>>>,
    pub(crate) changelog_rx: Option<std::sync::mpsc::Receiver<crate::changelog::ChangelogEvent>>,
    pub(crate) update_install_rx: Option<std::sync::mpsc::Receiver<UpdateInstallEvent>>,
    pub(crate) export_tasks: Vec<ExportTask>,
    pub(crate) pending_document_loads: std::collections::VecDeque<std::path::PathBuf>,

    pub(crate) show_about: bool,
    pub(crate) show_update_dialog: bool,
    pub(crate) update_markdown_cache: egui_commonmark::CommonMarkCache,
    pub(crate) update_notified: bool,
    pub about_icon: Option<egui::TextureHandle>,
    pub(crate) cached_theme: Option<ThemeColors>,
    pub(crate) cached_font_size: Option<f32>,
    pub(crate) cached_font_family: Option<String>,
    pub(crate) settings_preview: PreviewPane,
    pub(crate) needs_splash: bool,
    pub(crate) splash_start: Option<std::time::Instant>,
    pub(crate) show_meta_info_for: Option<std::path::PathBuf>,
    pub(crate) pending_relaunch: Option<katana_core::update::UpdatePreparation>,
    pub(crate) changelog_sections: Vec<crate::changelog::ChangelogSection>,
    pub(crate) needs_changelog_display: bool,
    pub(crate) old_app_version: Option<String>,
    /* WHY: Authoring — last known TextEdit cursor range (char-index based).
    Updated each frame by EditorContent before an action is dispatched. */
    pub(crate) editor_cursor_range: Option<egui::text::CCursorRange>,
    /* WHY: Authoring — pending cursor to restore after a buffer transform.
    Set by handle_action_author_markdown; consumed by EditorContent on the next frame. */
    pub(crate) pending_editor_cursor: Option<(usize, usize)>,
}
