#![allow(deprecated)]
#![deny(warnings, clippy::all)]
#![allow(
    missing_docs,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod about_info;
pub mod app;
pub mod app_action;
pub use app_action::AppAction;
pub mod app_state;
pub mod font_loader;
pub mod html_renderer;
pub(crate) mod http_cache_loader;
pub mod i18n;
pub mod icon;
pub mod lint_fix_batch;
pub(crate) mod linter_bridge;
pub(crate) mod linter_config_bridge;
pub(crate) mod linter_options_bridge;
#[cfg(test)]
mod linter_options_bridge_tests;
pub mod markdown_authoring_op;
pub(crate) mod markdown_formatting_bridge;
#[cfg(test)]
mod markdown_formatting_bridge_tests;
pub use icon::*;
pub mod changelog;
pub mod diagram_controller;
pub(crate) mod editor_undo;
pub mod native_menu;
pub mod os_command;
pub mod preview_pane;
pub mod settings;
pub mod shell;
pub mod shell_logic;
pub mod shell_ui;
pub mod svg_loader;
pub mod theme_bridge;
pub mod widgets;

pub mod state;
pub mod views;
