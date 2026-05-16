use crate::integration::harness_utils::flatten_shapes;
use eframe::egui;
use image::{ImageBuffer, Rgba};
use katana_core::markdown::{DiagramBlock, DiagramKind};
use katana_core::preview::{PreviewSection, PreviewSectionOps};
use katana_ui::preview_pane::{PreviewPane, RenderedSection, RendererLogicOps};
use std::path::Path;

fn render_preview_once(pane: &mut PreviewPane) {
    let ctx = egui::Context::default();
    let _ = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                pane.show_content(ui, None, None, None, None);
            });
        },
    );
}

fn write_test_png(path: &Path) {
    let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(2, 2, Rgba([0, 0, 0, 255]));
    image.save(path).unwrap();
}

#[test]
fn bare_anchor_img_inline_html_does_not_render_as_raw_text() {
    /* WHY: Regression guard for bug where bare HTML patterns like `<a><img></a>`
     * (unwrapped in <p>) were rendered as raw tag strings instead of being
     * processed or correctly ignored. */
    let md = "<a href=\"#\"><img src=\"test.png\" alt=\"Sponsor\"></a>\n";
    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut pane = PreviewPane::default();
            pane.update_markdown_sections(md, std::path::Path::new("/tmp/test.md"));
            pane.show_content(ui, None, None, None, None);
        });
    });

    let shapes: Vec<egui::Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
    let raw_html_visible = flatten_shapes(&shapes).into_iter().any(|shape| {
        if let egui::epaint::Shape::Text(text_shape) = shape {
            let txt = &text_shape.galley.job.text;
            txt.contains("<a href") || txt.contains("<img src")
        } else {
            false
        }
    });
    assert!(!raw_html_visible);
}

#[test]
fn p_align_center_strong_link_does_not_render_as_raw_text() {
    /* WHY: Regression guard for bug where complex nested HTML blocks like
     * `<p align="center"><strong><a>...</a></strong></p>` were
     * leaked as raw text in the preview. */
    let md = "<p align=\"center\"><strong><a href=\"#\">Link</a></strong></p>";
    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut pane = PreviewPane::default();
            pane.update_markdown_sections(md, std::path::Path::new("/tmp/test.md"));
            pane.show_content(ui, None, None, None, None);
        });
    });

    let shapes: Vec<egui::Shape> = output.shapes.into_iter().map(|cs| cs.shape).collect();
    let raw_html_visible = flatten_shapes(&shapes).into_iter().any(|shape| {
        if let egui::epaint::Shape::Text(text_shape) = shape {
            let txt = &text_shape.galley.job.text;
            txt.contains("<p align") || txt.contains("<strong>")
        } else {
            false
        }
    });
    assert!(!raw_html_visible);
}

#[test]
fn force_full_render_resets_local_image_viewer_state() {
    let temp_dir = tempfile::tempdir().unwrap();
    let image_path = temp_dir.path().join("diagram.png");
    write_test_png(&image_path);
    let document_path = temp_dir.path().join("sample.md");
    let source = format!("# Title\n\n![diagram](file://{})\n", image_path.display());
    let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());
    let mut pane = PreviewPane::default();
    pane.full_render(&source, &document_path, cache.clone(), true, 1);
    render_preview_once(&mut pane);

    pane.viewer_states[1].zoom_in();
    pane.viewer_states[1].pan_right();

    pane.full_render(&source, &document_path, cache, true, 1);
    render_preview_once(&mut pane);

    assert_eq!(pane.viewer_states[1].zoom, 1.0);
    assert_eq!(pane.viewer_states[1].pan, egui::Vec2::ZERO);
}

#[test]
fn sample_class_diagram_matches_single_render_when_rendered_with_other_diagrams() {
    let fixture_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.md");
    let source = std::fs::read_to_string(&fixture_path).unwrap();
    let sections = PreviewSectionOps::split_into_sections(&source);
    let (target_ordinal, target_source, target_lines) = sections
        .iter()
        .enumerate()
        .find_map(|(ordinal, section)| match section {
            PreviewSection::Diagram {
                kind: DiagramKind::Mermaid,
                source,
                lines,
            } if source.contains("class PreviewPane") => Some((ordinal, source, lines)),
            _ => None,
        })
        .expect("fixture should contain the 10.3 class diagram");
    let single_result = DiagramBlock {
        kind: DiagramKind::Mermaid,
        source: target_source.clone(),
    }
    .render();
    let single = RendererLogicOps::map_diagram_result(
        &DiagramKind::Mermaid,
        target_source,
        single_result,
        *target_lines,
    );
    let RenderedSection::Image {
        svg_data: single_svg,
        ..
    } = single
    else {
        panic!("single class diagram should render as image");
    };

    let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());
    let mut pane = PreviewPane::default();
    pane.full_render(&source, &fixture_path, cache, true, 10);
    pane.wait_for_renders();

    let RenderedSection::Image {
        svg_data: full_svg, ..
    } = &pane.sections[target_ordinal]
    else {
        panic!("full sample class diagram should render as image");
    };

    assert_eq!(full_svg.width, single_svg.width);
    assert_eq!(full_svg.height, single_svg.height);
    assert_eq!(full_svg.content_hash, single_svg.content_hash);
}

#[test]
fn sample_class_diagram_block_anchor_points_to_its_source_fence() {
    let fixture_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fixtures/sample.md");
    let source = std::fs::read_to_string(&fixture_path).unwrap();
    let cache = std::sync::Arc::new(katana_platform::InMemoryCacheService::default());
    let mut pane = PreviewPane::default();

    pane.full_render(&source, &fixture_path, cache, true, 10);
    pane.wait_for_renders();
    render_preview_once(&mut pane);

    assert!(
        pane.block_anchors
            .iter()
            .any(|(lines, _)| *lines == (392..411)),
        "10.3 class diagram must map to lines 393-411, anchors={:?}",
        pane.block_anchors
            .iter()
            .map(|(lines, _)| lines.clone())
            .collect::<Vec<_>>()
    );
}
