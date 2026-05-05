use accesskit::Role;
use egui_kittest::kittest::Queryable;
use katana_ui::app_state::{AppAction, ViewMode};

use crate::integration::harness_utils::{fresh_temp_dir, setup_harness, wait_for_workspace_load};

fn click_code_editor_input(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
) {
    for attempt in 0..600 {
        if let Some(editor) = harness.query_by(|node| {
            node.role() == Role::MultilineTextInput
                && node
                    .value()
                    .as_deref()
                    .is_some_and(|value| value.contains("alpha"))
        }) {
            editor.click();
            harness.step();
            return;
        }

        if let Some(editor) = harness.query_by(|node| node.role() == Role::MultilineTextInput) {
            editor.click();
            harness.step();
            return;
        }

        harness.step();

        if attempt < 20 {
            std::thread::yield_now();
        } else {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }

    panic!("code editor input with expected text did not appear in time");
}

fn open_code_only_document(
    harness: &mut egui_kittest::Harness<'static, katana_ui::shell::KatanaApp>,
    prefix: &str,
    filename: &str,
    content: &str,
) {
    let temp_dir = fresh_temp_dir(prefix);
    let test_file = temp_dir.join(filename);
    std::fs::write(&test_file, content).unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.run_steps(10);

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.run_steps(5);
}

#[test]
fn test_integration_view_modes() {
    /* WHY: Verify that the application correctly switches between Split,
     * PreviewOnly, and CodeOnly modes, and that the UI state reflects these changes. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_modes");
    let test_file = temp_dir.join("test_modes.md");
    std::fs::write(&test_file, "# Hello View Modes\n**Bold text here.**").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir.clone()));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.step();

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::PreviewOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::PreviewOnly
    );

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::Split);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::Split
    );

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.step();
    assert_eq!(
        harness.state_mut().app_state_mut().active_view_mode(),
        ViewMode::CodeOnly
    );
}

#[test]
fn input_assist_code_block_button_updates_editor_buffer() {
    let mut harness = setup_harness();
    harness.step();
    open_code_only_document(
        &mut harness,
        "katana_test_authoring_toolbar",
        "authoring.md",
        "alpha",
    );

    click_code_editor_input(&mut harness);
    harness.run_steps(5);

    let code_block_buttons: Vec<_> = harness
        .query_all_by_role_and_label(Role::Button, "Code Block")
        .collect();
    assert_eq!(
        code_block_buttons.len(),
        1,
        "there should be exactly one visible Code Block button before clicking"
    );
    code_block_buttons[0].click();
    harness.run_steps(5);

    let menu_item = harness
        .query_by_role_and_label(Role::Button, "text")
        .expect("code block kind menu should open after clicking the input assist code icon");
    menu_item.click();
    harness.run_steps(5);

    let buffer = harness
        .state()
        .app_state_for_test()
        .active_document()
        .expect("document should stay active")
        .buffer
        .clone();

    assert!(
        buffer.contains("```text"),
        "input assist code block action should update the editor buffer, got: {buffer:?}"
    );
}

#[test]
fn clipboard_image_file_url_paste_queues_image_ingest_action() {
    let mut harness = setup_harness();
    harness.step();
    open_code_only_document(
        &mut harness,
        "katana_test_clipboard_file_url",
        "clipboard_file_url.md",
        "alpha",
    );

    click_code_editor_input(&mut harness);
    harness.run_steps(5);

    harness.event(egui::Event::Paste(
        "file:///tmp/katana%20clipboard%20image.PNG".to_string(),
    ));
    harness.step();

    assert!(
        matches!(
            harness.state().pending_action_for_test(),
            AppAction::IngestClipboardImage
        ),
        "image file URL paste in the focused code editor should queue clipboard image ingestion"
    );
}

#[test]
fn code_block_kind_menu_closes_when_editor_is_clicked() {
    let mut harness = setup_harness();
    harness.step();
    open_code_only_document(
        &mut harness,
        "katana_test_authoring_menu_blur",
        "authoring.md",
        "alpha",
    );

    click_code_editor_input(&mut harness);
    harness.run_steps(5);

    harness
        .query_by_role_and_label(Role::Button, "Code Block")
        .expect("Code Block button should be visible")
        .click();
    harness.run_steps(5);
    assert!(
        harness
            .query_by_role_and_label(Role::Button, "text")
            .is_some(),
        "code block kind menu should be open before clicking outside"
    );

    harness.event(egui::Event::PointerButton {
        pos: egui::pos2(24.0, 24.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
    harness.event(egui::Event::PointerButton {
        pos: egui::pos2(24.0, 24.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    harness.run_steps(5);

    assert!(
        harness
            .query_by_role_and_label(Role::Button, "text")
            .is_none(),
        "code block kind menu should close after the editor is clicked"
    );
}

#[test]
#[ignore = "uses the current OS clipboard; run manually while an image is copied"]
fn live_clipboard_image_shortcut_inserts_markdown_from_current_os_clipboard() {
    let mut harness = setup_harness();
    harness.step();
    open_code_only_document(
        &mut harness,
        "katana_test_live_clipboard_shortcut",
        "clipboard_live.md",
        "alpha",
    );

    harness
        .get_by(|node| {
            node.role() == Role::MultilineTextInput && node.value().as_deref() == Some("alpha")
        })
        .click();
    harness.run_steps(5);

    let mut modifiers = egui::Modifiers::NONE;
    modifiers.command = true;
    harness.event(egui::Event::Key {
        key: egui::Key::V,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    });
    harness.run_steps(10);

    let buffer = harness
        .state()
        .app_state_for_test()
        .active_document()
        .expect("document should stay active")
        .buffer
        .clone();

    assert!(
        buffer.contains("![](./asset/img/"),
        "Command+V should read the OS clipboard image and insert markdown, got: {buffer:?}"
    );
}

#[test]
fn test_integration_editor_line_numbers_visibility() {
    /* WHY: Verify that line numbers are rendered in the editor when a document is open. */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_editor_lines");
    let test_file = temp_dir.join("lines.md");
    std::fs::write(&test_file, "Line 1\nLine 2\nLine 3").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.run_steps(10);

    harness
        .state_mut()
        .app_state_mut()
        .set_active_view_mode(ViewMode::CodeOnly);
    harness.run_steps(5);

    let count_1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.query_all_by_label("1").count()
    }))
    .unwrap_or(0);
    assert!(count_1 > 0, "Line number 1 should be visible");
}

#[test]
fn test_integration_update_buffer() {
    /* WHY: Verify that modifying the editor buffer correctly updates the internal state
     * and that these changes are reflected in the preview pane (if visible). */
    let mut harness = setup_harness();
    harness.step();

    let temp_dir = fresh_temp_dir("katana_test_ws_buf");
    let test_file = temp_dir.join("buf_test.md");
    std::fs::write(&test_file, "# Original").unwrap();

    harness
        .state_mut()
        .trigger_action(AppAction::OpenWorkspace(temp_dir));
    wait_for_workspace_load(&mut harness);
    harness
        .state_mut()
        .trigger_action(AppAction::SelectDocument(test_file.canonicalize().unwrap()));
    harness.run_steps(10);

    // Simulate typing " Updated"
    harness
        .state_mut()
        .trigger_action(AppAction::UpdateBuffer("# Original Updated".to_string()));
    harness.run_steps(10);

    assert!(harness.query_all_by_label("Original Updated").count() > 0);
}
