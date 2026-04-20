#![allow(clippy::module_inception)]
#![allow(deprecated)]

#[path = "integration/app/state.rs"]
mod app_state;
#[path = "integration/search/command_palette.rs"]
mod command_palette;
#[path = "integration/settings/font_bridge.rs"]
mod font_bridge;
#[path = "integration/settings/font_realtime.rs"]
mod font_realtime;
#[path = "integration/foreground_surface_isolation.rs"]
mod foreground_surface_isolation;
#[path = "integration/settings/i18n.rs"]
mod i18n;

pub static SERIAL_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn get_serial_test_mutex() -> &'static std::sync::Mutex<()> {
    &SERIAL_TEST_MUTEX
}

pub mod integration {
    pub use super::get_serial_test_mutex;
    #[path = "harness_utils.rs"]
    pub mod harness_utils;
}

#[path = "integration/overlap_checker.rs"]
mod overlap_checker;
#[path = "integration/preview_pane/mod.rs"]
mod preview_pane;
#[path = "integration/search/bar.rs"]
mod search_bar_test;
#[path = "integration/settings/ui.rs"]
mod settings_window;
#[path = "integration/shell_logic.rs"]
mod shell_logic;
#[path = "integration/settings/theme.rs"]
mod theme;
#[path = "integration/settings/theme_bridge.rs"]
mod theme_bridge;
#[path = "integration/tree_layout.rs"]
mod tree_layout;
#[path = "integration/preview_pane/styling.rs"]
mod underline_rendering;
