#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;
use katana_platform::CacheFacade;

use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::ShellLogicOps;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

pub(crate) mod manage;
mod open;
mod poll;

fn append_system_image_extensions(extensions: &mut Vec<String>) {
    for ext in katana_core::workspace::TreeEntry::image_extensions() {
        if extensions
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(ext))
        {
            continue;
        }
        extensions.push((*ext).to_string());
    }
}

pub(crate) trait WorkspaceOps {
    fn handle_open_explorer(&mut self, path: std::path::PathBuf);
    fn finish_open_explorer(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    );
    fn handle_refresh_explorer(&mut self);
    fn poll_explorer_load(&mut self, ctx: &egui::Context);
    fn handle_remove_explorer(&mut self, path: String);
    fn handle_remove_workspace_history(&mut self, path: String);
    fn save_workspace_state(&mut self);
}

impl WorkspaceOps for KatanaApp {
    fn handle_open_explorer(&mut self, path: std::path::PathBuf) {
        open::WorkspaceOpenHandlersOps::handle_open_explorer(self, path);
    }
    fn finish_open_explorer(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    ) {
        open::WorkspaceOpenHandlersOps::finish_open_explorer(self, path, ws);
    }
    fn handle_refresh_explorer(&mut self) {
        poll::handle_refresh_explorer(self);
    }
    fn poll_explorer_load(&mut self, ctx: &egui::Context) {
        poll::poll_explorer_load(self, ctx);
    }
    fn handle_remove_explorer(&mut self, path: String) {
        manage::handle_remove_explorer(self, path);
    }
    fn handle_remove_workspace_history(&mut self, path: String) {
        manage::handle_remove_workspace_history(self, path);
    }
    fn save_workspace_state(&mut self) {
        manage::save_workspace_state(self);
    }
}

#[cfg(test)]
mod tests;
