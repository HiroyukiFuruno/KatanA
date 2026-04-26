use eframe::egui;
use katana_platform::settings::IconSettings;
use katana_platform::settings::PresetReference;
use katana_platform::settings::PresetSource;

const ICON_PACK_COUNT: usize = 6;

pub(crate) struct IconPresetControlsOps;

impl IconPresetControlsOps {
    pub(crate) fn render(
        ui: &mut egui::Ui,
        current_pack: &mut String,
        icon_settings: &mut IconSettings,
        settings_changed: &mut bool,
        is_advanced_open: &mut bool,
    ) {
        let built_in_presets = Self::built_in_presets();
        let user_presets = Self::user_presets(icon_settings);
        let mut preset_state = icon_settings.preset_state.clone();
        Self::ensure_current_state(&mut preset_state, current_pack);

        let i18n = crate::i18n::I18nOps::get();
        let labels = crate::settings::tabs::preset_widget::PresetWidgetLabels {
            title: &i18n.settings.icons.preset_label,
            save: &i18n.settings.icons.save_preset,
            revert: &i18n.settings.icons.revert_default,
            advanced: Some(&i18n.common.advanced_settings),
        };
        let response = crate::settings::tabs::preset_widget::PresetWidgetOps::render(
            ui,
            "icon_preset_widget",
            &preset_state,
            &built_in_presets,
            &user_presets,
            labels,
        );
        Self::handle_response(
            ui,
            current_pack,
            icon_settings,
            settings_changed,
            is_advanced_open,
            response,
        );
    }

    fn handle_response(
        ui: &mut egui::Ui,
        current_pack: &mut String,
        icon_settings: &mut IconSettings,
        settings_changed: &mut bool,
        is_advanced_open: &mut bool,
        response: crate::settings::tabs::preset_widget::PresetWidgetResponse,
    ) {
        if let Some(reference) = response.selected {
            *settings_changed |= Self::apply_reference(current_pack, icon_settings, &reference);
        }
        if response.revert_clicked {
            icon_settings.active_overrides.clear();
            icon_settings.active_preset = None;
            Self::select_built_in_state(icon_settings, current_pack);
            *settings_changed = true;
        }
        if response.save_clicked {
            ui.data_mut(|data| data.insert_temp(egui::Id::new("katana_icon_saving_preset"), true));
        }
        if response.advanced_clicked {
            *is_advanced_open = true;
        }
    }

    fn apply_reference(
        current_pack: &mut String,
        icon_settings: &mut IconSettings,
        reference: &PresetReference,
    ) -> bool {
        match reference.source {
            PresetSource::BuiltIn => {
                *current_pack = reference.id.clone();
                icon_settings.active_preset = None;
                icon_settings.active_overrides.clear();
                icon_settings
                    .preset_state
                    .select_built_in(reference.id.clone(), reference.label.clone());
                true
            }
            PresetSource::User => Self::apply_user_preset(icon_settings, &reference.id),
            PresetSource::Custom | PresetSource::Unknown => false,
        }
    }

    fn apply_user_preset(icon_settings: &mut IconSettings, name: &str) -> bool {
        let Some(preset) = icon_settings
            .custom_presets
            .iter()
            .find(|preset| preset.name == name)
            .cloned()
        else {
            return false;
        };
        icon_settings.active_preset = Some(preset.name.clone());
        icon_settings.active_overrides = preset.overrides;
        icon_settings.preset_state.select_user(name);
        Self::sync_user_presets(icon_settings);
        true
    }

    fn ensure_current_state(
        preset_state: &mut katana_platform::settings::PresetState,
        current_pack: &str,
    ) {
        if preset_state.current.is_none() {
            preset_state.select_built_in(current_pack, Self::pack_label(current_pack));
        }
    }

    fn select_built_in_state(icon_settings: &mut IconSettings, current_pack: &str) {
        icon_settings
            .preset_state
            .select_built_in(current_pack, Self::pack_label(current_pack));
        Self::sync_user_presets(icon_settings);
    }

    fn sync_user_presets(icon_settings: &mut IconSettings) {
        icon_settings.preset_state.sync_user_preset_names(
            icon_settings
                .custom_presets
                .iter()
                .map(|preset| &preset.name),
        );
    }

    fn built_in_presets() -> Vec<PresetReference> {
        Self::available_packs()
            .iter()
            .map(|(id, label)| PresetReference::built_in(*id, *label))
            .collect()
    }

    fn user_presets(icon_settings: &IconSettings) -> Vec<PresetReference> {
        icon_settings
            .custom_presets
            .iter()
            .map(|preset| PresetReference::user(&preset.name))
            .collect()
    }

    pub(crate) fn pack_label(id: &str) -> String {
        Self::available_packs()
            .iter()
            .find(|(pack_id, _)| *pack_id == id)
            .map(|(_, label)| (*label).to_string())
            .unwrap_or_else(|| id.to_string())
    }

    fn available_packs() -> [(&'static str, &'static str); ICON_PACK_COUNT] {
        [
            ("katana", "KatanA (Default)"),
            ("material-symbols", "Material Symbols"),
            ("lucide", "Lucide"),
            ("tabler-icons", "Tabler Icons"),
            ("heroicons", "Heroicons"),
            ("feather", "Feather"),
        ]
    }
}
