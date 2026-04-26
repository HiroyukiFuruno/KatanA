/* WHY: Renders a complete icon override row. Extracted to prevent file length limits. */
pub(crate) struct IconsTableRowOps;

impl IconsTableRowOps {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_row(
        row: &mut egui_extras::TableRow<'_, '_>,
        state: &mut crate::app_state::AppState,
        icon: &crate::icon::Icon,
        icon_settings: &mut katana_platform::settings::types::icon::IconSettings,
        settings_changed: &mut bool,
    ) {
        let icon_name = icon.name();

        row.col(|ui| {
            /* WHY: Only display the icon name without its category path visually */
            let display_name = icon_name.rsplit('/').next().unwrap_or(icon_name);
            ui.label(display_name);
        });

        let mut current_vendor = icon_settings
            .active_overrides
            .get(icon_name)
            .and_then(|o| o.vendor.clone())
            .unwrap_or_else(|| "default".to_string());

        let mut row_changed = false;

        row.col(|ui| {
            super::row::IconsRowOps::render_vendor_col(
                ui,
                icon_name,
                &mut current_vendor,
                &mut row_changed,
            );
        });

        let mut current_color = icon_settings
            .active_overrides
            .get(icon_name)
            .and_then(|o| o.color);

        row.col(|_ui| {
            super::row::IconsRowOps::render_color_col(
                _ui,
                &mut current_color,
                &mut row_changed,
                icon_settings.colorful_vendor_icons,
            );
        });

        let mut current_frame_color = icon_settings
            .active_overrides
            .get(icon_name)
            .and_then(|o| o.frame_color);

        row.col(|_ui| {
            super::row::IconsRowOps::render_frame_color_col(
                _ui,
                &mut current_frame_color,
                &mut row_changed,
                icon_settings.colorful_vendor_icons,
            );
        });

        row.col(|ui| {
            super::row::IconsRowOps::render_preview_col(
                ui,
                state,
                icon,
                &current_vendor,
                &current_color,
                &current_frame_color,
                icon_settings.colorful_vendor_icons,
            );
        });

        /* WHY: Apply row changes to settings */
        if row_changed {
            *settings_changed = true;

            let ov = icon_settings
                .active_overrides
                .entry(icon_name.to_string())
                .or_insert_with(|| katana_platform::settings::types::icon::IconOverride {
                    vendor: None,
                    color: None,
                    frame_color: None,
                });

            ov.vendor = if current_vendor == "default" {
                None
            } else {
                Some(current_vendor)
            };
            ov.color = current_color;
            ov.frame_color = current_frame_color;

            if ov.vendor.is_none() && ov.color.is_none() && ov.frame_color.is_none() {
                icon_settings.active_overrides.remove(icon_name);
            }
            icon_settings.active_preset = None;
            icon_settings.preset_state.mark_modified();
            icon_settings.preset_state.sync_user_preset_names(
                icon_settings
                    .custom_presets
                    .iter()
                    .map(|preset| &preset.name),
            );
        }
    }
}
