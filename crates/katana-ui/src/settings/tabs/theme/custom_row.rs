use katana_platform::settings::CustomTheme;

pub(super) struct CustomThemeRowOps;

impl CustomThemeRowOps {
    pub(super) const DELETE_BUTTON_WIDTH: f32 = 28.0;
    pub(super) const ROW_SPACING: f32 = 8.0;

    pub(super) fn render_swatch(
        ui: &mut egui::Ui,
        bg_color: egui::Color32,
        accent_color: egui::Color32,
    ) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(
                crate::settings::PRESET_SWATCH_SIZE,
                crate::settings::PRESET_SWATCH_SIZE,
            ),
            egui::Sense::hover(),
        );
        let corner = crate::settings::PRESET_SWATCH_SIZE / crate::settings::SWATCH_CORNER_DIVISOR;
        ui.painter().rect_filled(rect, corner, bg_color);
        ui.painter()
            .circle_filled(rect.center(), corner, accent_color);
    }

    pub(super) fn render_name_button(
        ui: &mut egui::Ui,
        custom_theme: &CustomTheme,
        is_selected: bool,
    ) -> egui::Response {
        let button_width =
            Self::name_button_width(ui.available_width(), ui.spacing().item_spacing.x);
        let height = ui.spacing().interact_size.y;
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(button_width, height), egui::Sense::click());
        if ui.is_rect_visible(rect) {
            Self::paint_name_button(ui, rect, &response, custom_theme, is_selected);
        }
        response
    }

    pub(super) fn render_delete_button(ui: &mut egui::Ui) -> egui::Response {
        ui.add_sized(
            [Self::DELETE_BUTTON_WIDTH, ui.spacing().interact_size.y],
            crate::Icon::Remove.button(ui, crate::icon::IconSize::Medium),
        )
        .on_hover_text(
            crate::i18n::I18nOps::get()
                .settings
                .theme
                .delete_custom
                .clone(),
        )
    }

    fn paint_name_button(
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        custom_theme: &CustomTheme,
        is_selected: bool,
    ) {
        let visuals = if is_selected {
            ui.visuals().widgets.active
        } else if response.hovered() {
            ui.visuals().widgets.hovered
        } else {
            ui.visuals().widgets.inactive
        };
        let bg_fill = if is_selected {
            ui.visuals().selection.bg_fill
        } else if response.hovered() {
            visuals.bg_fill
        } else {
            crate::theme_bridge::TRANSPARENT
        };
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, bg_fill);
        Self::paint_name_border(ui, rect, response, is_selected, visuals);
        let text_pos = rect.left_center() + egui::vec2(ui.spacing().button_padding.x, 0.0);
        ui.painter().text(
            text_pos,
            egui::Align2::LEFT_CENTER,
            &custom_theme.name,
            egui::TextStyle::Button.resolve(ui.style()),
            visuals.text_color(),
        );
    }

    fn paint_name_border(
        ui: &egui::Ui,
        rect: egui::Rect,
        response: &egui::Response,
        is_selected: bool,
        visuals: egui::style::WidgetVisuals,
    ) {
        if !response.hovered() && !is_selected {
            return;
        }
        let fill = if is_selected {
            ui.visuals().selection.bg_fill
        } else {
            visuals.weak_bg_fill
        };
        ui.painter().rect_filled(rect, visuals.corner_radius, fill);
    }

    fn name_button_width(available_width: f32, spacing: f32) -> f32 {
        (available_width - Self::DELETE_BUTTON_WIDTH - spacing).max(0.0)
    }
}
