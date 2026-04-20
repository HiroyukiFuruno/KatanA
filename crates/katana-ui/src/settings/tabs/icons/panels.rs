use crate::settings::tabs::icons::{general, table};
use crate::widgets::AlignCenter;
use eframe::egui;

const PADDING: f32 = 8.0;

/* WHY: Renders the advanced settings panel as a full-height view. */
pub(crate) struct IconsPanelsOps;

impl IconsPanelsOps {
    /// Renders the advanced settings as a full-height view (100% of settings tab).
    /// When closed, this is a no-op — the trigger button lives in mod.rs.
    pub(crate) fn render_panels(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        i18n: &crate::i18n::I18nMessages,
        is_open: &mut bool,
        icon_settings: &mut katana_platform::settings::types::icon::IconSettings,
        settings_changed: &mut bool,
    ) {
        if !*is_open {
            return;
        }

        /* WHY: Header row — title left, close button right. */
        AlignCenter::new()
            .left(|ui| ui.heading(&i18n.settings.icons.advanced_settings))
            .right(|ui| {
                if ui
                    .button(&i18n.common.close)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    *is_open = false;
                }
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);

        ui.separator();

        /* WHY: Scrollable content fills 100% of remaining height — general settings + per-icon table + action buttons. */
        egui::ScrollArea::vertical()
            .id_salt("icon_advanced_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                general::IconsGeneralOps::render(ui, i18n, icon_settings, settings_changed);

                ui.add_space(PADDING);

                table::IconsTableOps::render(ui, state, i18n, icon_settings, settings_changed);

                ui.add_space(PADDING);
                ui.separator();
                ui.add_space(PADDING);

                /* WHY: Action buttons (save preset / revert / delete) inside scroll area. */
                AlignCenter::new()
                    .content(|ui| {
                        if ui.button(&i18n.settings.icons.save_preset).clicked() {
                            ui.data_mut(|d| {
                                d.insert_temp::<bool>(
                                    egui::Id::new("katana_icon_saving_preset"),
                                    true,
                                )
                            });
                        }

                        if !icon_settings.active_overrides.is_empty()
                            && ui.button(&i18n.settings.icons.revert_default).clicked()
                        {
                            icon_settings.active_overrides.clear();
                            icon_settings.active_preset = None;
                            *settings_changed = true;
                        }

                        if let Some(active_preset) = &icon_settings.active_preset {
                            let icon_bg = if ui.visuals().dark_mode {
                                crate::theme_bridge::TRANSPARENT
                            } else {
                                crate::theme_bridge::ThemeBridgeOps::from_gray(
                                    crate::shell_ui::LIGHT_MODE_ICON_BG,
                                )
                            };
                            let clicked = ui
                                .add(
                                    egui::Button::image(
                                        crate::Icon::Remove
                                            .ui_image(ui, crate::icon::IconSize::Medium),
                                    )
                                    .fill(icon_bg),
                                )
                                .on_hover_text(
                                    &crate::i18n::I18nOps::get().settings.theme.delete_custom,
                                )
                                .clicked();
                            if clicked {
                                let old_name = active_preset.clone();
                                icon_settings.custom_presets.retain(|p| p.name != old_name);
                                icon_settings.active_preset = None;
                                icon_settings.active_overrides.clear();
                                *settings_changed = true;
                            }
                        }
                    })
                    .show(ui);
            });
    }
}
