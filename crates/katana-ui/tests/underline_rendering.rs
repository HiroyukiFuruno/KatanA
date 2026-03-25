use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use katana_ui::preview_pane::PreviewPane;

#[test]
fn underline_tags_render_without_crash() {
    let mut pane = PreviewPane::default();

    // The <u> tag triggers our custom manual alignment logic
    // which bypasses egui's default TextFormat::underline stroke to avoid
    // the macOS CJK metric corruption bug.
    pane.update_markdown_sections(
        "Here is some <u>underlined text</u> in the preview.",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None);
    });
    harness.step();
    harness.run();

    // Verify the text is present in the layout
    let _text_node = harness.get_by_label_contains("underlined text");
}

#[test]
fn multiple_underlines_and_strikethroughs_in_same_block_render_safely() {
    let mut pane = PreviewPane::default();

    // Mixed decorations test the complex char_idx and stroke splitting logic
    // in the `flush_pending_inline` method override located in vendor/egui_commonmark/src/parsers/pulldown.rs
    pane.update_markdown_sections(
        "A <u>custom underline</u> and a ~~strikethrough~~ mixed.",
        std::path::Path::new("/tmp/test.md"),
    );

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None);
    });
    harness.step();
    harness.run();

    let _text_block = harness.get_by_label_contains("custom underline");
}
