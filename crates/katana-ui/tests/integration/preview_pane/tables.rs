use katana_ui::preview_pane::PreviewPane;
use eframe::egui;
use crate::integration::harness_utils::flatten_clipped_shapes;

#[test]
fn markdown_table_stretches_to_full_width() {
    /* WHY: Verify that markdown tables correctly expand to fill the entire available panel width, 
     * adhering to our "100% width" design requirement. */
    let table_md = "| Header A | Header B | Header C |\n|---|---|---|\n| Cell 1 | Cell 2 | Cell 3 |\n";
    let preview_width = 600.0_f32;
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(table_md, std::path::Path::new("/tmp/table_test.md"));

    let ctx = egui::Context::default();
    let raw_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(preview_width, 400.0))),
        ..Default::default()
    };

    let render = |ctx: &egui::Context, pane: &mut PreviewPane| {
        ctx.run(raw_input.clone(), |ctx| {
            egui::CentralPanel::default().frame(egui::Frame::NONE.inner_margin(egui::Margin::same(8))).show(ctx, |ui| {
                pane.show(ui);
            });
        })
    };

    let _ = render(&ctx, &mut pane);
    let output = render(&ctx, &mut pane);

    let content_width = preview_width - 16.0;
    let flat = flatten_clipped_shapes(&output.shapes);
    let mut table_frame_rect = egui::Rect::NOTHING;
    for s in flat.iter() {
        if let egui::epaint::Shape::Rect(rect_shape) = s {
            if rect_shape.stroke.width > 0.0 && rect_shape.rect.width() > 50.0 {
                table_frame_rect = rect_shape.rect;
                break;
            }
        }
    }

    assert!(table_frame_rect.width() > 0.0);
    let fill_ratio = table_frame_rect.width() / content_width;
    assert!((fill_ratio - 1.0).abs() < 0.05);
}

#[test]
fn markdown_table_has_visible_vertical_lines() {
    /* WHY: Verify that vertical lines between table columns are rendered with sufficient 
     * visibility (width and opacity), which is a common regression point in custom grid rendering. */
    let table_md = "| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n";
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(table_md, std::path::Path::new("/tmp/vline_test.md"));

    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| pane.show(ui));
    });

    let flat = flatten_clipped_shapes(&output.shapes);
    let vlines = flat.iter().filter(|s| match s {
        egui::epaint::Shape::LineSegment { points, .. } => (points[0].x - points[1].x).abs() < 1.0 && (points[0].y - points[1].y).abs() > 5.0,
        _ => false,
    }).count();
    assert!(vlines >= 2);
}

#[test]
fn markdown_table_max_width_is_constrained() {
    /* WHY: Verify that tables with extremely long content do not expand beyond the available 
     * viewport width, ensuring they wrap correctly. */
    let table_md = "| Header A | Header B |\n|---|---|\n| This is a very very extremely long sentence that should wrap | Short |\n";
    let preview_width = 400.0_f32;
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections(table_md, std::path::Path::new("/tmp/table_wrap.md"));

    let ctx = egui::Context::default();
    let output = ctx.run(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(preview_width, 400.0))),
        ..Default::default()
    }, |ctx| {
        egui::CentralPanel::default().frame(egui::Frame::NONE.inner_margin(egui::Margin::same(8))).show(ctx, |ui| pane.show(ui));
    });

    let flat = flatten_clipped_shapes(&output.shapes);
    let mut table_rect = egui::Rect::NOTHING;
    for s in flat {
        if let egui::epaint::Shape::Rect(r) = s {
            if r.rect.width() > 50.0 { table_rect = r.rect; break; }
        }
    }
    assert!(table_rect.width() <= (preview_width - 16.0));
}

