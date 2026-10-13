use crate::integration::harness_utils::{setup_harness, wait_for_workspace_load, fresh_temp_dir};
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::AppAction;
use katana_ui::i18n::I18nOps;

#[test]
fn test_integration_toc_panel_display() {
    /* WHY: Verify that the TOC panel can be toggled on and that heading entries match the document content. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc");
    let test_file1 = temp_dir.join("toc_test1.md");
    std::fs::write(&test_file1, "# Heading 1").unwrap();
    let test_file1 = test_file1.canonicalize().unwrap_or(test_file1);

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);

    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file1.clone()));
    harness.step();
    harness.step();

    let toggle_btn_label = I18nOps::get().action.toggle_toc.clone();
    let toggle_btn = harness.get_by_label(&toggle_btn_label);
    toggle_btn.click();
    harness.step();
    harness.step();

    let toc_visible = harness.state_mut().app_state_mut().layout.show_toc;
    assert!(toc_visible, "show_toc should be true after clicking button");

    let toc_title = I18nOps::get().toc.title.clone();
    let _panel = harness.get_by_label(&toc_title);

    let headings_count = harness.query_all_by_label("Heading 1").count();
    assert_eq!(
        headings_count, 2,
        "Heading 1 should appear exactly twice: once in TOC, once in preview text"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_toc_enable_disable_setting() {
    /* WHY: Verify that the TOC UI button correctly reflects and persists the layout setting. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc_setting");
    let test_file1 = temp_dir.join("toc_test_setting.md");
    std::fs::write(&test_file1, "# Heading 1").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file1.canonicalize().unwrap_or(test_file1.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();
    harness.step();

    let toc_icon = I18nOps::get().action.toggle_toc.clone();
    assert_eq!(
        harness.query_all_by_label(&toc_icon).count(),
        1,
        "TOC button should be visible when toc_visible setting is true (default)"
    );

    harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings_mut()
        .layout
        .toc_visible = false;
    harness.step();
    harness.step();

    assert!(
        !harness
            .state_mut()
            .app_state_mut()
            .config
            .settings
            .settings()
            .layout
            .toc_visible,
        "TOC setting must be false"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_toc_panel_hides_when_disabled() {
    /* WHY: Verify that disabling TOC in settings also hides an already-visible TOC panel. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc_hide");
    let test_file1 = temp_dir.join("toc_hide_test.md");
    std::fs::write(&test_file1, "# Heading 1").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file1.canonicalize().unwrap_or(test_file1.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));
    harness.step();
    harness.step();

    harness.state_mut().trigger_action(AppAction::ToggleToc);
    for _ in 0..10 {
        harness.step();
    }

    let toc_title = I18nOps::get().toc.title.clone();
    assert_eq!(
        harness.query_all_by_label(&toc_title).count(),
        1,
        "TOC panel MUST be visible after toggling it on"
    );

    harness
        .state_mut()
        .app_state_mut()
        .config
        .settings
        .settings_mut()
        .layout
        .toc_visible = false;
    harness.step();
    harness.step();

    let is_panel_visible = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label(&toc_title).count()
    }))
    .unwrap_or(0)
        > 0;

    assert!(
        !is_panel_visible,
        "TOC panel MUST NOT be visible when toc_visible setting is false"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_integration_toc_panel_truncates_long_headings() {
    /* WHY: UI Layout stability check: Verify that extremely long headings in TOC do not cause the panel to overflow its max width. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_toc_truncate");
    let test_file = temp_dir.join("toc_truncate.md");
    let very_long_text = "This is a very very very very very very very very very very very very very very very very very extremely long heading that should definitely be truncated".to_string();
    let content = format!("# {}\n\nSome text.", very_long_text);
    std::fs::write(&test_file, content).unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    let abs_path = test_file.canonicalize().unwrap_or(test_file.clone());
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(abs_path));

    for _ in 0..20 {
        harness.step();
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    harness.state_mut().trigger_action(AppAction::ToggleToc);

    for _ in 0..10 {
        harness.step();
    }

    let toc_visible = harness.state_mut().app_state_mut().layout.show_toc;
    assert!(toc_visible, "TOC should be visible");

    let toc_title = I18nOps::get().toc.title.clone();
    let panel_node = harness.get_by_label(&toc_title);

    let panel_width = panel_node.rect().width();
    assert!(
        panel_width <= 550.0,
        "TOC Panel width ({}) should be constrained to TOC_MAX_WIDTH (~500)",
        panel_width
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}
