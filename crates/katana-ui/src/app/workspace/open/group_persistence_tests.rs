use super::session::WorkspaceOpenSessionOps;
use crate::app::action::ActionOps;
use crate::app::workspace::WorkspaceOps;
use crate::shell::KatanaApp;
use crate::state::document::TabGroup;
use std::path::Path;

fn app(root: &Path) -> KatanaApp {
    let settings = katana_platform::SettingsService::new(Box::new(
        katana_platform::JsonFileRepository::new(root.join("settings.json")),
    ));
    let mut state = crate::app_state::AppState::new(
        Default::default(),
        Default::default(),
        settings,
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
        katana_platform::workspace::InMemoryWorkspaceRepository::default(),
    ));
    state.workspace.data = Some(katana_core::workspace::Workspace::new(root, Vec::new()));
    state
        .config
        .settings
        .settings_mut()
        .workspace
        .restore_session = true;
    KatanaApp::new(state)
}

fn group(id: &str, member: &str) -> TabGroup {
    TabGroup {
        id: id.into(),
        name: "".into(),
        color_hex: "#4A90D9".into(),
        collapsed: false,
        members: vec![member.into()],
    }
}

#[test]
fn duplicate_legacy_groups_restore_and_edit_persists() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let (original, docs) = write_legacy(root);
    let mut reopened = app(root);
    WorkspaceOpenSessionOps::restore_session_tabs(&mut reopened, &root.to_string_lossy());
    assert_eq!(reopened.state.document.tab_groups, original);
    reopened.state.document.open_documents = docs;
    reopened.process_action(
        &eframe::egui::Context::default(),
        crate::app_state::AppAction::RenameTabGroup {
            group_id: "b".into(),
            new_name: "Edited".into(),
        },
    );
    let mut again = app(root);
    WorkspaceOpenSessionOps::restore_session_tabs(&mut again, &root.to_string_lossy());
    assert_eq!(again.state.document.tab_groups[0], original[0]);
    assert_eq!(again.state.document.tab_groups[1].name, "Edited");
    assert_eq!(
        again.state.document.tab_groups[1].color_hex,
        original[1].color_hex
    );
}

fn write_legacy(root: &Path) -> (Vec<TabGroup>, Vec<katana_core::document::Document>) {
    let path = root.join("a.md");
    std::fs::write(&path, "fixture").unwrap();
    let path_string = path.to_string_lossy().to_string();
    let original = vec![group("a", &path_string), group("b", &path_string)];
    let mut writer = app(root);
    writer
        .state
        .document
        .open_documents
        .push(katana_core::document::Document::new(path, "fixture"));
    writer.state.document.tab_groups = original.clone();
    writer.save_workspace_state();
    (original, writer.state.document.open_documents)
}
