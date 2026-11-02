use super::*;
use crate::state::document::TabGroup;

fn group(id: &str, name: &str, color: &str) -> TabGroup {
    TabGroup {
        id: id.into(),
        name: name.into(),
        color_hex: color.into(),
        collapsed: false,
        members: vec![format!("/{id}.md")],
    }
}

fn app() -> KatanaApp {
    let mut state = crate::app_state::AppState::new(
        Default::default(),
        Default::default(),
        katana_platform::SettingsService::new(Box::new(katana_platform::InMemoryRepository)),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
        katana_platform::workspace::InMemoryWorkspaceRepository::default(),
    ));
    KatanaApp::new(state)
}

#[test]
fn all_mutators_reject_duplicate_pairs_before_mutation() {
    let mut app = app();
    app.state.document.tab_groups = vec![group("a", "", "#4A90D9"), group("b", "Work", "#D94A4A")];
    let original = app.state.document.tab_groups.clone();
    app.handle_action_create_tab_group(" ".into(), "#4a90d9".into(), vec!["/a.md".into()]);
    app.commit_tab_group_draft();
    assert_eq!(app.state.document.tab_groups, original);
    assert!(app.state.layout.group_draft.is_some());
    app.handle_action_rename_tab_group("b".into(), " ".into());
    assert_eq!(app.state.document.tab_groups[1].name, " ");
    let before = app.state.document.tab_groups.clone();
    app.handle_action_recolor_tab_group("b".into(), "#4a90d9".into());
    assert_eq!(app.state.document.tab_groups, before);
    app.handle_action_rename_tab_group("b".into(), "Work".into());
    app.handle_action_recolor_tab_group("a".into(), "#D94A4A".into());
    assert_eq!(app.state.document.tab_groups[0].color_hex, "#D94A4A");
    let before = app.state.document.tab_groups.clone();
    app.handle_action_rename_tab_group("b".into(), "".into());
    assert_eq!(app.state.document.tab_groups, before);
}

#[test]
fn legacy_duplicate_noop_and_deserialization_preserve_values() {
    let groups = vec![group("a", "", "#4A90D9"), group("b", "", "#4a90d9")];
    let saved = serde_json::to_string(&groups).unwrap();
    let restored: Vec<TabGroup> = serde_json::from_str(&saved).unwrap();
    assert_eq!(restored, groups);
    let mut app = app();
    app.state.document.tab_groups = restored;
    app.update_tab_group_identity("a", "   ".into(), "#4a90d9ff".into());
    assert_eq!(app.state.document.tab_groups, groups);
}

#[test]
fn named_many_creates_directly_but_duplicate_remains_a_draft() {
    let mut app = app();
    let ctx = eframe::egui::Context::default();
    let action = || crate::app_state::AppAction::CreateTabGroupMany {
        name: "Folder".into(),
        color_hex: "#4A90D9".into(),
        members: Vec::new(),
    };
    app.process_action(&ctx, action());
    assert_eq!(app.state.document.tab_groups.len(), 1);
    assert!(app.state.layout.group_draft.is_none());
    app.process_action(&ctx, action());
    assert_eq!(app.state.document.tab_groups.len(), 1);
    assert!(app.state.layout.group_draft.is_some());
}

#[test]
fn named_single_creates_directly_but_duplicate_remains_a_draft() {
    let mut app = app();
    let path = std::path::PathBuf::from("/single.md");
    app.state
        .document
        .open_documents
        .push(katana_core::document::Document::new(
            path.clone(),
            String::new(),
        ));
    let ctx = eframe::egui::Context::default();
    let action = || crate::app_state::AppAction::CreateTabGroup {
        name: "Single".into(),
        color_hex: "#4A90D9".into(),
        initial_member: path.clone(),
    };
    app.process_action(&ctx, action());
    assert_eq!(app.state.document.tab_groups.len(), 1);
    assert!(app.state.layout.group_draft.is_none());
    app.process_action(&ctx, action());
    assert_eq!(app.state.document.tab_groups.len(), 1);
    assert!(app.state.layout.group_draft.is_some());
}

#[test]
fn demo_group_respects_existing_user_pair() {
    let mut app = app();
    let groups = vec![group("user", "Demo", "#808080")];
    app.state.document.tab_groups = groups.clone();
    app.handle_action_open_help_demo();
    assert_eq!(app.state.document.tab_groups, groups);
    assert!(app.state.layout.status_message.is_some());
}

#[test]
fn finish_action_keeps_editor_and_draft_on_conflict() {
    let mut app = app();
    app.state.document.tab_groups = vec![group("a", "A", "#4A90D9"), group("b", "B", "#4A90D9")];
    app.state.layout.inline_rename_group = Some("b".into());
    let ctx = eframe::egui::Context::default();
    let popup = eframe::egui::Id::new("group_popup").with("b");
    let draft = popup.with("group_name_draft");
    eframe::egui::Popup::open_id(&ctx, popup);
    ctx.data_mut(|d| d.insert_temp(draft, "A".to_owned()));
    finish(&mut app, &ctx, "A");
    assert_eq!(app.state.layout.inline_rename_group.as_deref(), Some("b"));
    assert!(eframe::egui::Popup::is_id_open(&ctx, popup));
    assert_eq!(
        ctx.data_mut(|d| d.get_temp::<String>(draft)),
        Some("A".into())
    );
    assert_eq!(app.state.document.tab_groups[1].name, "B");
    assert!(app.state.layout.status_message.is_some());
    finish(&mut app, &ctx, "C");
    assert_eq!(app.state.layout.inline_rename_group, None);
    assert!(!eframe::egui::Popup::is_id_open(&ctx, popup));
    assert_eq!(app.state.document.tab_groups[1].name, "C");
}

fn finish(app: &mut KatanaApp, ctx: &eframe::egui::Context, name: &str) {
    app.process_action(
        ctx,
        crate::app_state::AppAction::FinishInlineRename {
            group_id: "b".into(),
            new_identity: Some((name.into(), "#4A90D9".into())),
        },
    );
}
