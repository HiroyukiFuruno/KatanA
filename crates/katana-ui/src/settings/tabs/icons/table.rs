/* WHY: Operations for rendering the advanced override table. */
pub(crate) struct IconsTableOps;

impl IconsTableOps {
    /* WHY: Renders a sticky-header table for fine-grained icon overrides. */
    pub(crate) fn render(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        i18n: &crate::i18n::I18nMessages,
        icon_settings: &mut katana_platform::settings::types::icon::IconSettings,
        settings_changed: &mut bool,
    ) {
        const COL_ICON_WIDTH: f32 = 180.0;
        const COL_VENDOR_WIDTH: f32 = 140.0;
        const COL_COLOR_WIDTH: f32 = 120.0;
        const COL_BORDER_WIDTH: f32 = 120.0;
        const COL_PREVIEW_MIN: f32 = 80.0;
        const HEADER_HEIGHT: f32 = 20.0;
        const ROW_HEIGHT: f32 = 30.0;

        let mut grouped_icons: std::collections::BTreeMap<String, Vec<&crate::icon::Icon>> =
            std::collections::BTreeMap::new();

        for icon in crate::icon::ALL_ICONS {
            let name = icon.name();
            let vendor = if let Some(slash_idx) = name.find('/') {
                name[..slash_idx].to_string()
            } else {
                "katana".to_string()
            };
            grouped_icons.entry(vendor).or_default().push(icon);
        }

        for (category, icons) in grouped_icons {
            egui::CollapsingHeader::new(&category)
                .default_open(true)
                .show(ui, |ui| {
                    egui_extras::TableBuilder::new(ui)
                        .striped(true)
                        .resizable(false)
                        .vscroll(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(egui_extras::Column::exact(COL_ICON_WIDTH))
                        .column(egui_extras::Column::exact(COL_VENDOR_WIDTH))
                        .column(egui_extras::Column::exact(COL_COLOR_WIDTH))
                        .column(egui_extras::Column::exact(COL_BORDER_WIDTH))
                        .column(egui_extras::Column::remainder().at_least(COL_PREVIEW_MIN))
                        .header(HEADER_HEIGHT, |mut header| {
                            header.col(|ui| {
                                ui.strong(&i18n.settings.icons.table_header_icon);
                            });
                            header.col(|ui| {
                                ui.strong(&i18n.settings.icons.table_header_vendor);
                            });
                            header.col(|ui| {
                                ui.strong(&i18n.settings.icons.table_header_color);
                            });
                            header.col(|ui| {
                                ui.strong(&i18n.settings.icons.table_header_border);
                            });
                            header.col(|ui| {
                                ui.strong(&i18n.settings.icons.table_header_preview);
                            });
                        })
                        .body(|body| {
                            body.rows(ROW_HEIGHT, icons.len(), |mut row| {
                                let icon = icons[row.index()];
                                let icon_name = icon.name();

                                row.col(|ui| {
                                    ui.label(icon_name);
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
                                        .or_insert_with(|| {
                                            katana_platform::settings::types::icon::IconOverride {
                                                vendor: None,
                                                color: None,
                                                frame_color: None,
                                            }
                                        });

                                    ov.vendor = if current_vendor == "default" {
                                        None
                                    } else {
                                        Some(current_vendor)
                                    };
                                    ov.color = current_color;
                                    ov.frame_color = current_frame_color;

                                    if ov.vendor.is_none()
                                        && ov.color.is_none()
                                        && ov.frame_color.is_none()
                                    {
                                        icon_settings.active_overrides.remove(icon_name);
                                    }
                                    icon_settings.active_preset = None;
                                }
                            });
                        });
                });
        }
    }
}
