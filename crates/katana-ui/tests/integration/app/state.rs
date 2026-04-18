use katana_core::ai::AiProviderRegistry;
use katana_core::document::Document;
use katana_core::plugin::PluginRegistry;
use katana_ui::app_state::*;

#[test]
fn new_app_state_is_empty_and_default_view() {
    /* WHY: Verify that a freshly initialized AppState has no workspace and zero open documents. */
    let state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert!(state.workspace.data.is_none());
    assert!(state.document.open_documents.is_empty());
    assert_eq!(state.document.active_doc_idx, None);
    assert_eq!(state.active_view_mode(), ViewMode::PreviewOnly);
}

#[test]
fn active_document_returns_correct_doc_when_set() {
    /* WHY: Verify that active_document() correctly indexes into the open_documents list based on active_doc_idx. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let doc1 = Document::new("doc1.md", "Doc1");
    let doc2 = Document::new("doc2.md", "Doc2");

    state.document.open_documents.push(doc1);
    state.document.open_documents.push(doc2);

    state.document.active_doc_idx = Some(1);
    assert_eq!(state.active_document().unwrap().buffer, "Doc2");

    state.document.active_doc_idx = Some(0);
    assert_eq!(state.active_document().unwrap().buffer, "Doc1");

    state.document.active_doc_idx = Some(999);
    assert!(state.active_document().is_none());
}

#[test]
fn is_dirty_reflects_active_document_state() {
    /* WHY: Verify that the top-level is_dirty() check correctly reports the dirty status of the currently active document. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    assert!(!state.is_dirty());

    let mut doc = Document::new("test.md", "test");
    doc.is_dirty = true;
    state.document.open_documents.push(doc);
    state.document.active_doc_idx = Some(0);

    assert!(state.is_dirty());
}

#[test]
fn active_document_mut_returns_correct_mut_doc() {
    /* WHY: Verify that active_document_mut() allows in-place modification of the active document's buffer. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let doc1 = Document::new("doc1.md", "Doc1");
    state.document.open_documents.push(doc1);
    state.document.active_doc_idx = Some(0);

    if let Some(mut_doc) = state.active_document_mut() {
        mut_doc.buffer = "Updated".to_string();
    }

    assert_eq!(state.active_document().unwrap().buffer, "Updated");
}

#[test]
fn active_path_returns_path_of_active_document() {
    /* WHY: Verify that active_path() retrieves the filesystem path of the currently selected document. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let doc1 = Document::new("doc1.md", "Doc1");
    state.document.open_documents.push(doc1);
    state.document.active_doc_idx = Some(0);

    assert_eq!(
        state.active_path(),
        Some(std::path::PathBuf::from("doc1.md"))
    );

    state.document.active_doc_idx = None;
    assert_eq!(state.active_path(), None);
}

#[test]
fn set_active_split_direction_with_no_document_does_nothing() {
    /* WHY: Safety check: ensure that toggling layout options when no documents are open doesn't crash or create invalid state. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.set_active_split_direction(katana_platform::SplitDirection::Vertical);
    assert!(state.document.tab_split_states.is_empty());
}

#[test]
fn set_active_pane_order_with_no_document_does_nothing() {
    /* WHY: Safety check for pane order toggling without active documents. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.set_active_pane_order(katana_platform::PaneOrder::PreviewFirst);
    assert!(state.document.tab_split_states.is_empty());
}

#[test]
fn set_active_split_direction_and_pane_order_adds_new_state() {
    /* WHY: Verify that layout preferences (split direction and order) are correctly initialized and stored for a document. */
    let mut state = AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    let doc1 = Document::new("doc1.md", "Doc1");
    state.document.open_documents.push(doc1);
    state.document.active_doc_idx = Some(0);

    state.set_active_split_direction(katana_platform::SplitDirection::Horizontal);
    assert_eq!(state.document.tab_split_states.len(), 1);
    assert_eq!(
        state.document.tab_split_states[0].state.direction,
        katana_platform::SplitDirection::Horizontal
    );

    state.document.tab_split_states.clear();
    state.set_active_pane_order(katana_platform::PaneOrder::EditorFirst);
    assert_eq!(state.document.tab_split_states.len(), 1);
    assert_eq!(
        state.document.tab_split_states[0].state.order,
        katana_platform::PaneOrder::EditorFirst
    );
}
