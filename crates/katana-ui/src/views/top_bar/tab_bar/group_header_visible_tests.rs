use super::GroupHeader;
use crate::app::action::ActionOps;
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
    let mut app = super::tests::make_app();
    app.state.document.tab_groups.push(group());
    app.state.layout.inline_rename_group = Some("visible_test".into());
    let mut harness = egui_kittest::Harness::builder()
        .with_size(TEST_VIEWPORT)
        .build_ui_state(
            |ui, app: &mut crate::shell::KatanaApp| {
                let group = group();
                let mut action = None;
                GroupHeader {
                    g: &group,
                    groups: std::slice::from_ref(&group),
                    inline_rename_group: app.state.layout.inline_rename_group.as_ref(),
                }
                .show(ui, &mut action);
                if let Some(action) = action {
                    app.process_action(ui.ctx(), action);
                }
            },
            app,
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
