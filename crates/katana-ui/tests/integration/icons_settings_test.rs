use egui_kittest::Harness;
use katana_ui::settings::tabs::icons::IconsTabOps;
use egui_kittest::kittest::Queryable;

#[test]
fn red_advanced_settings_panel_content_is_visible() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 600.0))
        .build_ui(|ui| {
            let mut state = crate::integration::common::create_mock_app_state();
            
            // Force advanced settings to be open in UI data
            ui.data_mut(|d| d.insert_temp(egui::Id::new("icons_advanced_is_open"), true));
            
            IconsTabOps::render_icons_tab(ui, &mut state);
        });

    harness.run();

    // The i18n key for table header icon might be "Icon" or translated.
    // In our tests, we use the Japanese locale by default or English.
    // Let's check for "高度な設定" which we saw in the image.
    harness.get_by_label("高度な設定"); // Should exist in panels.rs

    // Now check for something INSIDE the table from table.rs
    // "アイコン" (Icon) or "ベンダー" (Vendor) should be in the header.
    // If this fails, it means the content is not rendered.
    harness.get_by_label("アイコン"); 
}

mod integration {
    pub mod common {
        pub fn create_mock_app_state() -> katana_ui::app_state::AppState {
            // This is a simplified mock. Ideally we use existing test helpers.
            let mut state = katana_ui::app_state::AppState::default();
            // Ensure icons are loaded if needed
            state
        }
    }
}
