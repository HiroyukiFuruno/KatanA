use super::types::*;
use crate::app_state::SettingsTab;
use eframe::egui;

pub(super) fn render_settings_tree(ui: &mut egui::Ui, state: &mut crate::app_state::AppState) {
    let settings_msgs = &crate::i18n::I18nOps::get().settings;

    let appearance_title = settings_msgs
        .tabs
        .iter()
        .find(|t| t.key == "group_appearance")
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "Appearance".to_string());

    crate::widgets::Accordion::new(
        "settings_grp_appearance",
        egui::RichText::new(appearance_title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
        |ui| {
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Theme,
                settings_msgs.tab_name("theme"),
            );
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Icons,
                settings_msgs.tab_name("icon"),
            );
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Font,
                settings_msgs.tab_name("font"),
            );
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Layout,
                settings_msgs.tab_name("layout"),
            );
        },
    )
    .default_open(true)
    .open(state.config.settings_tree_force_open)
    .show_vertical_line(
        state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line,
    )
    .show(ui);

    ui.add_space(SETTINGS_GROUP_SPACING);

    let system_title = settings_msgs
        .tabs
        .iter()
        .find(|t| t.key == "group_system")
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "System".to_string());

    crate::widgets::Accordion::new(
        "settings_grp_system",
        egui::RichText::new(system_title)
            .strong()
            .size(SETTINGS_HEADER_FONT_SIZE),
        |ui| {
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Workspace,
                settings_msgs.tab_name("workspace"),
            );
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Updates,
                settings_msgs.tab_name("updates"),
            );
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Behavior,
                settings_msgs.tab_name("behavior"),
            );
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Shortcuts,
                settings_msgs.tab_name("shortcuts"),
            );
            show_tab_button(
                ui,
                &mut state.config.active_settings_tab,
                SettingsTab::Linter,
                settings_msgs.tab_name("linter"),
            );
        },
    )
    .default_open(true)
    .open(state.config.settings_tree_force_open)
    .show_vertical_line(
        state
            .config
            .settings
            .settings()
            .layout
            .accordion_vertical_line,
    )
    .show(ui);
}

fn show_tab_button(
    ui: &mut egui::Ui,
    active: &mut SettingsTab,
    tab: SettingsTab,
    label: impl Into<String>,
) {
    let selected = *active == tab;
    let fill = if selected {
        ui.visuals().selection.bg_fill
    } else {
        crate::theme_bridge::TRANSPARENT
    };
    if ui
        .add(
            egui::Button::selectable(selected, label.into())
                .frame_when_inactive(true)
                .fill(fill),
        )
        .clicked()
    {
        *active = tab;
    }
}
