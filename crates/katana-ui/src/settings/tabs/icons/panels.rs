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
        current_pack: &str,
        icon_settings: &mut katana_platform::settings::types::icon::IconSettings,
        settings_changed: &mut bool,
    ) {
        if !*is_open {
            return;
        }

        let mut force_open: Option<bool> = None;

        /* WHY: Header row — title left, close button right. */
        AlignCenter::new()
            .left(|ui| ui.heading(&i18n.common.advanced_settings))
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

        /* WHY: Action buttons (Expand / Collapse) and Scrollable content fills 100% of remaining height — general settings + per-icon table + action buttons. */
        AlignCenter::new()
            .left(|ui| {
                let i18n_common = &crate::i18n::I18nOps::get().common;
                if ui.button(&i18n_common.expand_all).clicked() {
                    force_open = Some(true);
                }
                if ui.button(&i18n_common.collapse_all).clicked() {
                    force_open = Some(false);
                }
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            })
            .show(ui);

        ui.add_space(PADDING);

        /* WHY: Search bar for icon filtering — consistent with Linter and Shortcuts tabs */
        let mut search_query = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new("icon_advanced_search_filter"))
                .unwrap_or_default()
        });
        let search_response = crate::widgets::SearchBar::simple(&mut search_query)
            .hint_text(&i18n.settings.icons.search_placeholder)
            .show_search_icon(true)
            .id_source("icon_advanced_search_filter")
            .show(ui);
        if search_response.changed() {
            let q = search_query.clone();
            ui.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new("icon_advanced_search_filter"), q);
            });
        }

        egui::ScrollArea::vertical()
            .id_salt("icon_advanced_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                general::IconsGeneralOps::render(ui, i18n, icon_settings, settings_changed);

                ui.add_space(PADDING);

                table::IconsTableOps::render(
                    ui,
                    state,
                    i18n,
                    icon_settings,
                    settings_changed,
                    force_open,
                    &search_query,
                );

                ui.add_space(PADDING);
                ui.separator();
                ui.add_space(PADDING);

                /* WHY: Action buttons (save preset / revert / delete) inside scroll area. */
                AlignCenter::new()
                    .right(|ui| {
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
                            icon_settings.preset_state.select_built_in(
                                current_pack,
                                super::preset_controls::IconPresetControlsOps::pack_label(
                                    current_pack,
                                ),
                            );
                            icon_settings.preset_state.sync_user_preset_names(
                                icon_settings
                                    .custom_presets
                                    .iter()
                                    .map(|preset| &preset.name),
                            );
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
                                icon_settings.preset_state.select_built_in(
                                    current_pack,
                                    super::preset_controls::IconPresetControlsOps::pack_label(
                                        current_pack,
                                    ),
                                );
                                icon_settings.preset_state.sync_user_preset_names(
                                    icon_settings
                                        .custom_presets
                                        .iter()
                                        .map(|preset| &preset.name),
                                );
                                *settings_changed = true;
                            }
                        }
                        ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
                    })
                    .show(ui);
            });
    }
}
