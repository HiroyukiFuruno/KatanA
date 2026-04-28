use super::*;

/* WHY: Tuning constants for title-bar layout measurement. */
const TITLE_ICON_PX: f32 = 12.0;
const TITLE_BTN_PADDING: f32 = 8.0;
const TITLE_SIDE_MIN: f32 = 56.0;
const TITLE_SIDE_EXTRA: f32 = 8.0;

impl<'a> Modal<'a> {
    pub(super) fn render_title_bar<T>(
        ui: &mut egui::Ui,
        title: &str,
        controls: ModalWindowControls,
        on_window_button: &mut impl FnMut(ModalWindowButton) -> Option<T>,
    ) -> Option<T> {
        let mut result = None;

        /* WHY: Allocate explicit left/center/right regions where the left and
        right regions have a fixed width computed from the expected number
        of control buttons. This ensures the title is truly centered in
        the remaining space regardless of modal width (auto or fullscreen). */

        /* WHY: Estimate button width: icon size + padding. Keep conservative margins. */
        let icon_px = TITLE_ICON_PX;
        let btn_padding = TITLE_BTN_PADDING;

        let left_count = if cfg!(target_os = "macos") {
            1 + if controls.show_fullscreen { 1 } else { 0 }
        } else {
            0
        };
        let right_count = if !cfg!(target_os = "macos") {
            1 + if controls.show_fullscreen { 1 } else { 0 }
        } else {
            0
        };

        let max_btns = left_count.max(right_count) as f32;
        let side_width =
            (max_btns * (icon_px + btn_padding)).max(TITLE_SIDE_MIN) + TITLE_SIDE_EXTRA;

        let available_width = ui.available_width();
        let row_height = ui.spacing().interact_size.y;
        let center_width = (available_width - side_width * 2.0).max(0.0);

        ui.allocate_ui_with_layout(
            egui::vec2(available_width, row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                /* WHY: left fixed-area */
                ui.allocate_ui_with_layout(
                    egui::vec2(side_width, row_height),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |left_ui| {
                        if cfg!(target_os = "macos")
                            && let Some(action) =
                                Self::render_window_controls(left_ui, controls, on_window_button)
                        {
                            result = Some(action);
                        }
                    },
                );

                /* WHY: center flexible-area */
                ui.allocate_ui_with_layout(
                    egui::vec2(center_width, row_height),
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |center_ui| {
                        center_ui.label(egui::RichText::new(title).strong());
                    },
                );
                /* WHY: right fixed-area */
                ui.allocate_ui_with_layout(
                    egui::vec2(side_width, row_height),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |right_ui| {
                        if !cfg!(target_os = "macos")
                            && let Some(action) =
                                Self::render_window_controls(right_ui, controls, on_window_button)
                        {
                            result = Some(action);
                        }
                    },
                );
            },
        );

        result
    }

    pub(super) fn render_window_controls<T>(
        ui: &mut egui::Ui,
        controls: ModalWindowControls,
        on_window_button: &mut impl FnMut(ModalWindowButton) -> Option<T>,
    ) -> Option<T> {
        let fullscreen_label = if controls.is_fullscreen {
            controls.exit_fullscreen_tooltip
        } else {
            controls.enter_fullscreen_tooltip
        };
        let mut result = None;

        let controls_layout = if cfg!(target_os = "macos") {
            egui::Layout::left_to_right(egui::Align::Center)
        } else {
            egui::Layout::right_to_left(egui::Align::Center)
        };

        ui.with_layout(controls_layout, |ui| {
            let close_btn = ui
                .add(
                    crate::Icon::Close
                        .button(ui, crate::icon::IconSize::Small)
                        .frame(false),
                )
                .on_hover_text(controls.close_tooltip);
            if close_btn.clicked() {
                result = on_window_button(ModalWindowButton::Close);
            }
            if controls.show_fullscreen {
                let fullscreen_btn = ui
                    .add(
                        crate::Icon::Fullscreen
                            .button(ui, crate::icon::IconSize::Small)
                            .frame(false),
                    )
                    .on_hover_text(fullscreen_label);
                if fullscreen_btn.clicked() {
                    result = on_window_button(ModalWindowButton::Fullscreen);
                }
            }
        });

        result
    }
}
