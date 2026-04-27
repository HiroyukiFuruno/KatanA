use eframe::egui;

pub(super) const PADDING_X: f32 = 8.0;
pub(super) const PADDING_Y: f32 = 6.0;
pub(super) const ROUNDING_RADIUS: f32 = 6.0;
pub(super) const ITEM_SPACING: f32 = 4.0;
pub(super) const MIN_TEXT_INPUT_WIDTH: f32 = 64.0;

const TOGGLE_BUTTON_COUNT: usize = 3;

pub(super) fn content_width(ui: &egui::Ui, desired_width: Option<f32>) -> f32 {
    let outer_width = desired_width.unwrap_or_else(|| ui.available_width());
    (outer_width - PADDING_X * 2.0).max(MIN_TEXT_INPUT_WIDTH)
}

pub(super) fn trailing_width(ui: &egui::Ui, has_toggles: bool, has_clear_button: bool) -> f32 {
    let button_count =
        usize::from(has_clear_button) + if has_toggles { TOGGLE_BUTTON_COUNT } else { 0 };
    if button_count == 0 {
        return 0.0;
    }
    let button_width = ui.spacing().interact_size.x;
    let spacing_width = ui.spacing().item_spacing.x * button_count as f32;
    button_width * button_count as f32 + spacing_width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_width_reserves_frame_padding_inside_requested_width() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let width = content_width(ui, Some(240.0));

                assert_eq!(width, 224.0);
            });
        });
    }

    #[test]
    fn trailing_width_reserves_clear_and_toggle_buttons() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let clear_only = trailing_width(ui, false, true);
                let clear_and_toggles = trailing_width(ui, true, true);

                assert!(clear_only > 0.0);
                assert!(clear_and_toggles > clear_only);
            });
        });
    }
}
