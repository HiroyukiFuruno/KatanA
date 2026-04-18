use katana_ui::preview_pane::PreviewPane;
use eframe::egui;
use crate::integration::harness_utils::flatten_clipped_shapes;

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
