use super::types::*;

pub const DEFAULT_BAR_WIDTH: f32 = 280.0;
pub const DEFAULT_DIALOG_WIDTH: f32 = 450.0;
pub const BODY_TO_BAR_SPACING: f32 = 12.0;
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
        }
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
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

    pub fn show<T>(
        self,
        ctx: &egui::Context,
        body: impl FnOnce(&mut egui::Ui),
        footer: impl FnOnce(&mut egui::Ui) -> Option<T>,
    ) -> Option<T> {
        let mut result: Option<T> = None;

        let dialog_width = self.width.unwrap_or(DEFAULT_DIALOG_WIDTH);

        egui::Window::new(self.title)
            .id(egui::Id::new(self.id))
            .collapsible(false)
            .resizable(false)
            .default_width(dialog_width)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_max_width(dialog_width);

                ui.vertical_centered(|ui| {
                    ui.set_max_width(dialog_width);
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
                // allow(horizontal_layout)
                ui.horizontal(|ui| {
                    ui.set_max_width(dialog_width);
                    result = footer(ui);
                });
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
