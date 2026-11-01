use super::fixture::*;
use crate::shell::KatanaApp;
use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};

fn cancel(harness: &mut Harness<'_, KatanaApp>, escape: bool) {
    if escape {
        harness.key_press(egui::Key::Escape);
    } else {
        let label = crate::i18n::I18nOps::get().action.cancel.clone();
        harness.get_by_label(&label).click();
    }
    harness.step();
}

#[test]
fn full_app_group_cancel_button_and_escape_discard_draft() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    for escape in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let (mut harness, file) = prepare(&directory);
        let before = harness.state().state.document.tab_groups.len();
        open_group_menu(&mut harness, &file);
        let popup = egui::Id::new("new_group_draft");
        harness.event(egui::Event::Text("discarded".to_owned()));
        harness.step();
        cancel(&mut harness, escape);
        assert_closed(&mut harness, popup);
        assert_eq!(harness.state().state.document.tab_groups.len(), before);
    }
}

fn focus_name_field(harness: &mut Harness<'_, KatanaApp>, popup: egui::Id) {
    let field = popup.with("group_name_field");
    let pos = harness
        .ctx
        .read_response(field)
        .expect("name field response")
        .rect
        .center();
    harness.event(egui::Event::PointerMoved(pos));
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
}

#[test]
fn full_app_existing_group_rename_cancel_preserves_name() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().unwrap();
    let (mut harness, file) = prepare(&directory);
    open_group_menu(&mut harness, &file);
    harness.event(egui::Event::Text("ExistingGroup".to_owned()));
    harness.step();
    harness.key_press(egui::Key::Enter);
    harness.step();
    let popup = popup_id(&harness);
    assert_closed(&mut harness, popup);
    let name = group_name(&harness);
    assert_eq!(name, "ExistingGroup");
    harness.get_by_label(&name).click_secondary();
    steps(&mut harness);
    focus_name_field(&mut harness, popup);
    harness.event(egui::Event::Text("discarded".to_owned()));
    harness.step();
    cancel(&mut harness, false);
    assert_closed(&mut harness, popup);
    assert_eq!(group_name(&harness), name);
}

fn install_conflict_groups(
    harness: &mut Harness<'_, KatanaApp>,
    directory: &tempfile::TempDir,
    second_name: &str,
    second_color: &str,
) {
    let member = directory
        .path()
        .join("group-native-fixture.md")
        .to_string_lossy()
        .to_string();
    let make =
        |id: &str, name: &str, color: &str, member: String| crate::state::document::TabGroup {
            id: id.into(),
            name: name.into(),
            color_hex: color.into(),
            collapsed: false,
            members: vec![member],
        };
    harness.state_mut().state.document.tab_groups = vec![
        make("a", "A", "#4A90D9", member),
        make("b", second_name, second_color, "other.md".into()),
    ];
}

#[test]
fn existing_rename_conflict_stays_open_and_preserves_groups() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().unwrap();
    let (mut harness, _) = prepare(&directory);
    install_conflict_groups(&mut harness, &directory, "AB", "#4A90D9");
    steps(&mut harness);
    harness
        .get_all_by_label("A")
        .next()
        .unwrap()
        .click_secondary();
    steps(&mut harness);
    let popup = egui::Id::new("group_popup").with("a");
    focus_name_field(&mut harness, popup);
    harness.event(egui::Event::Text("B".into()));
    harness.step();
    let error = crate::i18n::I18nOps::get()
        .tab
        .group_identity_duplicate
        .clone();
    assert!(harness.query_by_label(&error).is_some());
    harness.key_press(egui::Key::Enter);
    steps(&mut harness);
    assert!(egui::Popup::is_id_open(&harness.ctx, popup));
    assert_eq!(harness.state().state.document.tab_groups[0].name, "A");
}

#[test]
fn existing_recolor_conflict_stays_open_and_preserves_color() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().unwrap();
    let (mut harness, _) = prepare(&directory);
    install_conflict_groups(&mut harness, &directory, "A", "#D94A4A");
    steps(&mut harness);
    harness
        .get_all_by_label("A")
        .next()
        .unwrap()
        .click_secondary();
    steps(&mut harness);
    harness.get_by_label("#D94A4A").click();
    steps(&mut harness);
    let error = crate::i18n::I18nOps::get()
        .tab
        .group_identity_duplicate
        .clone();
    assert!(harness.query_by_label(&error).is_some());
    harness
        .get_by_label(&crate::i18n::I18nOps::get().action.confirm)
        .click();
    steps(&mut harness);
    assert_eq!(
        harness.state().state.document.tab_groups[0].color_hex,
        "#4A90D9"
    );
    assert_group_popup_open(&harness);
    confirm_unique_color(&mut harness);
}

fn assert_group_popup_open(harness: &Harness<'_, KatanaApp>) {
    let popup = egui::Id::new("group_popup").with("a");
    assert!(egui::Popup::is_id_open(&harness.ctx, popup));
}

fn confirm_unique_color(harness: &mut Harness<'_, KatanaApp>) {
    harness.get_by_label("#4AD97A").click();
    steps(harness);
    harness
        .get_by_label(&crate::i18n::I18nOps::get().action.confirm)
        .click();
    steps(harness);
    assert_eq!(
        harness.state().state.document.tab_groups[0].color_hex,
        "#4AD97A"
    );
}
