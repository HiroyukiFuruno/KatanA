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
fn full_app_group_cancel_button_and_escape_preserve_group() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    for escape in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let (mut harness, file) = prepare(&directory);
        let before = harness.state().state.document.tab_groups.len();
        open_group_menu(&mut harness, &file);
        let popup = popup_id(&harness);
        let original = group_name(&harness);
        harness.event(egui::Event::Text("discarded".to_owned()));
        harness.step();
        cancel(&mut harness, escape);
        assert_closed(&mut harness, popup);
        assert_eq!(harness.state().state.document.tab_groups.len(), before + 1);
        assert_eq!(group_name(&harness), original);
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
    let popup = popup_id(&harness);
    harness.event(egui::Event::Text("ExistingGroup".to_owned()));
    harness.step();
    harness.key_press(egui::Key::Enter);
    harness.step();
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
