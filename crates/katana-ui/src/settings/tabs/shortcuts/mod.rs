use super::ShortcutsTabOps;
use crate::app_state::AppState;
use crate::i18n::I18nOps;
use crate::state::command_inventory::types::CommandGroup;
use eframe::egui;

pub mod capture;
pub mod group;
pub mod helpers;
pub mod key_events;
pub mod modal_widgets;
pub mod row;
const SEARCH_FILTER_ID: &str = "shortcut_search_filter";

impl ShortcutsTabOps {
    pub(crate) fn render_shortcuts_tab(ui: &mut egui::Ui, state: &mut AppState) {
        let os_bindings = state
            .config
            .settings
            .settings()
            .shortcuts
            .current_os_bindings()
            .clone();

        let groups = [
            CommandGroup::App,
            CommandGroup::File,
            CommandGroup::Edit,
            CommandGroup::View,
            CommandGroup::Behavior,
            CommandGroup::Help,
        ];

        let recording_id_salt = egui::Id::new("recording_shortcut_id");
        let recording_id = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(recording_id_salt)
                .unwrap_or_default()
        });

        /* WHY: If recording is active, show the capture modal and filter out key events.
        The modal renders at Foreground order to appear above the settings panel. */
        if !recording_id.is_empty() {
            Self::show_capture_modal(ui, state, &recording_id, recording_id_salt, &os_bindings);
        }

        modal_widgets::ModalWidgets::render_conflict_warning(ui);

        /* WHY: Search bar to filter shortcuts by command name, styled like VS Code, alongside Expand/Collapse buttons */
        let mut search_query = ui.memory(|mem| {
            mem.data
                .get_temp::<String>(egui::Id::new(SEARCH_FILTER_ID))
                .unwrap_or_default()
        });
        let mut force_open: Option<bool> = None;
        crate::widgets::AlignCenter::new()
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

        ui.add_space(crate::settings::SECTION_SPACING);

        {
            let i18n = I18nOps::get();
            let search_response = crate::widgets::SearchBar::simple(&mut search_query)
                .hint_text(&i18n.settings.shortcuts.search_placeholder)
                .show_search_icon(true)
                .id_source(SEARCH_FILTER_ID)
                .show(ui);

            if search_response.changed() {
                let q = search_query.clone();
                ui.memory_mut(|mem| {
                    mem.data.insert_temp(egui::Id::new(SEARCH_FILTER_ID), q);
                });
            }
        }

        ui.add_space(crate::settings::SECTION_SPACING);

        let accordion_line = state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line;
        let search_lower = search_query.to_lowercase();

        for group in groups {
            Self::render_group(
                ui,
                state,
                group,
                &search_lower,
                &recording_id,
                recording_id_salt,
                &os_bindings,
                accordion_line,
                force_open,
            );
        }

        let i18n = I18nOps::get();
        if ui
            .button(&i18n.settings.shortcuts.restore_defaults)
            .clicked()
        {
            let s = state.config.settings.settings_mut();
            s.shortcuts.macos.clear();
            s.shortcuts.linux.clear();
            s.shortcuts.windows.clear();
            state.config.try_save_settings();
        }
    }
}
