#![allow(unused_imports)]
#![allow(dead_code)]
use crate::Icon;
use crate::app_state::{AppAction, AppState};
use crate::shell::KatanaApp;
use crate::state::update::UpdatePhase;
use katana_core::update::ReleaseInfo;

use crate::i18n;
use egui::{Align, Layout};
use std::path::{Path, PathBuf};

pub mod about;
pub(super) mod about_widgets;
pub mod command_palette;
pub(super) mod command_palette_results;
pub mod file_ops;
pub(super) mod file_ops_rename_delete;
pub mod meta_info;
pub mod search;
pub(super) mod search_tabs;
pub mod terms;
pub mod update;
