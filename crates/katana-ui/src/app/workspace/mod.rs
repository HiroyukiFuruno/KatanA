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

mod manage;
mod open;
mod poll;

pub(crate) trait WorkspaceOps {
    fn handle_open_workspace(&mut self, path: std::path::PathBuf);
    fn finish_open_workspace(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    );
    fn handle_refresh_workspace(&mut self);
    fn poll_workspace_load(&mut self, ctx: &egui::Context);
    fn handle_remove_workspace(&mut self, path: String);
    fn save_workspace_state(&mut self);
}

impl WorkspaceOps for KatanaApp {
    fn handle_open_workspace(&mut self, path: std::path::PathBuf) {
        open::handle_open_workspace(self, path);
    }
    fn finish_open_workspace(
        &mut self,
        path: std::path::PathBuf,
        ws: katana_core::workspace::Workspace,
    ) {
        open::finish_open_workspace(self, path, ws);
    }
    fn handle_refresh_workspace(&mut self) {
        poll::handle_refresh_workspace(self);
    }
    fn poll_workspace_load(&mut self, ctx: &egui::Context) {
        poll::poll_workspace_load(self, ctx);
    }
    fn handle_remove_workspace(&mut self, path: String) {
        manage::handle_remove_workspace(self, path);
    }
    fn save_workspace_state(&mut self) {
        manage::save_workspace_state(self);
    }
}

#[cfg(test)]
mod tests;
