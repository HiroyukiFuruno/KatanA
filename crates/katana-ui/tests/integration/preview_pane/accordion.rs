use katana_ui::preview_pane::PreviewPane;
use eframe::egui;
use egui_kittest::kittest::Queryable;

#[test]
fn test_preview_accordion_auto_open_on_search_match() {
    /* WHY: Verify that hidden content inside a <details> block is automatically revealed 
     * by opening the accordion if it contains a search match. */
    let text = "<details><summary>Hidden</summary>\n\nMysteryWord\n\n</details>".to_string();

    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(&text, std::path::Path::new("/tmp/test.md"));

    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::Vec2::new(800.0, 600.0))
        .build_ui(|ui| {
            pane.show_content(ui, None, None, Some("MysteryWord".to_string()), None);
        });

    harness.step();
    harness.step();

    let found = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        harness.get_by_label("MysteryWord")
    }));

    assert!(
        found.is_ok(),
        "The accordion must be auto-opened, exposing the search match 'MysteryWord'"
    );
}
