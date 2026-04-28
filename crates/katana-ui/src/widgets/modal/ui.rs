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
            .default_width(dialog_width);
        /* WHY: Allow resizing only when a fixed size (fullscreen) is supplied. */
        if fixed_size.is_some() {
            window = window.resizable(true);
        } else {
            window = window.resizable(false);
        }
        /* WHY: Position and other window setup handled below (keeps previous behavior). */
        if has_controls {
            window = window.title_bar(false);
        }
        if let Some(frame) = self.frame {
            window = window.frame(frame);
        }

        let response = window.show(ctx, |ui| {
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
        if let Some(response) = response {
            crate::widgets::InteractionFacade::register_hover_blocker(ctx, response.response.rect);
        }

        result
    }

    pub fn show_body_only(self, ctx: &egui::Context, body: impl FnOnce(&mut egui::Ui)) {
        self.show(ctx, body, |_ui| None::<()>);
    }
}
