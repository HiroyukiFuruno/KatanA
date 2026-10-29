use super::{PreviewPane, RenderedSection};
use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};
use std::path::Path;

const VIEWPORT: egui::Vec2 = egui::vec2(640.0, 480.0);

fn pending_loading_bounds(document_path: &Path, image_path: &Path) -> egui::Rect {
    let mut pane = PreviewPane::default();
    pane.md_file_path = document_path.to_path_buf();
    pane.sections.push(RenderedSection::LocalImage {
        path: image_path.to_path_buf(),
        alt: "loading image".to_owned(),
        source_lines: 1,
    });
    pane.viewer_states.push(super::ViewerState::default());
    let label = crate::widgets::LoadingPresentation::new(
        crate::widgets::LoadingState::Pending,
        crate::widgets::LoadingLabel::RenderingKind("image"),
    )
    .label;
    let harness = Harness::builder().with_size(VIEWPORT).build_ui(|ui| {
        pane.render_sections(ui, None, None, None, None, false);
    });
    let spinner = harness.get_by_role(egui::accesskit::Role::ProgressIndicator);
    spinner.rect().union(harness.get_by_label(&label).rect())
}

#[test]
fn standalone_png_loading_is_centered_while_markdown_image_loading_stays_inline() {
    let directory = tempfile::tempdir().expect("fixture directory");
    let image_path = directory.path().join("standalone.png");
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]))
        .save(&image_path)
        .expect("PNG fixture");
    let markdown_path = directory.path().join("notes.md");

    let standalone_bounds = pending_loading_bounds(&image_path, &image_path);
    let markdown_bounds = pending_loading_bounds(&markdown_path, &image_path);

    assert!((standalone_bounds.center().x - VIEWPORT.x / 2.0).abs() <= 1.0);
    assert!((standalone_bounds.center().y - VIEWPORT.y / 2.0).abs() <= 1.0);
    assert!((markdown_bounds.center().x - VIEWPORT.x / 2.0).abs() <= 1.0);
    assert!(
        markdown_bounds.center().y < 96.0,
        "inline loading should remain in content flow at y={}",
        markdown_bounds.center().y
    );
}
