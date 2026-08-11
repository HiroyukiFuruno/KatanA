use eframe::egui;

use super::super::painter_tests::grid_cell;

const VIEWPORT_WIDTH: f32 = 300.0;
const VIEWPORT_HEIGHT: f32 = 200.0;

#[test]
fn cell_style_handles_clipping_selection_grid_lines_and_color_fallbacks() {
    let context = crate::test_ui::Context::default();
    context.run_ui(egui::RawInput::default(), |ui| {
        let viewport = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        );
        let painter = ui.painter().with_clip_rect(viewport);

        let mut clipped = grid_cell();
        clipped.clipped_bounds.width = 0;
        super::paint_cell(ui, &painter, viewport, &clipped, false);

        let mut selected = grid_cell();
        selected.selected = true;
        selected.appearance.fill_color = Some("invalid".to_owned());
        super::paint_cell(ui, &painter, viewport, &selected, true);

        let mut active = grid_cell();
        active.active = true;
        active.appearance.fill_color = Some("#123456".to_owned());
        super::paint_cell(ui, &painter, viewport, &active, false);
    });

    assert_eq!(
        super::parse_color("#123456"),
        Some(egui::Color32::from_rgb(0x12, 0x34, 0x56))
    );
    assert_eq!(super::parse_color("invalid"), None);
    assert_eq!(
        super::translated(
            egui::pos2(5.0, 6.0),
            katana_document_viewer::DocumentRect {
                x: 2,
                y: 3,
                width: 4,
                height: 5,
            }
        ),
        egui::Rect::from_min_size(egui::pos2(7.0, 9.0), egui::vec2(4.0, 5.0))
    );
}
