use super::GroupHeader;
use crate::app_state::AppAction;
use crate::state::document::TabGroup;
use eframe::egui;

fn group() -> TabGroup {
    TabGroup {
        id: "visible_test".into(),
        name: "Group".into(),
        color_hex: "#4A90D9".into(),
        collapsed: false,
        members: vec!["file.md".into()],
    }
}

const TEST_VIEWPORT: egui::Vec2 = egui::vec2(640.0, 240.0);

#[test]
fn closed_header_popup_removes_visible_controls() {
    use egui_kittest::kittest::Queryable;
    let mut harness = egui_kittest::Harness::builder()
        .with_size(TEST_VIEWPORT)
        .build_ui_state(
            |ui, editing: &mut bool| {
                let group = group();
                let mut action = None;
                GroupHeader {
                    g: &group,
                    inline_rename_group: editing.then_some(&group.id),
                }
                .show(ui, &mut action);
                if matches!(action, Some(AppAction::FinishInlineRename { .. })) {
                    *editing = false;
                }
            },
            true,
        );
    for _ in 0..3 {
        harness.step();
    }
    let cancel = crate::i18n::I18nOps::get().action.cancel.clone();
    assert!(harness.query_by_label(&cancel).is_some());
    harness.key_press(egui::Key::Enter);
    for _ in 0..3 {
        harness.step();
        assert!(harness.query_by_label(&cancel).is_none());
    }
}
