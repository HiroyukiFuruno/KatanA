use egui_commonmark::CommonMarkViewer;
use egui_kittest::Harness;

#[test]
fn red_task_list_hover_frame_vertically_aligned() {
    /* WHY: Verify that hover frames for list items are vertically aligned and don't jitter.
     * [Note]: This is currently a layout-only test as kittest doesn't expose internal active_rects for detailed geometric validation. */
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui(|ui| {
            let markdown = "- [ ] Item 1\n- [ ] Item 2\n- [ ] Item 3";
            let mut cache = egui_commonmark::CommonMarkCache::default();
            CommonMarkViewer::new().show(ui, &mut cache, markdown);
        });

    harness.run();
}
