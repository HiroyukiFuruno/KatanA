use crate::settings::tabs::icons::{general, table};
use crate::widgets::AlignCenter;
use eframe::egui;

const SCROLL_AREA_HEIGHT_RATIO: f32 = 0.8;

/* WHY: Parameters for rendering the sliding panels to reduce mod.rs file length. */
pub(crate) struct IconsPanelsOps;

impl IconsPanelsOps {
    pub(crate) fn render_panels(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        i18n: &crate::i18n::I18nMessages,
        is_open: &mut bool,
        icon_settings: &mut katana_platform::settings::types::icon::IconSettings,
        settings_changed: &mut bool,
    ) {
        use crate::settings::tabs::icons::{
            ADVANCED_PANEL_ID, PANEL_PADDING, SYMMETRIC_PADDING_X, SYMMETRIC_PADDING_Y,
        };

        egui::TopBottomPanel::bottom(ADVANCED_PANEL_ID)
            .resizable(*is_open)
            .min_height(if *is_open { 100.0 } else { 0.0 })
            .max_height(ui.available_height() * SCROLL_AREA_HEIGHT_RATIO)
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(
                SYMMETRIC_PADDING_X,
                SYMMETRIC_PADDING_Y,
            )))
            .show_inside(ui, |ui| {
                if *is_open {
                    ui.add_space(PANEL_PADDING);

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(PANEL_PADDING);
                        ui.heading(&i18n.settings.icons.advanced_settings);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(&i18n.common.close)
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                *is_open = false;
                            }
                        });
                    });

                    ui.add_space(PANEL_PADDING);
                    ui.separator();
                    ui.add_space(PANEL_PADDING);

                    egui::ScrollArea::vertical()
                        .id_source("icon_advanced_scroll")
                        .show(ui, |ui| {
                            general::IconsGeneralOps::render(
                                ui,
                                i18n,
                                icon_settings,
                                settings_changed,
                            );
                            ui.add_space(PANEL_PADDING);
                            table::IconsTableOps::render(
                                ui,
                                state,
                                i18n,
                                icon_settings,
                                settings_changed,
                            );
                            ui.add_space(PANEL_PADDING);
                        });
                } else {
                    AlignCenter::new()
                        .content(|ui| {
                            if ui
                                .button(&i18n.settings.icons.advanced_settings)
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                *is_open = true;
                            }
                        })
                        .show(ui);
                }
            });
    }
}
