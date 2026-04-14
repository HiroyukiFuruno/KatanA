use super::*;

#[test]
fn ensure_svg_extension_inserts_suffix_before_query_string() {
    let url =
        "https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors";

    let normalized = ensure_svg_extension(url);

    assert_eq!(
        normalized,
        "https://img.shields.io/badge/Sponsor-%E2%9D%A4%EF%B8%8F-ea4aaa.svg?style=for-the-badge&logo=github-sponsors"
    );
}

#[test]
fn ensure_svg_extension_preserves_existing_svg_suffix_before_query_string() {
    let url = "https://img.shields.io/badge/License-MIT-blue.svg?style=flat";

    assert_eq!(ensure_svg_extension(url), url);
}

#[test]
fn heading_with_align_center_is_centered() {
    use eframe::egui;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    let html = "<h1 align=\"center\">Centered Heading</h1>";
    let parser = katana_core::html::HtmlParser::new(std::path::Path::new("."));
    let nodes = parser.parse(html);

    let mut harness = Harness::builder()
        .with_size(egui::vec2(600.0, 400.0))
        .build_ui(move |ui| {
            ui.set_width(600.0);
            let renderer = HtmlRenderer::new(ui, std::path::Path::new("."));
            renderer.render(&nodes);
        });

    harness.step();

    let label = harness.get_by_label("Centered Heading");
    let bounds = label
        .accesskit_node()
        .raw_bounds()
        .expect("heading must have bounds");

    assert!(
        bounds.x0 > 200.0,
        "Heading with align='center' should be centered, but its x0 is {:.1}",
        bounds.x0
    );
}
