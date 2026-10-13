use katana_platform::theme::types::Rgba;

/* WHY: Operations for rendering icon color customization columns. */
pub(crate) struct IconsColorsOps;

impl IconsColorsOps {
    /* WHY: Renders the primary color picker column for an icon. */
    pub(crate) fn render_color_col(
        _ui: &mut egui::Ui,
        current_color: &mut Option<Rgba>,
        settings_changed: &mut bool,
        colorful_vendor_icons: bool,
    ) {
        use crate::theme_bridge::ThemeBridgeOps;
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                ui.add_enabled_ui(colorful_vendor_icons, |ui| {
                    use std::cell::Cell;
                    let changed = Cell::new(false);
                    let new_color = Cell::new(*current_color);

                    let mut c = ThemeBridgeOps::rgba_to_color32(
                        current_color
                            .unwrap_or(ThemeBridgeOps::color32_to_rgba(ui.visuals().text_color())),
                    );
                    let picker = crate::widgets::InlineColorPicker::new().rgba(true);
                    let response = picker.show(ui, &mut c);
                    if response.changed() {
                        let [r, g, b, a] = c.to_srgba_unmultiplied();
                        new_color.set(Some(Rgba { r, g, b, a }));
                        changed.set(true);
                    }

                    if changed.get() {
                        *current_color = new_color.get();
                        *settings_changed = true;
                    }
                });
            })
            .show(_ui);
    }

    /* WHY: Renders the frame/border color picker column for an icon. */
    pub(crate) fn render_frame_color_col(
        _ui: &mut egui::Ui,
        current_frame_color: &mut Option<Rgba>,
        settings_changed: &mut bool,
        colorful_vendor_icons: bool,
    ) {
        use crate::theme_bridge::ThemeBridgeOps;
        crate::widgets::AlignCenter::new()
            .content(|ui| {
                ui.add_enabled_ui(colorful_vendor_icons, |ui| {
                    use std::cell::Cell;
                    let changed = Cell::new(false);
                    let new_color = Cell::new(*current_frame_color);

                    let mut c = ThemeBridgeOps::rgba_to_color32(current_frame_color.unwrap_or(
                        ThemeBridgeOps::color32_to_rgba(ui.visuals().weak_text_color()),
                    ));
                    let picker = crate::widgets::InlineColorPicker::new().rgba(true);
                    let response = picker.show(ui, &mut c);
                    if response.changed() {
                        let [r, g, b, a] = c.to_srgba_unmultiplied();
                        new_color.set(Some(Rgba { r, g, b, a }));
                        changed.set(true);
                    }

                    if changed.get() {
                        *current_frame_color = new_color.get();
                        *settings_changed = true;
                    }
                });
            })
            .show(_ui);
    }
}
