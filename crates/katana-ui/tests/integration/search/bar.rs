use eframe::egui;
use egui_kittest::kittest::Queryable;
use katana_ui::state::search::SearchParams;
use katana_ui::widgets::SearchBar;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_search_bar_id_stability() {
    /* WHY: Regression test for ID stability in the SearchBar widget.
     * Verifies that the internal TextEdit maintains focus even when the clear button ([x]) toggles visibility,
     * which relies on consistent widget IDs to prevent IME breakages. */
    let params = Rc::new(RefCell::new(SearchParams::default()));

    let params_clone = Rc::clone(&params);
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(300.0, 300.0))
        .build_ui(move |ui| {
            // explicitly set id source so we can test stability
            let mut p = params_clone.borrow_mut();
            SearchBar::new(&mut p).id_source("test_bar").show(ui);
        });

    harness.run();

    let node = harness.get_by_role(accesskit::Role::TextInput);
    node.click();
    harness.run();

    let focused_id = harness.ctx.memory(|m| m.focused());
    assert!(
        focused_id.is_some(),
        "TextEdit should be focused after click"
    );

    // Simulate typing text, which forces the [x] button to appear BEFORE the TextEdit.
    // In previous versions this shifted the auto-id.
    params.borrow_mut().query = "a".to_string();
    harness.run();

    let focused_id_after = harness.ctx.memory(|m| m.focused());
    assert_eq!(
        focused_id, focused_id_after,
        "TextEdit ID changed after query was modified, causing loss of focus (and breaking IME)!"
    );
}
