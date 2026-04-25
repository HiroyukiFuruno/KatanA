#![allow(unsafe_op_in_unsafe_fn)]
use crate::app_state::AppAction;

#[cfg(target_os = "macos")]
mod ffi {
    pub const TAG_OPEN_WORKSPACE: i32 = 1;
    pub const TAG_SAVE: i32 = 2;
    pub const TAG_OPEN_FILE: i32 = 21;
    pub const TAG_LANG_EN: i32 = 3;
    pub const TAG_LANG_JA: i32 = 4;
    pub const TAG_ABOUT: i32 = 5;
    pub const TAG_SETTINGS: i32 = 6;
    pub const TAG_LANG_ZH_CN: i32 = 7;
    pub const TAG_LANG_ZH_TW: i32 = 8;
    pub const TAG_LANG_KO: i32 = 9;
    pub const TAG_LANG_PT: i32 = 10;
    pub const TAG_LANG_FR: i32 = 11;
    pub const TAG_LANG_DE: i32 = 12;
    pub const TAG_LANG_ES: i32 = 13;
    pub const TAG_LANG_IT: i32 = 14;
    pub const TAG_CHECK_UPDATES: i32 = 15;
    pub const TAG_RELEASE_NOTES: i32 = 16;
    pub const TAG_COMMAND_PALETTE: i32 = 17;
    pub const TAG_DEMO: i32 = 18;
    pub const TAG_WELCOME_SCREEN: i32 = 19;
    pub const TAG_USER_GUIDE: i32 = 20;
    pub const TAG_CLOSE_WORKSPACE: i32 = 23;
    pub const TAG_EXPLORER: i32 = 24;
    pub const TAG_REFRESH_EXPLORER: i32 = 25;
    pub const TAG_CLOSE_ALL: i32 = 26;
    pub const TAG_GITHUB: i32 = 27;
    pub const TAG_REFRESH_DOCUMENT: i32 = 34;
    pub const TAG_ZOOM_IN: i32 = 35;
    pub const TAG_ZOOM_OUT: i32 = 36;
    #[allow(dead_code)]
    unsafe extern "C" {
        pub fn katana_setup_native_menu();
        pub fn katana_poll_menu_action() -> i32;
        pub fn katana_set_app_icon_png(png_data: *const u8, png_len: std::ffi::c_ulong);
        pub fn katana_set_process_name();
        pub fn native_free_menu_actions();
        pub fn katana_update_menu_strings(
            file: *const std::ffi::c_char,
            open_workspace: *const std::ffi::c_char,
            open_file: *const std::ffi::c_char,
            save: *const std::ffi::c_char,
            settings: *const std::ffi::c_char,
            preferences: *const std::ffi::c_char,
            language: *const std::ffi::c_char,
            about: *const std::ffi::c_char,
            quit: *const std::ffi::c_char,
            check_updates: *const std::ffi::c_char,
            help: *const std::ffi::c_char,
            release_notes: *const std::ffi::c_char,
            command_palette: *const std::ffi::c_char,
            view: *const std::ffi::c_char,
            demo: *const std::ffi::c_char,
            welcome_screen: *const std::ffi::c_char,
            user_guide: *const std::ffi::c_char,
            close_workspace: *const std::ffi::c_char,
            explorer: *const std::ffi::c_char,
            refresh_explorer: *const std::ffi::c_char,
            close_all: *const std::ffi::c_char,
            github: *const std::ffi::c_char,
            refresh_document: *const std::ffi::c_char,
            zoom_in: *const std::ffi::c_char,
            zoom_out: *const std::ffi::c_char,
        );
        pub fn katana_update_menu_state(
            save_enabled: bool,
            close_workspace_enabled: bool,
            refresh_explorer_enabled: bool,
            close_all_enabled: bool,
        );
    }
}

mod strings;
mod types;
pub use types::NativeMenuOps;

impl NativeMenuOps {
    #[allow(clippy::missing_safety_doc)]
    #[cfg(all(target_os = "macos", not(test)))]
    pub unsafe fn setup() {
        ffi::katana_setup_native_menu();
    }

    #[allow(clippy::missing_safety_doc)]
    #[cfg(all(target_os = "macos", not(test)))]
    pub unsafe fn set_process_name() {
        ffi::katana_set_process_name();
    }

