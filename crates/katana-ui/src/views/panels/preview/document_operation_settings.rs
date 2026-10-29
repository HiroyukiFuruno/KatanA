use crate::app_state::{AppAction, ViewMode};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreviewMenu {
    Refresh,
    Search,
    Toc,
    Export,
    Story,
    Tools,
    Info,
    Slideshow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DocumentOperation {
    PreviewMenu(PreviewMenu),
    ViewMode(ViewMode),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DocumentOperationSettings {
    refresh: bool,
    search: bool,
    toc: bool,
    export: bool,
    story: bool,
    tools: bool,
    info: bool,
    code_only: bool,
    split: bool,
    slideshow: bool,
}

impl DocumentOperationSettings {
    pub(crate) fn evaluate(path: Option<&Path>, pptx_ready: Option<bool>) -> Self {
        let Some(path) = path else {
            return Self::empty_workspace();
        };
        let format = katana_core::document_source::BinaryDocumentFormat::from_path(path);
        let is_pptx = matches!(
            format,
            Some(katana_core::document_source::BinaryDocumentFormat::Pptx)
        );
        let static_settings = Self::for_format(path, format);
        Self {
            story: static_settings.story || (is_pptx && pptx_ready.unwrap_or(false)),
            slideshow: if is_pptx {
                pptx_ready.unwrap_or(true)
            } else {
                static_settings.slideshow
            },
            ..static_settings
        }
    }

    fn empty_workspace() -> Self {
        /* WHY: 空ワークスペースでも従来どおり各操作を選べる状態を維持する。 */
        Self {
            refresh: true,
            search: true,
            toc: true,
            export: true,
            story: true,
            tools: true,
            info: true,
            code_only: true,
            split: true,
            slideshow: false,
        }
    }

    fn for_format(
        path: &Path,
        format: Option<katana_core::document_source::BinaryDocumentFormat>,
    ) -> Self {
        let is_office = matches!(
            format,
            Some(
                katana_core::document_source::BinaryDocumentFormat::Docx
                    | katana_core::document_source::BinaryDocumentFormat::Xlsx
                    | katana_core::document_source::BinaryDocumentFormat::Pptx
            )
        );
        let document_menus = supports_document_menus(path, is_office);
        let text_document = document_menus && format.is_none();
        Self {
            refresh: true,
            search: !is_virtual_document(path),
            toc: document_menus,
            export: text_document,
            story: text_document,
            tools: document_menus,
            info: true,
            code_only: text_document,
            split: text_document,
            slideshow: text_document,
        }
    }

    pub(crate) fn allows(self, operation: DocumentOperation) -> bool {
        match operation {
            DocumentOperation::PreviewMenu(menu) => match menu {
                PreviewMenu::Refresh => self.refresh,
                PreviewMenu::Search => self.search,
                PreviewMenu::Toc => self.toc,
                PreviewMenu::Export => self.export,
                PreviewMenu::Story => self.story,
                PreviewMenu::Tools => self.tools,
                PreviewMenu::Info => self.info,
                PreviewMenu::Slideshow => self.slideshow,
            },
            DocumentOperation::ViewMode(mode) => match mode {
                ViewMode::PreviewOnly => true,
                ViewMode::CodeOnly => self.code_only,
                ViewMode::Split => self.split,
            },
        }
    }

    pub(crate) fn allows_action(self, action: &AppAction, current_mode: ViewMode) -> Option<bool> {
        Self::operation_for_action(action, current_mode).map(|operation| self.allows(operation))
    }

    fn operation_for_action(
        action: &AppAction,
        current_mode: ViewMode,
    ) -> Option<DocumentOperation> {
        match action {
            AppAction::SetViewMode(mode) => Some(DocumentOperation::ViewMode(*mode)),
            AppAction::ToggleSplitMode => Some(DocumentOperation::ViewMode(ViewMode::Split)),
            AppAction::SetSplitDirection(_)
            | AppAction::SetPaneOrder(_)
            | AppAction::ToggleScrollSync(_) => Some(DocumentOperation::ViewMode(ViewMode::Split)),
            AppAction::ToggleCodePreview => {
                let mode = match current_mode {
                    ViewMode::Split | ViewMode::CodeOnly => ViewMode::PreviewOnly,
                    ViewMode::PreviewOnly => ViewMode::CodeOnly,
                };
                Some(DocumentOperation::ViewMode(mode))
            }
            _ => Self::sidebar_operation_for_action(action),
        }
    }

    fn sidebar_operation_for_action(action: &AppAction) -> Option<DocumentOperation> {
        let menu = match action {
            AppAction::RefreshDocument { .. } => PreviewMenu::Refresh,
            AppAction::OpenDocSearch | AppAction::ToggleDocSearch => PreviewMenu::Search,
            AppAction::ShowMetaInfo(_) => PreviewMenu::Info,
            AppAction::ExportDocument(_) => PreviewMenu::Export,
            AppAction::ToggleToc => PreviewMenu::Toc,
            AppAction::ToggleExportPanel => PreviewMenu::Export,
            AppAction::ToggleStoryPanel
            | AppAction::ToggleSlideshowHoverHighlight
            | AppAction::ToggleSlideshowShowDiagramControls => PreviewMenu::Story,
            AppAction::ToggleToolsPanel => PreviewMenu::Tools,
            AppAction::ToggleSlideshow => PreviewMenu::Slideshow,
            _ => return None,
        };
        Some(DocumentOperation::PreviewMenu(menu))
    }
}

fn supports_document_menus(path: &Path, is_office: bool) -> bool {
    !katana_core::workspace::TreeEntry::path_is_canvas(path)
        && !katana_core::workspace::TreeEntry::path_is_image(path)
        && !is_html_document(path)
        && !is_office
}

fn is_html_document(path: &Path) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
        })
}

fn is_virtual_document(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path.starts_with("Katana://Welcome")
        || path.starts_with("Katana://Guide")
        || path.starts_with("Katana://ChangeLog")
}
