/* WHY: Root module for font loader tests. Organizes tests by category into sub-modules for better maintainability and to comply with file length limits. */

use super::*;
use egui::FontDefinitions;
#[cfg(target_os = "macos")]
use katana_core::markdown::color_preset::DiagramColorPreset;
// use std::fs;
// #[cfg(target_os = "macos")]
// use std::sync::Arc;
// use tempfile::TempDir;

#[cfg(target_os = "macos")]
const APPLE_COLOR_EMOJI_FONT_NAME: &str = "Apple Color Emoji";

mod definitions;
#[cfg(target_os = "macos")]
mod emoji;
#[cfg(target_os = "macos")]
mod jitter;
mod normalization;
mod state;
