use super::popups::IconsPopupsOps;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use katana_core::ai::AiProviderRegistry;
use katana_core::plugin::PluginRegistry;

fn build_state() -> crate::app_state::AppState {
    crate::app_state::AppState::new(
        AiProviderRegistry::new(),
        PluginRegistry::new(),
        katana_platform::SettingsService::default(),
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    )
}

#[test]
fn save_preset_dialog_places_cancel_left_and_save_right() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(640.0, 360.0))
        .build_ui(|ui| {
            let mut state = build_state();
            ui.data_mut(|data| {
                data.insert_temp(egui::Id::new("katana_icon_saving_preset"), true);
            });
            IconsPopupsOps::render(ui, &mut state);
        });

    harness.run_steps(1);

    let i18n = crate::i18n::I18nOps::get();
    let cancel_bounds = harness
        .get_by_label(&i18n.action.cancel)
        .accesskit_node()
        .raw_bounds()
        .expect("cancel button must have bounds");
    let save_bounds = harness
        .get_by_label(&i18n.action.save)
        .accesskit_node()
        .raw_bounds()
        .expect("save button must have bounds");

    assert!(cancel_bounds.x0 < save_bounds.x0);
}
