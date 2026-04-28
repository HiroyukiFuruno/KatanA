pub use crate::state::command_palette::CommandPaletteState;
pub use crate::state::config::{ConfigState, SettingsSection, SettingsTab};
pub use crate::state::diagnostics::DiagnosticsState;
pub use crate::state::document::{
    DocumentState, SplitViewState, TabSplitState, TabViewMode, ViewMode,
};
pub use crate::state::layout::{DiffReviewSnapshot, LayoutState};
pub use crate::state::scroll::{ScrollSource, ScrollState};
pub use crate::state::search::{SearchState, SearchTab};
pub use crate::state::update::{UpdatePhase, UpdateState};
pub use crate::state::workspace::WorkspaceState;

pub use katana_platform::CacheFacade;

use katana_core::{ai::AiProviderRegistry, document::Document, plugin::PluginRegistry};
use katana_platform::SettingsService;
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExportFormat {
    Html,
    Pdf,
    Png,
    Jpg,
}

pub use crate::app_action::AppAction;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum StatusType {
    Info,
    Success,
    Warning,
    Error,
}

pub struct AppState {
    pub document: DocumentState,
    pub workspace: WorkspaceState,
    pub layout: LayoutState,
    pub search: SearchState,
    pub scroll: ScrollState,
    pub update: UpdateState,
    pub config: ConfigState,
    pub diagnostics: DiagnosticsState,
    pub command_palette: CommandPaletteState,
    pub global_workspace: katana_platform::workspace::GlobalWorkspaceService,
    pub active_toc_index: Option<usize>,
}

impl AppState {
    pub fn new(
        ai_registry: AiProviderRegistry,
        plugin_registry: PluginRegistry,
        settings: SettingsService,
        cache: std::sync::Arc<dyn katana_platform::CacheFacade>,
    ) -> Self {
        let _ = ai_registry;
        let mut search = SearchState::new();
        search.md_history.recent_terms = settings.settings().search.recent_md_queries.clone();

        let mut layout = LayoutState::new();
        layout.slideshow_hover_highlight = settings.settings().behavior.slideshow_hover_highlight;
        layout.slideshow_show_diagram_controls =
            settings.settings().behavior.slideshow_show_diagram_controls;
        /* WHY: Apply settings-driven defaults for panel pin state.
         * These are read-only initial values — the user's runtime toggles are not persisted. */
        layout.show_toc = settings.settings().layout.toc_default_visible;
        layout.show_explorer = settings.settings().layout.explorer_default_visible;

        Self {
            document: DocumentState::new(),
            workspace: WorkspaceState::new(),
            layout,
            search,
            scroll: ScrollState::new(),
            update: UpdateState::new(),
            config: ConfigState::new(plugin_registry, settings, cache),
            diagnostics: DiagnosticsState::new(),
            command_palette: CommandPaletteState::new(),
            global_workspace: katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
                katana_platform::workspace::JsonWorkspaceRepository::with_default_path(),
            )),
            active_toc_index: None,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.active_document().map(|d| d.is_dirty).unwrap_or(false)
    }

    pub fn active_document(&self) -> Option<&Document> {
        self.document
            .active_doc_idx
            .and_then(|idx| self.document.open_documents.get(idx))
    }

    pub fn active_document_mut(&mut self) -> Option<&mut Document> {
        self.document
            .active_doc_idx
            .and_then(|idx| self.document.open_documents.get_mut(idx))
    }

    pub fn active_path(&self) -> Option<std::path::PathBuf> {
        self.active_document().map(|d| d.path.clone())
    }

    pub fn active_view_mode(&self) -> ViewMode {
        self.active_document()
            .and_then(|doc| {
                self.document
                    .tab_view_modes
                    .iter()
                    .find(|t| t.path == doc.path)
                    .map(|t| t.mode)
            })
            .unwrap_or_else(|| {
                self.active_document()
                    .map(Self::default_view_mode_for_document)
                    .unwrap_or(ViewMode::PreviewOnly)
            })
    }

    fn default_view_mode_for_document(_doc: &Document) -> ViewMode {
        ViewMode::PreviewOnly
    }
}
