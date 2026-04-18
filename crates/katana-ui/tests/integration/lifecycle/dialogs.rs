use crate::integration::harness_utils::setup_harness;
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::changelog::ChangelogSection;
use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_changelog_tab_display() {
    /* WHY: Verify that the release notes (changelog) tab can be opened, sections can be populated for testing,
     * and that it displays correctly with correctly translated titles. */
    let mut harness = setup_harness();
    harness.step();

    harness
        .state_mut()
        .trigger_action(AppAction::ShowReleaseNotes);

    harness.step();

    harness.state_mut().clear_changelog_rx_for_test();
    harness
        .state_mut()
        .set_changelog_sections_for_test(vec![ChangelogSection {
            version: "0.8.0".to_string(),
            heading: "v0.8.0".to_string(),
            body: "### Features\n- Fixed the close button overlap".to_string(),
            default_open: true,
        }]);

    for _ in 0..9 {
        harness.step();
    }
    {
        let state = harness.state();
        let app = state.app_state_for_test();
        let active_doc = app.active_document().expect("a document MUST be active");
        assert!(
            active_doc
                .path
                .to_string_lossy()
                .starts_with("Katana://ChangeLog")
        );
    }
    harness.step();
    harness.step();

    let i18n = I18nOps::get();
    let expected_title = format!("{} v{}", i18n.menu.release_notes, env!("CARGO_PKG_VERSION"));
    harness.get_by_label(&expected_title);

    harness.get_by_label("Fixed the close button overlap");

    /* WHY: Verify that the version header exists and can be interacted with (clicking to toggle section). */
    let header_label = "v0.8.0";
    harness.get_by_label(header_label).hover();
    harness.step();
    harness.get_by_label(header_label).click();
    harness.step();
}
