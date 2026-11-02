use crate::app::DocumentOps;
use crate::shell::KatanaApp;
use eframe::egui;
use egui_kittest::{Harness, kittest::Queryable};

const SETTLE_FRAMES: usize = 3;
const CLOSED_VERIFICATION_FRAMES: usize = 3;
const VIEWPORT_WIDTH: f32 = 1024.0;
const VIEWPORT_HEIGHT: f32 = 768.0;

pub(super) fn steps(harness: &mut Harness<'_, KatanaApp>) {
    for _ in 0..SETTLE_FRAMES {
        harness.step();
    }
}

pub(super) fn open_group_menu(harness: &mut Harness<'_, KatanaApp>, file: &str) {
    let node = harness
        .get_all_by_label(file)
        .min_by(|a, b| a.rect().center().y.total_cmp(&b.rect().center().y))
        .expect("tab label");
    node.click_secondary();
    steps(harness);
    let label = crate::i18n::I18nOps::get().tab.create_new_group.clone();
    harness.get_by_label(&label).click();
    steps(harness);
    steps(harness);
}

pub(super) fn popup_id(harness: &Harness<'_, KatanaApp>) -> egui::Id {
    let group = harness
        .state()
        .state
        .document
        .tab_groups
        .last()
        .expect("created group");
    egui::Id::new("group_popup").with(&group.id)
}

pub(super) fn prepare(directory: &tempfile::TempDir) -> (Harness<'static, KatanaApp>, String) {
    let path = directory.path().join("group-native-fixture.md");
    std::fs::write(&path, "# fixture\n").expect("write isolated fixture");
    let file = path.file_name().unwrap().to_string_lossy().into_owned();
    let settings_path = directory.path().join("settings.json");
    let mut harness = make_harness(settings_path);
    harness.state_mut().handle_select_document(path, true);
    steps(&mut harness);
    (harness, file)
}

pub(super) fn assert_closed(harness: &mut Harness<'_, KatanaApp>, popup: egui::Id) {
    for _ in 0..CLOSED_VERIFICATION_FRAMES {
        harness.step();
        assert!(
            !egui::Popup::is_id_open(&harness.ctx, popup),
            "popup reopened or remained open"
        );
        assert!(
            harness
                .query_by_label(&crate::i18n::I18nOps::get().action.cancel)
                .is_none(),
            "popup controls remain visible"
        );
        assert!(
            harness
                .query_by_label(&crate::i18n::I18nOps::get().action.confirm)
                .is_none(),
            "popup confirmation remains visible"
        );
        assert!(
            harness
                .query_by_label(&crate::i18n::I18nOps::get().tab.group_name_placeholder)
                .is_none(),
            "popup name input remains visible"
        );
    }
    assert!(harness.state().state.layout.inline_rename_group.is_none());
}

pub(super) fn group_name(harness: &Harness<'_, KatanaApp>) -> String {
    harness
        .state()
        .state
        .document
        .tab_groups
        .last()
        .unwrap()
        .name
        .clone()
}

fn make_harness(settings_path: std::path::PathBuf) -> Harness<'static, KatanaApp> {
    Harness::builder()
        .with_size(egui::vec2(VIEWPORT_WIDTH, VIEWPORT_HEIGHT))
        .build_eframe(move |_cc| make_app(settings_path))
}

fn make_app(settings_path: std::path::PathBuf) -> KatanaApp {
    let settings = katana_platform::SettingsService::new(Box::new(
        katana_platform::JsonFileRepository::new(settings_path),
    ));
    let mut state = crate::app_state::AppState::new(
        katana_core::ai::AiProviderRegistry::new(),
        katana_core::plugin::PluginRegistry::new(),
        settings,
        std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
    );
    state.config.settings.settings_mut().terms_accepted_version =
        Some(crate::about_info::APP_VERSION.to_owned());
    state
        .config
        .settings
        .settings_mut()
        .updates
        .previous_app_version = Some(crate::about_info::APP_VERSION.to_owned());
    state.global_workspace = katana_platform::workspace::GlobalWorkspaceService::new(Box::new(
        katana_platform::workspace::InMemoryWorkspaceRepository::default(),
    ));
    let mut app = KatanaApp::new(state);
    app.skip_splash();
    app.disable_update_check_for_test();
    app.disable_changelog_popup_for_test();
    app
}
