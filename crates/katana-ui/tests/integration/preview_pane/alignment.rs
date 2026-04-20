use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use egui_kittest::{Harness, kittest::{NodeT, Queryable}};
use eframe::egui;

#[test]
fn multiple_centered_paragraphs_have_increasing_y_positions() {
    /* WHY: Verify that our custom centering layout for HTML blocks (like <p align="center">) 
     * correctly updates the cursor Y-position for subsequent blocks, preventing overlapping. */
    let html = concat!(
        "<p align=\"center\">First paragraph</p>\n\n",
        "<p align=\"center\">Second paragraph</p>\n\n",
        "<p align=\"center\">Third paragraph</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string(), 1)];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    let first_bounds = harness.get_by_label("First paragraph").accesskit_node().raw_bounds().unwrap();
    let second_bounds = harness.get_by_label("Second paragraph").accesskit_node().raw_bounds().unwrap();
    let third_bounds = harness.get_by_label("Third paragraph").accesskit_node().raw_bounds().unwrap();

    assert!(second_bounds.y0 > first_bounds.y0);
    assert!(third_bounds.y0 > second_bounds.y0);
}

#[test]
fn readme_header_full_structure_renders() {
    /* WHY: Verify that a complex, typical README header with icons, headings, and badges 
     * all rendered using HTML alignment tags, displays correctly without missing elements. */
    let html = concat!(
        "<h1 align=\"center\">KatanA Desktop</h1>\n\n",
        "<p align=\"center\">\n",
        "  A fast, lightweight Markdown workspace for macOS.\n",
        "</p>\n\n",
        "<p align=\"center\">\n",
        "  English | <a href=\"README.ja.md\">\u{65e5}\u{672c}\u{8a9e}</a>\n",
        "</p>\n"
    );
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string(), 1)];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui, None, None, None, None);
    });
    harness.step();
    harness.step();
    harness.run();

    let _heading = harness.get_by_label("KatanA Desktop");
    let _description = harness.get_by_label("A fast, lightweight Markdown workspace for macOS.");
}

#[test]
fn centered_single_text_is_horizontally_centered() {
    /* WHY: Verify the geometric centering of HTML aligned text. 
     * The center of the text bounding box should match the center of the available panel width. */
    let html = "<p align=\"center\">Centered Text Here</p>\n";
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Markdown(html.to_string(), 1)];

    let panel_width: f32 = 800.0;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(panel_width, 200.0))
        .build_ui(move |ui| {
            pane.show_content(ui, None, None, None, None);
        });
    harness.step();
    harness.step();
    harness.step();
    harness.run();

    let bounds = harness.get_by_label("Centered Text Here").accesskit_node().raw_bounds().unwrap();
    let widget_center_x = (bounds.x0 + bounds.x1) / 2.0;
    let panel_center_x = f64::from(panel_width) / 2.0;

    assert!(
        (widget_center_x - panel_center_x).abs() <= 50.0,
        "Text center {} must be near panel center {}",
        widget_center_x,
        panel_center_x
    );
}
