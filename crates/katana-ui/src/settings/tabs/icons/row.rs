use crate::theme_bridge::ThemeBridgeOps;
use katana_platform::theme::Rgba;

/* WHY: Operations for rendering a single row in the icon override table. */
pub(crate) struct IconsRowOps;

impl IconsRowOps {
    /* WHY: Renders the vendor selection column for an icon. */
    pub(crate) fn render_vendor_col(
        ui: &mut egui::Ui,
        icon_name: &str,
        current_vendor: &mut String,
        settings_changed: &mut bool,
    ) {
        crate::widgets::StyledComboBox::new(
            &format!("vendor_{}", icon_name),
            current_vendor.clone(),
        )
        .show(ui, |ui| {
            let is_selected = current_vendor == "default";
            let default_label = crate::i18n::I18nOps::get()
                .settings
                .icons
                .revert_default
                .clone();
            if ui
                .add(egui::Button::selectable(is_selected, default_label).frame_when_inactive(true))
                .clicked()
            {
                *current_vendor = "default".to_string();
                *settings_changed = true;
            }
            for p in crate::icon::AVAILABLE_PACKS {
                let pid = p.manifest().id.to_string();
                let is_pack_sel = *current_vendor == pid;
                if ui
                    .add(
                        egui::Button::selectable(is_pack_sel, pid.clone())
                            .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    *current_vendor = pid;
                    *settings_changed = true;
                }
            }
        });
    }

    /* WHY: Renders the color picker column for an icon. Delegate to IconsColorsOps. */
    pub(crate) fn render_color_col(
        ui: &mut egui::Ui,
        current_color: &mut Option<Rgba>,
        settings_changed: &mut bool,
        colorful_vendor_icons: bool,
    ) {
        crate::settings::tabs::icons::colors::IconsColorsOps::render_color_col(
            ui,
            current_color,
            settings_changed,
            colorful_vendor_icons,
        );
    }

    /* WHY: Renders the frame/border color picker column for an icon. Delegate to IconsColorsOps. */
    pub(crate) fn render_frame_color_col(
        ui: &mut egui::Ui,
        current_frame_color: &mut Option<Rgba>,
        settings_changed: &mut bool,
        colorful_vendor_icons: bool,
    ) {
        crate::settings::tabs::icons::colors::IconsColorsOps::render_frame_color_col(
            ui,
            current_frame_color,
            settings_changed,
            colorful_vendor_icons,
        );
    }

    /* WHY: Renders the resulting preview icon with all overrides applied. */
    pub(crate) fn render_preview_col(
        ui: &mut egui::Ui,
        state: &crate::app_state::AppState,
        icon: &crate::icon::Icon,
        current_vendor: &str,
        current_color: &Option<Rgba>,
        current_frame_color: &Option<Rgba>,
        colorful_vendor_icons: bool,
    ) {
        let image = icon.image(crate::icon::IconSize::Large);
        /* WHY: For preview, we manually apply the color */
        let apply_vendor = if current_vendor != "default" {
            current_vendor
        } else {
            &state.config.settings.settings().theme.icon_pack
        };

        let mut color = ui.visuals().text_color();

        if colorful_vendor_icons {
            if let Some(rgba) = current_color {
                color = ThemeBridgeOps::rgba_to_color32(*rgba);
            } else {
                let default_vendor_color =
                    icon.vendor_default_color(apply_vendor, ui.visuals().dark_mode);
                color = default_vendor_color.unwrap_or(color);
            }
        }

        let icon_bg = if ui.visuals().dark_mode {
            crate::theme_bridge::TRANSPARENT
        } else {
            crate::theme_bridge::ThemeBridgeOps::light_mode_icon_bg()
        };
        let mut btn = egui::Button::image(image.tint(color))
            .frame(false)
            .fill(icon_bg);
        if let Some(rgba) = current_frame_color.filter(|_| colorful_vendor_icons) {
            btn = btn.frame(true).fill(ThemeBridgeOps::rgba_to_color32(rgba));
        }

        ui.add(btn);
    }
}
