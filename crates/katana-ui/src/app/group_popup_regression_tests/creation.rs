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
        assert_eq!(harness.state().state.document.tab_groups.len(), before + 1);
        let popup = popup_id(&harness);
        assert_name_field_ready(&harness, popup);
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

fn assert_name_field_ready(
    harness: &egui_kittest::Harness<'_, crate::shell::KatanaApp>,
    popup: egui::Id,
) {
    assert!(
        harness
            .query_by_label(&crate::i18n::I18nOps::get().action.cancel)
            .is_some()
    );
    assert!(egui::Popup::is_id_open(&harness.ctx, popup));
    let field = popup.with("group_name_field");
    assert!(
        harness
            .ctx
            .read_response(field)
            .expect("name field")
            .enabled()
    );
    assert_eq!(harness.ctx.memory(|memory| memory.focused()), Some(field));
}

#[test]
fn full_app_popup_blocks_background_until_closed() {
    let _lock = crate::test_render_env::RenderEnvLock::lock();
    let directory = tempfile::tempdir().unwrap();
    let (mut harness, file) = prepare(&directory);
    open_group_menu(&mut harness, &file);
    let popup = popup_id(&harness);
    let search = crate::i18n::I18nOps::get().search.modal_title.clone();
    harness.get_by_label(&search).click();
    steps(&mut harness);
    assert!(!harness.state().state.layout.show_search_modal);
    assert_closed(&mut harness, popup);
    harness.get_by_label(&search).click();
    steps(&mut harness);
    assert!(harness.state().state.layout.show_search_modal);
    harness.get_all_by_label(&file).next().unwrap().click();
    steps(&mut harness);
    assert!(harness.state().state.layout.show_search_modal);
    assert_eq!(harness.state().state.document.tab_groups.len(), 1);
}
