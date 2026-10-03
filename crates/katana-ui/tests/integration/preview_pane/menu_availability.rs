use crate::integration::harness_utils::{setup_harness, wait_for_workspace_load};
use egui_kittest::kittest::{NodeT, Queryable};
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;
use katana_ui::shell::KatanaApp;

#[test]
fn html_preview_sidebar_exposes_unsupported_menus_as_disabled_hit_targets() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let html = workspace.path().join("requirements.html");
    std::fs::write(&html, "<main><h1>Requirements</h1></main>").expect("html fixture");
    let html = html.canonicalize().expect("canonical html fixture");

    let mut harness = setup_harness();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace.path().to_path_buf()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(html));
    for _ in 0..4 {
        harness.step();
    }

    assert_preview_sidebar_is_disabled(&mut harness);
}

#[test]
fn office_preview_sidebar_exposes_unsupported_menus_as_disabled_hit_targets() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/screenshot/fixtures/v0-22-38-multi-format/representative.xlsx")
        .canonicalize()
        .expect("representative xlsx fixture");
    let workspace = fixture.parent().expect("fixture directory").to_path_buf();
    let mut harness = setup_harness();
    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(workspace));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(fixture));
    for _ in 0..4 {
        harness.step();
    }

    assert_preview_sidebar_is_disabled(&mut harness);
}

fn assert_preview_sidebar_is_disabled(harness: &mut egui_kittest::Harness<'static, KatanaApp>) {
    let i18n = I18nOps::get();
    for label in [
        i18n.action.toggle_toc.clone(),
        i18n.menu.export.clone(),
        i18n.preview.slideshow_settings.clone(),
        i18n.menu.view.clone(),
    ] {
        let node = harness
            .query_all_by_label(&label)
            .find(|node| {
                node.accesskit_node()
                    .raw_bounds()
                    .is_some_and(|bounds| bounds.x0 >= 1_150.0)
            })
            .unwrap_or_else(|| panic!("preview sidebar button not found: {label}"));
        assert!(
            node.accesskit_node().is_disabled(),
            "preview sidebar button must be disabled: {label}"
        );
        node.click();
        harness.step();
    }

    let layout = &harness.state().app_state_for_test().layout;
    assert!(!layout.show_toc);
    assert!(!layout.show_export_panel);
    assert!(!layout.show_story_panel);
    assert!(!layout.show_tools_panel);
    assert!(!layout.show_slideshow);
}
