#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::ShellLogicOps;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

pub(crate) trait DownloadOps {
    fn start_download(&mut self, req: DownloadRequest);
    fn poll_download(&mut self, ctx: &egui::Context);
}

impl DownloadOps for KatanaApp {
    fn start_download(&mut self, req: DownloadRequest) {
        let (tx, rx) = std::sync::mpsc::channel();
        let tool_name = req.tool_name.clone();
        self.download_rx = Some(rx);
        self.active_download = Some(req.clone());
        self.state.layout.status_message = Some((
            crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().plantuml.downloading_tool,
                &[("tool", &tool_name)],
            ),
            crate::app_state::StatusType::Info,
        ));
        let url = req.url;
        let dest = req.dest;
        std::thread::spawn(move || {
            let result = katana_core::system::ProcessService::download_file(&url, &dest);
            let _ = tx.send(result);
        });
    }
    fn poll_download(&mut self, ctx: &egui::Context) {
        let done = if let Some(rx) = &self.download_rx {
            match rx.try_recv() {
                Ok(Ok(())) => {
                    let tool_name = self
                        .active_download
                        .as_ref()
                        .map(|it| it.tool_name.as_str())
                        .unwrap_or("Tool");
                    self.state.layout.status_message = Some((
                        crate::i18n::I18nOps::tf(
                            &crate::i18n::I18nOps::get().plantuml.tool_installed,
                            &[("tool", tool_name)],
                        ),
                        crate::app_state::StatusType::Success,
                    ));
                    self.pending_action = AppAction::RefreshDiagrams;
                    true
                }
                Ok(Err(e)) => {
                    self.state.layout.status_message = Some((
                        format!(
                            "{}{}",
                            crate::i18n::I18nOps::get().plantuml.download_error.clone(),
                            e
                        ),
                        crate::app_state::StatusType::Error,
                    ));
                    true
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(
                        DOWNLOAD_STATUS_CHECK_INTERVAL_MS,
                    ));
                    false
                }
                Err(_) => true,
            }
        } else {
            false
        };
        if done {
            self.download_rx = None;
            self.active_download = None;
        }
    }
}
