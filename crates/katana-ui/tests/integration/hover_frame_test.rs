use egui_kittest::Harness;
use katana_ui::widgets::markdown::MarkdownViewer;

#[test]
fn red_task_list_hover_frame_vertically_aligned() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui(|ui| {
            let markdown = "- [ ] Item 1\n- [ ] Item 2\n- [ ] Item 3";
            let mut cache = egui_commonmark::CommonMarkCache::default();
            MarkdownViewer::new("hover_test", markdown).show(ui, &mut cache);
        });

    harness.run();

    /* WHY: Query for List items, check horizontal bounding boxes and hover rects */
    /* WHY: Not possible to query internal `active_rects` easily from outside. */
    /* WHY: I need to write a test inside `vendor/...` or find a way. */
}
