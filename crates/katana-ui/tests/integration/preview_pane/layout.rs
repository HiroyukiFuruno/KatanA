use katana_ui::preview_pane::PreviewPane;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use eframe::egui;

#[test]
fn blockquote_long_line_wraps_within_preview_width() {
    /* WHY: Verify that blockquotes with long lines correctly trigger text wrapping within the fixed width of the preview pane, 
     * preventing horizontal overflow. */
    let markdown = concat!(
        "> Note: On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple. ",
        "Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, std::path::Path::new("/tmp/blockquote.md"));

    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 240.0))
        .build_ui(move |ui| {
            pane.show_content(ui, None, None, None, None);
        });
    harness.step();
    harness.step();
    harness.run();

    let quote = harness.get_by_label_contains("Note:");
    let bounds = quote.accesskit_node().raw_bounds().expect("blockquote text should have bounds");
    assert!(
        bounds.x1 <= 520.0,
        "blockquote long line must stay within the preview viewport"
    );
}

#[test]
fn preview_scroll_content_uses_viewport_width_instead_of_intrinsic_text_width() {
    /* WHY: Verify that the overall width of the preview content is governed by the viewport width (with padding), 
     * ensuring that elements like headings or paragraph text start and wrap at correct offsets. */
    let markdown = "# PaddingHeading\n\n> Note: Long line wrapping test for viewport width constraints.";
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, std::path::Path::new("/tmp/preview-width.md"));

    let mut harness = Harness::builder()
        .with_size(egui::vec2(500.0, 280.0))
        .build_ui(move |ui| {
            // Simulate typical container padding (12px)
            egui::Frame::none().inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                pane.show(ui);
            });
        });

    harness.step();
    let heading = harness.get_by_label("PaddingHeading");
    let heading_rect = heading.accesskit_node().raw_bounds().unwrap();
    
    assert!(
        (heading_rect.x0 - 12.0).abs() <= 2.0,
        "preview heading should start at the viewport padding (12px), got left edge {}",
        heading_rect.x0
    );
}

#[test]
fn paragraph_with_inline_link_wraps_from_the_left_edge_after_link() {
    /* WHY: Verify that when a paragraph wraps mid-line after an inline link, the continuation line 
     * correctly restarts from the left edge of the paragraph column rather than having an offset. */
    let markdown = concat!(
        "KatanA Desktop is under active development. See the ",
        "[Releases page](https://example.com) ",
        "for the latest version and changelog.\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(markdown, std::path::Path::new("/tmp/inline-link-wrap.md"));

    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 220.0))
        .build_ui(move |ui| {
            pane.show_content(ui, None, None, None, None);
        });

    harness.step();
    harness.run();

    let tail_word = harness.get_by_label_contains("changelog.");
    let tail_bounds = tail_word.accesskit_node().raw_bounds().unwrap();
    let paragraph = harness.get_by_label_contains("KatanA Desktop");
    let paragraph_bounds = paragraph.accesskit_node().raw_bounds().unwrap();

    assert!(
        (tail_bounds.x0 - paragraph_bounds.x0).abs() <= 24.0,
        "text after an inline link should wrap from the left edge"
    );
}
