use katana_ui::preview_pane::PreviewPane;
use eframe::egui;

#[test]
fn block_anchors_extracted_for_rich_blocks() {
    /* WHY: Verify that the preview pane correctly extracts bounding boxes (anchors) for 
     * rich blocks like Mermaid diagrams and Alerts, enabling features like hover-sync and source-mapping. */
    let md = concat!(
        "Some text\n",
        "```mermaid\n",
        "graph TD;\n",
        "```\n",
        "\n",
        "> [!NOTE]\n",
        "> An alert block\n"
    );

    let ctx = egui::Context::default();
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0))),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut pane = PreviewPane::default();
                pane.update_markdown_sections(md, std::path::Path::new("/tmp/test.md"));
                pane.show_content(ui, None, None, None, None);
                assert!(!pane.block_anchors.is_empty());
            });
        },
    );
}

#[test]
fn consecutive_rich_blocks_produce_correct_block_anchors() {
    /* WHY: Verify that consecutive rich blocks (e.g., Code -> Alert -> Code) produce 
     * non-overlapping, monotonically increasing anchors. This guards against stack-depth 
     * corruption in the markdown event handler. */
    let md = concat!("```rust\nfn hello() {}\n```\n\n> [!NOTE]\n> Alert\n\n```python\nprint(1)\n```\n");
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut pane = PreviewPane::default();
            pane.update_markdown_sections(md, std::path::Path::new("/tmp/test.md"));
            pane.show_content(ui, None, None, None, None);
            let anchors = &pane.block_anchors;
            assert!(anchors.len() >= 3);
            for window in anchors.windows(2) {
                assert!(window[1].0.start >= window[0].0.end);
            }
        });
    });
}
