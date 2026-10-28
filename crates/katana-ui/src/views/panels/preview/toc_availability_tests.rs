use super::{DocumentOperation, DocumentOperationSettings, PreviewMenu, PreviewMenuAvailability};
use crate::app_state::{AppAction, AppState, DocumentOperationRuntimeSnapshot, ViewMode};
use katana_core::{ai::AiProviderRegistry, plugin::PluginRegistry};
use std::path::{Path, PathBuf};

#[test]
fn operation_settings_cover_preview_menus_and_modes_by_format() {
    for (path, document, text) in [
        ("notes.md", true, true),
        ("manual.pdf", true, false),
        ("report.docx", false, false),
        ("book.xlsx", false, false),
        ("deck.pptx", false, false),
        ("page.html", false, false),
        ("photo.png", false, false),
        ("board.canvas", false, false),
        ("notes.txt", true, true),
    ] {
        assert_format_settings(path, document, text);
    }
}

fn assert_format_settings(path: &str, document: bool, text: bool) {
    let settings = DocumentOperationSettings::evaluate(Some(Path::new(path)), Some(false));
    for (menu, expected) in [
        (PreviewMenu::Refresh, true),
        (PreviewMenu::Search, true),
        (PreviewMenu::Toc, document),
        (PreviewMenu::Export, text),
        (PreviewMenu::Story, text),
        (PreviewMenu::Tools, document),
        (PreviewMenu::Info, true),
        (PreviewMenu::Slideshow, text),
    ] {
        assert_eq!(
            settings.allows(DocumentOperation::PreviewMenu(menu)),
            expected,
            "{path} {menu:?}"
        );
    }
    for (mode, expected) in [
        (ViewMode::PreviewOnly, true),
        (ViewMode::CodeOnly, text),
        (ViewMode::Split, text),
    ] {
        assert_eq!(
            settings.allows(DocumentOperation::ViewMode(mode)),
            expected,
            "{path}"
        );
    }
}

#[test]
fn pptx_slideshow_requires_ready_runtime_but_static_query_reports_format_support() {
    let path = Path::new("deck.pptx");
    for ready in [false, true] {
        let settings = DocumentOperationSettings::evaluate(Some(path), Some(ready));
        assert_eq!(
            settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Slideshow)),
            ready
        );
        assert_eq!(
            settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Story)),
            ready
        );
    }
    assert!(PreviewMenuAvailability::for_path(
        Some(path),
        PreviewMenu::Slideshow
    ));
}

#[test]
fn stale_same_path_reopen_identity_cannot_reuse_pptx_readiness() {
    let path = Path::new("deck.pptx");
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let mut reopened = katana_core::document::Document::new_empty(path);
    reopened.revision = 4;
    state.document.open_documents.push(reopened);
    state.document.active_doc_idx = Some(0);

    let mut pane = crate::preview_pane::PreviewPane::default();
    pane.full_render_document_path(path, false);
    let current_preview_generation = pane.session_generation;

    /* WHY: 同じパスとタブを開き直したとき、旧リビジョンの準備完了状態を引き継がない。 */
    state.document_operation_runtime = DocumentOperationRuntimeSnapshot {
        active_document_path: Some(PathBuf::from(path)),
        active_document_revision: Some(3),
        active_doc_idx: Some(0),
        preview_generation: Some(current_preview_generation),
        pptx_slideshow_ready: true,
        ..Default::default()
    };
    assert!(
        !PreviewMenuAvailability::settings_for_state(&state)
            .allows(DocumentOperation::PreviewMenu(PreviewMenu::Slideshow))
    );

    /* WHY: 現在の識別情報なら本番の共有ポリシーが準備完了状態を許可する。 */
    state.document_operation_runtime.active_document_revision = Some(4);
    assert!(
        PreviewMenuAvailability::settings_for_state(&state)
            .allows(DocumentOperation::PreviewMenu(PreviewMenu::Slideshow))
    );
    state.document_operation_runtime.active_document_path = Some(PathBuf::from("other.pptx"));
    assert!(
        !PreviewMenuAvailability::settings_for_state(&state)
            .allows(DocumentOperation::PreviewMenu(PreviewMenu::Slideshow))
    );
    state.document_operation_runtime.active_document_path = Some(PathBuf::from(path));
    state.document_operation_runtime.active_doc_idx = Some(1);
    assert!(
        !PreviewMenuAvailability::settings_for_state(&state)
            .allows(DocumentOperation::PreviewMenu(PreviewMenu::Slideshow))
    );
    state.document_operation_runtime.active_doc_idx = Some(0);
    state.document_operation_runtime.preview_generation = None;
    assert!(
        !PreviewMenuAvailability::settings_for_state(&state)
            .allows(DocumentOperation::PreviewMenu(PreviewMenu::Slideshow))
    );
}

#[test]
fn no_active_document_disables_preview_menus_and_code_modes() {
    let settings = DocumentOperationSettings::evaluate(None, Some(false));
    assert!(settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Toc)));
    assert!(settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Tools)));
    assert!(!settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Slideshow)));
    assert!(settings.allows(DocumentOperation::ViewMode(ViewMode::PreviewOnly)));
    assert!(settings.allows(DocumentOperation::ViewMode(ViewMode::CodeOnly)));
    assert!(settings.allows(DocumentOperation::ViewMode(ViewMode::Split)));
}

#[test]
fn code_modes_are_not_inferred_from_tools_and_recovery_to_preview_is_safe() {
    let pdf = DocumentOperationSettings::evaluate(Some(Path::new("manual.pdf")), Some(false));
    assert!(pdf.allows(DocumentOperation::PreviewMenu(PreviewMenu::Tools)));
    assert!(!pdf.allows(DocumentOperation::ViewMode(ViewMode::CodeOnly)));
    assert!(!pdf.allows(DocumentOperation::ViewMode(ViewMode::Split)));
    assert!(
        pdf.allows_action(&AppAction::ToggleCodePreview, ViewMode::CodeOnly)
            .unwrap()
    );
    assert!(
        !pdf.allows_action(&AppAction::ToggleCodePreview, ViewMode::PreviewOnly)
            .unwrap()
    );
}

#[test]
fn virtual_documents_keep_refresh_and_info_but_hide_search() {
    let settings =
        DocumentOperationSettings::evaluate(Some(Path::new("Katana://Welcome")), Some(false));
    assert!(settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Refresh)));
    assert!(!settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Search)));
    assert!(settings.allows(DocumentOperation::PreviewMenu(PreviewMenu::Info)));
}

#[test]
fn toc_compatibility_uses_the_shared_settings() {
    assert!(PreviewMenuAvailability::for_path(
        Some(Path::new("notes.md")),
        PreviewMenu::Toc
    ));
    assert!(!PreviewMenuAvailability::for_path(
        Some(Path::new("deck.pptx")),
        PreviewMenu::Toc
    ));
}
