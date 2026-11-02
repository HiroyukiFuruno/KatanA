use super::fixture::*;
use eframe::egui;
use egui_kittest::kittest::Queryable;

#[test]
fn full_app_group_named_and_empty_enter_close_popup() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    for name in ["NativeGroupProbe", ""] {
        let directory = tempfile::tempdir().unwrap();
        let (mut harness, file) = prepare(&directory);
        let before = harness.state().state.document.tab_groups.len();
        open_group_menu(&mut harness, &file);
        assert_eq!(harness.state().state.document.tab_groups.len(), before);
        let popup = egui::Id::new("new_group_draft");
        assert_name_field_ready(&harness);
        if !name.is_empty() {
            harness.event(egui::Event::Text(name.to_owned()));
            harness.step();
        }
        harness.key_press(egui::Key::Enter);
        harness.step();
        assert_closed(&mut harness, popup);
        assert_eq!(harness.state().state.document.tab_groups.len(), before + 1);
        assert_eq!(group_name(&harness), name);
    }
}

fn assert_name_field_ready(harness: &egui_kittest::Harness<'_, crate::shell::KatanaApp>) {
    assert!(
        harness
            .query_by_label(&crate::i18n::I18nOps::get().action.cancel)
            .is_some()
    );
    assert!(harness.state().state.layout.group_draft.is_some());
}

#[test]
fn full_app_popup_blocks_background_until_closed() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().unwrap();
    let (mut harness, file) = prepare(&directory);
    open_group_menu(&mut harness, &file);
    let popup = egui::Id::new("new_group_draft");
    let search = crate::i18n::I18nOps::get().search.modal_title.clone();
    harness.get_by_label(&search).click();
    steps(&mut harness);
    assert!(!harness.state().state.layout.show_search_modal);
    harness
        .get_by_label(&crate::i18n::I18nOps::get().action.cancel)
        .click();
    steps(&mut harness);
    assert_closed(&mut harness, popup);
    harness.get_by_label(&search).click();
    steps(&mut harness);
    assert!(harness.state().state.layout.show_search_modal);
    harness.get_all_by_label(&file).next().unwrap().click();
    steps(&mut harness);
    assert!(harness.state().state.layout.show_search_modal);
    assert_eq!(harness.state().state.document.tab_groups.len(), 0);
}

fn start_duplicate_draft(
    harness: &mut egui_kittest::Harness<'_, crate::shell::KatanaApp>,
    directory: &tempfile::TempDir,
) {
    harness
        .state_mut()
        .state
        .document
        .tab_groups
        .push(crate::state::document::TabGroup {
            id: "existing".into(),
            name: "".into(),
            color_hex: "#4A90D9".into(),
            collapsed: false,
            members: vec!["other.md".into()],
        });
    harness
        .state_mut()
        .trigger_action(crate::app_state::AppAction::CreateTabGroup {
            name: String::new(),
            color_hex: "#4A90D9".into(),
            initial_member: directory.path().join("group-native-fixture.md"),
        });
    steps(harness);
}

#[test]
fn duplicate_empty_pair_shows_error_until_color_changes() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().unwrap();
    let (mut harness, _file) = prepare(&directory);
    start_duplicate_draft(&mut harness, &directory);
    let error = crate::i18n::I18nOps::get()
        .tab
        .group_identity_duplicate
        .clone();
    assert!(harness.query_by_label(&error).is_some());
    harness.key_press(egui::Key::Enter);
    steps(&mut harness);
    assert_eq!(harness.state().state.document.tab_groups.len(), 1);
    assert!(harness.state().state.layout.group_draft.is_some());
    harness.get_by_label("#D94A4A").click();
    steps(&mut harness);
    assert!(harness.query_by_label(&error).is_none());
    harness
        .get_by_label(&crate::i18n::I18nOps::get().action.confirm)
        .click();
    steps(&mut harness);
    let groups = &harness.state().state.document.tab_groups;
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[1].name, "");
    assert_eq!(groups[1].color_hex, "#D94A4A");
}
