use super::types::*;

pub const DEFAULT_BAR_WIDTH: f32 = 280.0;
pub const DEFAULT_DIALOG_WIDTH: f32 = 450.0;
pub const BODY_TO_BAR_SPACING: f32 = 12.0;
pub const HEADER_TO_BODY_SPACING: f32 = 10.0;
pub const BAR_TO_FOOTER_SPACING: f32 = 16.0;

impl<'a> Modal<'a> {
    pub fn new(id: &'a str, title: &'a str) -> Self {
        Self {
            id,
            title,
            progress: None,
            show_pct: false,
            bar_width: DEFAULT_BAR_WIDTH,
            width: None,
            fixed_size: None,
            frame: None,
            window_controls: None,
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn fixed_size(mut self, size: egui::Vec2) -> Self {
        self.fixed_size = Some(size);
        self
    }

    pub fn frame(mut self, frame: egui::Frame) -> Self {
        self.frame = Some(frame);
        self
    }

    pub fn progress(mut self, ratio: f32) -> Self {
        self.progress = Some(ratio.clamp(0.0, 1.0));
        self
    }

    pub fn maybe_progress(mut self, ratio: Option<f32>) -> Self {
        self.progress = ratio.map(|r| r.clamp(0.0, 1.0));
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_pct = show;
        self
    }

    pub fn bar_width(mut self, width: f32) -> Self {
        self.bar_width = width;
        self
    }

    pub fn window_controls(mut self, controls: ModalWindowControls<'a>) -> Self {
        self.window_controls = Some(controls);
        self
    }

    pub fn show<T>(
        self,
        ctx: &egui::Context,
        body: impl FnOnce(&mut egui::Ui),
        footer: impl FnOnce(&mut egui::Ui) -> Option<T>,
    ) -> Option<T> {
        self.show_with_controls(ctx, |_ui| Option::<T>::None, body, footer, |_| None)
    }

    pub fn show_with_controls<T>(
        self,
        ctx: &egui::Context,
        header: impl FnOnce(&mut egui::Ui) -> Option<T>,
        body: impl FnOnce(&mut egui::Ui),
        footer: impl FnOnce(&mut egui::Ui) -> Option<T>,
        mut on_window_button: impl FnMut(ModalWindowButton) -> Option<T>,
    ) -> Option<T> {
        let mut result: Option<T> = None;
        let dialog_width = self.width.unwrap_or(DEFAULT_DIALOG_WIDTH);
        let window_controls = self.window_controls;
        let has_controls = window_controls.is_some();
        let fixed_size = self.fixed_size;
        let content_width = fixed_size.map_or(dialog_width, |size| size.x);

        let mut window = egui::Window::new(self.title)
            .id(egui::Id::new(self.id))
            .collapsible(false)
            .resizable(false)
            .default_width(dialog_width)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO);
        if let Some(size) = fixed_size {
            window = window.fixed_size(size);
        }
        if has_controls {
            window = window.title_bar(false);
        }
        if let Some(frame) = self.frame {
            window = window.frame(frame);
        }

        window.show(ctx, |ui| {
            ui.set_max_width(content_width);

            if has_controls {
                let controls = window_controls.expect("modal controls are initialized");
                if let Some(action) =
                    Self::render_title_bar(ui, self.title, controls, &mut on_window_button)
                {
                    result = Some(action);
                }
                ui.add_space(HEADER_TO_BODY_SPACING);
            }

            ui.vertical_centered(|ui| {
                ui.set_max_width(content_width);

                let header_result = header(ui);
                if header_result.is_some() {
                    result = header_result;
                }

                body(ui);

                if let Some(ratio) = self.progress {
                    ui.add_space(BODY_TO_BAR_SPACING);
                    let mut bar = egui::ProgressBar::new(ratio).desired_width(self.bar_width);
                    if self.show_pct {
                        bar = bar.show_percentage();
                    }
                    ui.add(bar);
                }
            });

            ui.add_space(BAR_TO_FOOTER_SPACING);
            let mut footer_result = None;
            crate::widgets::AlignCenter::new()
                .shrink_to_fit(true)
                .content(|ui| {
                    ui.set_max_width(content_width);
                    footer_result = footer(ui);
                })
                .show(ui);
            if footer_result.is_some() {
                result = footer_result;
            }
        });

        result
    }

    fn render_title_bar<T>(
        ui: &mut egui::Ui,
        title: &str,
        controls: ModalWindowControls,
        on_window_button: &mut impl FnMut(ModalWindowButton) -> Option<T>,
    ) -> Option<T> {
        let mut result = None;

        ui.columns_const::<3, _>(|[left, center, right]| {
            if cfg!(target_os = "macos") {
                result = Self::render_window_controls(left, controls, on_window_button);
            } else {
                result = Self::render_window_controls(right, controls, on_window_button);
            }

            center.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.label(egui::RichText::new(title).strong());
                },
            );
        });

        result
    }

    fn render_window_controls<T>(
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

    pub fn show_body_only(self, ctx: &egui::Context, body: impl FnOnce(&mut egui::Ui)) {
        self.show(ctx, body, |_ui| None::<()>);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_builder_defaults() {
        let modal = Modal::new("t", "Title");
        assert_eq!(modal.progress, None);
        assert!(!modal.show_pct);
        assert!((modal.bar_width - DEFAULT_BAR_WIDTH).abs() < f32::EPSILON);
    }

    #[test]
    fn test_modal_builder_with_progress() {
        let modal = Modal::new("t", "T")
            .progress(0.5)
            .show_percentage(true)
            .bar_width(200.0);
        assert_eq!(modal.progress, Some(0.5));
        assert!(modal.show_pct);
        assert!((modal.bar_width - 200.0).abs() < f32::EPSILON);
    }
}
