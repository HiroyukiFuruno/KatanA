#![cfg(test)]

use crate::preview_pane::slideshow::render_slideshow_settings_sidebar;
use crate::state::layout::LayoutState;
use eframe::egui;

#[test]
fn slideshow_settings_sidebar_is_top_aligned() {
    let ctx = egui::Context::default();
    let mut layout_state = LayoutState::new();
    layout_state.slideshow_settings_open = true;

    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(1000.0, 800.0),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    let blocker_rect = ui.max_rect();
                    render_slideshow_settings_sidebar(
                        ctx,
                        ui,
                        &mut layout_state,
                        blocker_rect,
                        1.0,
                    );
                });
            });
        },
    );

    let mut all_texts = vec![];
    let text_shapes: Vec<(&egui::epaint::TextShape, egui::Rect)> = output
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::epaint::Shape::Text(text) => {
                all_texts.push(text.galley.job.text.clone());
                Some((text, clipped.clip_rect))
            }
            _ => None,
        })
        .collect();

    let toggle = text_shapes
        .iter()
        .find(|(t, _)| t.galley.job.text.contains("Hover Highlight"))
        .map(|t| t.0.visual_bounding_rect());

    assert!(toggle.is_some(), "Should render toggle");

    let toggle_rect = toggle.unwrap();
    let toggle_y = toggle_rect.top();

    assert!(
        toggle_y < 100.0,
        "Toggle should be top-aligned in the panel, got Y = {}",
        toggle_y
    );
}
