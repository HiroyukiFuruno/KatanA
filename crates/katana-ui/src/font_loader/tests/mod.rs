/* WHY: Root module for font loader tests. Organizes tests by category into sub-modules for better maintainability and to comply with file length limits. */

use super::*;
#[cfg(target_os = "macos")]
// #[cfg(target_os = "macos")]
// use egui::FontData;
use egui::FontDefinitions;
// #[cfg(target_os = "macos")]
// use egui::FontId;
use katana_core::markdown::color_preset::DiagramColorPreset;
// use std::fs;
// #[cfg(target_os = "macos")]
// use std::sync::Arc;
// use tempfile::TempDir;

#[cfg(target_os = "macos")]
const APPLE_COLOR_EMOJI_FONT_NAME: &str = "Apple Color Emoji";

mod definitions;
mod emoji;
mod jitter;
mod normalization;
mod state;