    #[allow(clippy::missing_safety_doc)]
    #[cfg(all(target_os = "macos", not(test)))]
    pub unsafe fn set_app_icon_png(png_data: *const u8, png_len: usize) {
        ffi::katana_set_app_icon_png(png_data, png_len as std::ffi::c_ulong);
    }

    #[cfg(all(target_os = "macos", not(test)))]
    pub fn update_native_menu_strings_from_i18n() {
        strings::update_from_i18n();
    }

    #[cfg(any(not(target_os = "macos"), test))]
    pub fn update_native_menu_strings_from_i18n() {}

    #[cfg(all(target_os = "macos", not(test)))]
    pub fn update_availability(state: &crate::app_state::AppState) {
        let is_available = |id: &str| {
            crate::state::command_inventory::CommandInventory::all()
                .into_iter()
                .find(|cmd| cmd.id == id)
                .is_some_and(|cmd| (cmd.is_available)(state))
        };
        unsafe {
            ffi::katana_update_menu_state(
                is_available("file.save"),
                is_available("file.close_workspace"),
                is_available("view.refresh_explorer"),
                is_available("view.close_all"),
            );
        }
    }

    #[cfg(any(not(target_os = "macos"), test))]
    pub fn update_availability(_state: &crate::app_state::AppState) {}

    #[cfg(target_os = "macos")]
    pub(crate) fn poll(_open_folder_dialog: fn() -> Option<std::path::PathBuf>) -> AppAction {
        let action = unsafe { ffi::katana_poll_menu_action() };
        match action {
            ffi::TAG_OPEN_WORKSPACE => AppAction::PickOpenWorkspace,
            ffi::TAG_OPEN_FILE => AppAction::PickOpenFileInCurrentWorkspace,
            ffi::TAG_SAVE => AppAction::SaveDocument,
            ffi::TAG_LANG_EN => AppAction::ChangeLanguage("en".to_string()),
            ffi::TAG_LANG_JA => AppAction::ChangeLanguage("ja".to_string()),
            ffi::TAG_LANG_ZH_CN => AppAction::ChangeLanguage("zh-CN".to_string()),
            ffi::TAG_LANG_ZH_TW => AppAction::ChangeLanguage("zh-TW".to_string()),
            ffi::TAG_LANG_KO => AppAction::ChangeLanguage("ko".to_string()),
            ffi::TAG_LANG_PT => AppAction::ChangeLanguage("pt".to_string()),
            ffi::TAG_LANG_FR => AppAction::ChangeLanguage("fr".to_string()),
            ffi::TAG_LANG_DE => AppAction::ChangeLanguage("de".to_string()),
            ffi::TAG_LANG_ES => AppAction::ChangeLanguage("es".to_string()),
            ffi::TAG_LANG_IT => AppAction::ChangeLanguage("it".to_string()),
            ffi::TAG_ABOUT => AppAction::ToggleAbout,
            ffi::TAG_CHECK_UPDATES => AppAction::CheckForUpdates,
            ffi::TAG_RELEASE_NOTES => AppAction::ShowReleaseNotes,
            ffi::TAG_SETTINGS => AppAction::ToggleSettings,
            ffi::TAG_COMMAND_PALETTE => AppAction::ToggleCommandPalette,
            ffi::TAG_DEMO => AppAction::OpenHelpDemo,
            ffi::TAG_WELCOME_SCREEN => AppAction::OpenWelcomeScreen,
            ffi::TAG_USER_GUIDE => AppAction::OpenUserGuide,
            ffi::TAG_CLOSE_WORKSPACE => AppAction::CloseWorkspace,
            ffi::TAG_EXPLORER => AppAction::ToggleExplorer,
            ffi::TAG_REFRESH_EXPLORER => AppAction::RefreshExplorer,
            ffi::TAG_CLOSE_ALL => AppAction::CloseAllDocuments,
            ffi::TAG_GITHUB => AppAction::OpenGitHub,
            ffi::TAG_REFRESH_DOCUMENT => AppAction::RefreshDocument { is_manual: true },
            ffi::TAG_ZOOM_IN => AppAction::ZoomIn,
            ffi::TAG_ZOOM_OUT => AppAction::ZoomOut,
            _ => AppAction::None,
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn poll(_open_folder_dialog: fn() -> Option<std::path::PathBuf>) -> AppAction {
        AppAction::None
    }
}
