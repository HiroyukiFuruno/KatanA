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
        force_open: Option<bool>,
        search_query: &str,
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
            if name.starts_with("../system/") {
                continue;
            }
            /* WHY: Filter icons by search query for consistency with other settings tabs */
            let search_lower = search_query.to_lowercase();
            if !search_lower.is_empty() && !name.to_lowercase().contains(&search_lower) {
                continue;
            }
            let vendor = if let Some(slash_idx) = name.find('/') {
                name[..slash_idx].to_string()
            } else {
                "katana".to_string()
            };
            let list = grouped_icons.entry(vendor).or_default();
            if !list.iter().any(|i| i.name() == name) {
                list.push(icon);
            }
        }

        ui.indent("icons_table_header_indent", |ui| {
            egui_extras::TableBuilder::new(ui)
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
                .body(|_body| {});
        });

        const HEADER_BOTTOM_MARGIN: f32 = 8.0;
        const BODY_MAX_HEIGHT: f32 = 400.0;

        ui.add_space(HEADER_BOTTOM_MARGIN); /* WHY: Spacing between fixed header and the scrollable accordion contents */

        egui::ScrollArea::vertical()
            .id_salt("icons_overrides_tree_scroll")
            .max_height(BODY_MAX_HEIGHT)
            .show(ui, |ui| {
                for (category, icons) in grouped_icons {
                    crate::widgets::Accordion::new(
                        format!("icons_advanced_{}", category),
                        egui::RichText::new(&category).strong(),
                        |ui| {
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
                                .body(|body| {
                                    body.rows(ROW_HEIGHT, icons.len(), |mut row| {
                                        let index = row.index();
                                        let icon = icons[index];
                                        super::table_row::IconsTableRowOps::render_row(
                                            &mut row,
                                            state,
                                            icon,
                                            icon_settings,
                                            settings_changed,
                                        );
                                    });
                                });
                        },
                    )
                    .default_open(true)
                    .force_open(force_open)
                    .show(ui);
                }
            });
    }
}
