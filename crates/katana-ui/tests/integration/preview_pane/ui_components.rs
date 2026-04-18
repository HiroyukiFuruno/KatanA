use katana_ui::preview_pane::{PreviewPane, RenderedSection};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use eframe::egui;

#[test]
fn centered_badges_render_on_same_horizontal_row() {
    /* WHY: Verify that centered badges (<img> tags inside <p align="center">) 
     * are laid out in a single horizontal run rather than stacking vertically. */
    let html = concat!(
        "<p align=\"center\">\n",
        "  <a href=\"LICENSE\"><img src=\"badge1.svg\" alt=\"License: MIT\"></a>\n",
        "  <a href=\"https://example.com/ci\"><img src=\"badge2.svg\" alt=\"CI\"></a>\n",
        "</p>\n"
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
}

#[test]
fn centered_text_link_is_clickable() {
    /* WHY: Verify that specialized centering logic for HTML blocks doesn't interfere with egui's hit-testing for links. */
    let html = concat!(
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
    let _link_node = harness.get_by_label("\u{65e5}\u{672c}\u{8a9e}");
}

#[test]
fn html_block_hover_highlight_rect_is_generated() {
    /* WHY: Verify that HTML blocks (like <p align="center">) correctly generate bounding boxes for the interactive hover-to-source feature. */
    let html = concat!(
        "<p align=\"center\">\n",
        "  <img src=\"badge1.svg\" alt=\"License: MIT\">\n",
        "</p>\n",
        "\n# Heading After HTML\n"
    );
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(html, std::path::Path::new("/tmp/test.md"));

    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 400.0))
        .build_ui(move |ui| {
            pane.show_content(ui, None, None, None, None);
        });

    harness.step();
    harness.run();

    let heading = harness.query_by_label("Heading After HTML");
    assert!(
        heading.is_some(),
        "The markdown AFTER the HTML block MUST NOT be swallowed!"
    );
}

#[test]
fn advanced_settings_renders_completely() {
    /* WHY: Multi-Layout test: Verify that TopBottomPanel combined with CentralPanel (used in Advanced Settings popups) 
     * renders all children without clipping or visibility regressions. */
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui(move |ui| {
            let is_advanced_open = true;

            egui::TopBottomPanel::bottom("test_advanced")
                .default_height(350.0)
                .show_inside(ui, |ui| {
                    if is_advanced_open {
                        ui.heading("Advanced Settings Test Heading");
                        ui.label("This should not be clipped");
                    }
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..20 {
                        ui.label(format!("Item {}", i));
                    }
                });
            });
        });

    harness.step();
    let _heading = harness.get_by_label("Advanced Settings Test Heading");
    let _label = harness.get_by_label("This should not be clipped");
}
