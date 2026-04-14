#![cfg(test)]
#![allow(clippy::module_inception)]
#![allow(deprecated)]

pub mod integration {
    pub mod app_state;
    pub mod command_palette;
    pub mod font_bridge;
    pub mod font_realtime;
    pub mod i18n;
    pub mod integration;
    pub mod overlap_checker;
    pub mod preview_pane;
    pub mod preview_search;
    pub mod sample_fixture_tests;
    pub mod settings_window;
    pub mod shell_logic;
    pub mod theme;
    pub mod theme_bridge;
    pub mod theme_rendering_sync;
    pub mod tree_layout;
    pub mod underline_rendering;
}
