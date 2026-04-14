use crate::settings::tabs::icons::{general, table};
use crate::widgets::AlignCenter;
use eframe::egui;

/* WHY: Parameters for rendering the sliding panels to reduce mod.rs file length. */
pub(crate) struct IconsPanelsOps;

impl IconsPanelsOps {
    pub(crate) fn render_advanced_panel(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        i18n: &crate::i18n::I18nMessages,
        is_open: &mut bool,
        icon_settings: &mut katana_platform::settings::types::icon::IconSettings,
        settings_changed: &mut bool,
    ) {
        use crate::settings::tabs::icons::{
            ADVANCED_PANEL_HEIGHT, ADVANCED_PANEL_ID, PANEL_PADDING,
        };

        egui::TopBottomPanel::bottom(ADVANCED_PANEL_ID)
            .default_height(ADVANCED_PANEL_HEIGHT)
            .show_animated_inside(ui, *is_open, |ui| {
                ui.add_space(PANEL_PADDING);

                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(PANEL_PADDING);
                    ui.heading(&i18n.settings.icons.advanced_settings);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::ImageButton::new(
                                    crate::icon::Icon::Close
                                        .ui_image(ui, crate::icon::IconSize::Small),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            *is_open = false;
                        }
                    });
                });

                ui.add_space(PANEL_PADDING);
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_source("icons_advanced_panel_scroll")
                    .show(ui, |ui| {
                        ui.add_space(PANEL_PADDING);
                        general::IconsGeneralOps::render(ui, i18n, icon_settings, settings_changed);
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
            });
    }

    pub(crate) fn render_trigger_panel(
        ui: &mut egui::Ui,
        i18n: &crate::i18n::I18nMessages,
        is_open: &mut bool,
    ) {
        use crate::settings::tabs::icons::{
            SYMMETRIC_PADDING_X, SYMMETRIC_PADDING_Y, TRIGGER_PANEL_ID,
        };

        egui::TopBottomPanel::bottom(TRIGGER_PANEL_ID)
            .frame(egui::Frame::none().inner_margin(egui::Margin::symmetric(
                SYMMETRIC_PADDING_X,
                SYMMETRIC_PADDING_Y,
            )))
            .show_inside(ui, |ui| {
                AlignCenter::new()
                    .content(|ui| {
                        if ui
                            .add(
                                egui::Button::new(&i18n.settings.icons.advanced_settings)
                                    .frame_when_inactive(true),
                            )
                            .clicked()
                        {
                            *is_open = !*is_open;
                        }
                    })
                    .show(ui);
            });
    }
}
