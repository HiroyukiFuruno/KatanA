#![allow(unused_imports)]
#![allow(dead_code)]
#[path = "update_helpers.rs"]
mod update_helpers;

use crate::app::*;
use crate::shell::*;

use crate::preview_pane::PreviewPane;
use crate::shell_logic::ShellLogicOps;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

pub(crate) trait UpdateOps {
    fn start_update_check(&mut self, is_manual: bool);
    fn tick_update_check(&mut self, ctx: &egui::Context);
    fn poll_update_install(&mut self, ctx: &egui::Context);
    fn poll_update_check(&mut self, _ctx: &egui::Context);
}

impl UpdateOps for KatanaApp {
    fn start_update_check(&mut self, is_manual: bool) {
        if self.state.update.checking {
            if is_manual {
                self.show_update_dialog = true;
            }
            return;
        }
        self.state.update.checking = true;
        self.state.update.check_error = None;
        self.state.update.available = None;
        if is_manual {
            self.show_update_dialog = true;
            self.update_notified = true;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.update_rx = Some(rx);
        std::thread::spawn(move || {
            let result =
                katana_core::update::UpdateOps::check_for_updates_simple(env!("CARGO_PKG_VERSION"));
            let _ = tx.send(result);
        });
    }

    fn tick_update_check(&mut self, ctx: &egui::Context) {
        if self.state.update.checking {
            return;
        }

        let interval = self
            .state
            .config
            .settings
            .settings()
            .updates
            .interval
            .as_duration();

        let Some(interval) = interval else {
            return;
        };

        let last_checked = self
            .state
            .config
            .settings
            .settings()
            .updates
            .last_checked_timestamp_sec;
        let Some(last_checked_at) = last_checked.and_then(|value| {
            std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(value))
        }) else {
            self.start_update_check(false);
            return;
        };

        let now = std::time::SystemTime::now();
        match now.duration_since(last_checked_at) {
            Ok(elapsed) if elapsed >= interval => self.start_update_check(false),
            Ok(elapsed) => {
                let remaining = interval - elapsed;
                ctx.request_repaint_after(remaining);
            }
            Err(_) => self.start_update_check(false),
        }
    }

    fn poll_update_install(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.update_install_rx else {
            return;
        };
        while let Ok(event) = rx.try_recv() {
            match event {
                UpdateInstallEvent::Progress(prog) => {
                    self.state.update.phase =
                        Some(update_helpers::UpdateHelpers::compute_update_phase(prog));
                    ctx.request_repaint();
                }
                UpdateInstallEvent::Finished(Ok(prep)) => {
                    self.state.update.checking = false;
                    self.state.update.phase = Some(crate::app_state::UpdatePhase::ReadyToRelaunch);
                    self.pending_relaunch = Some(prep);
                    self.show_update_dialog = true;
                    self.update_install_rx = None;
                    ctx.request_repaint();
                    break;
                }
                UpdateInstallEvent::Finished(Err(err)) => {
                    self.state.update.checking = false;
                    self.state.update.phase = None;
                    self.state.update.check_error =
                        Some(katana_core::update::CheckUpdateError::Other(err));
                    self.show_update_dialog = true;
                    self.update_install_rx = None;
                    ctx.request_repaint();
                    break;
                }
            }
        }
    }

    fn poll_update_check(&mut self, _ctx: &egui::Context) {
        let Some(rx) = &self.update_rx else { return };
        match rx.try_recv() {
            Ok(Ok(Some(release_info))) => {
                self.state.update.checking = false;
                self.update_rx = None;
                update_helpers::UpdateHelpers::remember_update_check_timestamp(self);
                let is_newer = katana_core::update::UpdateOps::is_newer_version(
                    env!("CARGO_PKG_VERSION"),
                    &release_info.tag_name,
                );
                if is_newer {
                    self.state.update.available = Some(release_info);
                } else {
                    self.state.update.available = None;
                }
                if !self.update_notified {
                    if is_newer {
                        self.show_update_dialog = true;
                    }
                    self.update_notified = true;
                } else if !is_newer {
                    self.update_notified = false;
                }
            }
            Ok(Ok(None)) => {
                self.state.update.checking = false;
                self.update_rx = None;
                self.state.update.available = None;
                update_helpers::UpdateHelpers::remember_update_check_timestamp(self);
                self.update_notified = false;
            }
            Ok(Err(err)) => {
                self.state.update.checking = false;
                self.state.update.check_error = Some(err);
                self.update_rx = None;
                update_helpers::UpdateHelpers::remember_update_check_timestamp(self);
                self.state.update.available = None;
                self.update_notified = false;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(_) => {
                self.state.update.checking = false;
                self.update_rx = None;
                update_helpers::UpdateHelpers::remember_update_check_timestamp(self);
                self.state.update.available = None;
                self.update_notified = false;
            }
        }
    }
}
