use eframe::egui;

const SPLASH_REPAINT_INTERVAL_MS: u64 = 32;

const SPLASH_BG_DARK: u8 = 30;
const SPLASH_BG_LIGHT: u8 = 240;
const SPLASH_ICON_SIZE: f32 = 128.0;
const SPLASH_ICON_SPACING: f32 = 16.0;
const SPLASH_HEADING_SIZE: f32 = 32.0;
const SPLASH_HEADING_SPACING: f32 = 8.0;
const SPLASH_VERSION_SIZE: f32 = 16.0;
const SPLASH_PROGRESS_SPACING: f32 = 24.0;
const SPLASH_PROGRESS_WIDTH: f32 = 240.0;
const SPLASH_PROGRESS_PHASE1: f32 = 0.25;
const SPLASH_PROGRESS_PHASE2: f32 = 0.6;
const SPLASH_PROGRESS_PHASE3: f32 = 0.95;
const SPLASH_PROGRESS_TEXT_SIZE: f32 = 12.0;
const SPLASH_PROGRESS_TEXT_DIM: f32 = 0.7;
const SPLASH_PROGRESS_BAR_MARGIN: f32 = 4.0;
const SPLASH_PROGRESS_BG_LIGHT: u8 = 100;
const SPLASH_PROGRESS_BG_DARK: u8 = 200;

const SPLASH_CONTENT_HEIGHT: f32 = SPLASH_ICON_SIZE
    + SPLASH_ICON_SPACING
    + SPLASH_HEADING_SIZE
    + SPLASH_HEADING_SPACING
    + SPLASH_VERSION_SIZE
    + SPLASH_PROGRESS_SPACING
    + SPLASH_PROGRESS_TEXT_SIZE
    + SPLASH_PROGRESS_BAR_MARGIN
    + SPLASH_PROGRESS_SPACING;

pub(crate) struct SplashOverlay<'a> {
    pub elapsed: f32,
    pub about_icon: Option<&'a egui::TextureHandle>,
}

impl<'a> SplashOverlay<'a> {
    pub fn new(elapsed: f32, about_icon: Option<&'a egui::TextureHandle>) -> Self {
        Self {
            elapsed,
            about_icon,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let opacity = crate::shell_logic::calculate_splash_opacity(self.elapsed);
        if opacity <= 0.0 || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            return true;
        }

        egui::Area::new(egui::Id::new("splash_screen_area"))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| self.draw_splash_content(ui, ctx, opacity));

        ctx.request_repaint_after(std::time::Duration::from_millis(SPLASH_REPAINT_INTERVAL_MS));
        false
    }

    fn draw_splash_content(&self, ui: &mut egui::Ui, ctx: &egui::Context, opacity: f32) {
        let is_dark = ctx.style().visuals.dark_mode;
        #[allow(deprecated)]
        let content_rect = ctx.screen_rect();

        let bg = if is_dark {
            SPLASH_BG_DARK
        } else {
            SPLASH_BG_LIGHT
        };
        let fill = crate::theme_bridge::from_rgb(bg, bg, bg).gamma_multiply(opacity);
        ui.painter().rect_filled(content_rect, 1.0, fill);

        let center = content_rect.center();
        let rect = egui::Rect::from_center_size(
            center,
            egui::vec2(content_rect.width(), SPLASH_CONTENT_HEIGHT),
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.vertical_centered(|ui| self.draw_inner_elements(ui, is_dark, opacity));
        });
    }

    fn draw_inner_elements(&self, ui: &mut egui::Ui, is_dark: bool, opacity: f32) {
        let text_color = if is_dark {
            crate::theme_bridge::WHITE
        } else {
            crate::theme_bridge::BLACK
        }
        .gamma_multiply(opacity);

        if let Some(tex) = self.about_icon {
            let size = egui::vec2(SPLASH_ICON_SIZE, SPLASH_ICON_SIZE);
            ui.image(egui::load::SizedTexture::new(tex.id(), size));
            ui.add_space(SPLASH_ICON_SPACING);
        }

        let heading = egui::RichText::new(crate::about_info::APP_DISPLAY_NAME)
            .strong()
            .size(SPLASH_HEADING_SIZE)
            .color(text_color);
        ui.label(heading);
        ui.add_space(SPLASH_HEADING_SPACING);

        let version = format!("Version {}", env!("CARGO_PKG_VERSION"));
        ui.label(
            egui::RichText::new(version)
                .size(SPLASH_VERSION_SIZE)
                .color(text_color),
        );
        ui.add_space(SPLASH_PROGRESS_SPACING);

        self.draw_progress_bar(ui, text_color, is_dark, opacity);
    }

    fn draw_progress_bar(
        &self,
        ui: &mut egui::Ui,
        text_color: egui::Color32,
        is_dark: bool,
        opacity: f32,
    ) {
        let progress = crate::shell_logic::calculate_splash_progress(self.elapsed);
        let text = if progress < SPLASH_PROGRESS_PHASE1 {
            "Initializing Katana engine..."
        } else if progress < SPLASH_PROGRESS_PHASE2 {
            "Parsing workspace structure..."
        } else if progress < SPLASH_PROGRESS_PHASE3 {
            "Increasing context size... w"
        } else {
            "Ready."
        };

        ui.label(
            egui::RichText::new(text)
                .size(SPLASH_PROGRESS_TEXT_SIZE)
                .color(text_color.gamma_multiply(SPLASH_PROGRESS_TEXT_DIM)),
        );
        ui.add_space(SPLASH_PROGRESS_BAR_MARGIN);

        let bg = if is_dark {
            SPLASH_PROGRESS_BG_DARK
        } else {
            SPLASH_PROGRESS_BG_LIGHT
        };
        ui.visuals_mut().selection.bg_fill =
            crate::theme_bridge::from_rgb(bg, bg, bg).gamma_multiply(opacity);

        ui.add(
            egui::ProgressBar::new(progress)
                .desired_width(SPLASH_PROGRESS_WIDTH)
                .show_percentage(),
        );
    }
}
