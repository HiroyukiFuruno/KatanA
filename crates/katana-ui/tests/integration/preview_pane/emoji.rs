use katana_ui::preview_pane::PreviewPane;
use katana_core::markdown::color_preset::DiagramColorPreset;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use eframe::egui;

#[test]
fn emoji_rendered_as_image_using_dedicated_painter() {
    /* WHY: Verify that emojis are no longer rendered as text glyphs (which often fail to display correctly in egui) 
     * but instead as images using our custom emoji replacement logic. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("Hello 🌍", std::path::Path::new("/tmp/emoji.md"));
    let mut harness = Harness::new_ui(move |ui| {
        katana_ui::font_loader::SystemFontLoader::setup_fonts(
            ui.ctx(),
            DiagramColorPreset::current(),
            None,
            None,
        );
        pane.show_content(ui, None, None, None, None);
    });
    harness.run();
    harness.get_by_role(egui::accesskit::Role::Image);
}

#[test]
#[cfg(target_os = "macos")]
fn inline_emoji_stays_within_text_line_height_budget() {
    /* WHY: Verify that substituted emoji images have correct dimensions and vertical alignment 
     * relative to the surrounding paragraph text, ensuring they don't cause line jumping. */
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(
        "👉 Become a Sponsor",
        std::path::Path::new("/tmp/emoji-line.md"),
    );
    let mut harness = Harness::new_ui(move |ui| {
        katana_ui::font_loader::SystemFontLoader::setup_fonts(
            ui.ctx(),
            DiagramColorPreset::current(),
            None,
            None,
        );
        pane.show_content(ui, None, None, None, None);
    });
    harness.run();

    let image = harness.get_by_role(egui::accesskit::Role::Image);
    let image_bounds = image.accesskit_node().raw_bounds().expect("emoji image should have bounds");
    let text = harness.get_by_label("Sponsor");
    let text_bounds = text.accesskit_node().raw_bounds().expect("text should have bounds");

    let image_height = image_bounds.y1 - image_bounds.y0;
    let text_height = text_bounds.y1 - text_bounds.y0;

    assert!(
        image_height <= text_height * 1.15,
        "inline emoji should stay near text line height"
    );
}
