#![allow(clippy::module_inception)]
#![allow(deprecated)]

#[path = "integration/app_state.rs"]
mod app_state;
#[path = "integration/command_palette.rs"]
mod command_palette;
#[path = "integration/font_bridge.rs"]
mod font_bridge;
#[path = "integration/font_realtime.rs"]
mod font_realtime;
#[path = "integration/i18n.rs"]
mod i18n;
#[path = "integration/integration.rs"]
mod integration;
#[path = "integration/overlap_checker.rs"]
mod overlap_checker;
#[path = "integration/preview_pane.rs"]
mod preview_pane;
#[path = "integration/settings_window.rs"]
mod settings_window;
#[path = "integration/shell_logic.rs"]
mod shell_logic;
#[path = "integration/theme.rs"]
mod theme;
#[path = "integration/theme_bridge.rs"]
mod theme_bridge;
#[path = "integration/theme_rendering_sync.rs"]
mod theme_rendering_sync;
#[path = "integration/tree_layout.rs"]
mod tree_layout;
#[path = "integration/underline_rendering.rs"]
mod underline_rendering;
