#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::*;
use crate::shell::*;

use crate::app::export_poll::ExportPoll;
use crate::preview_pane::{DownloadRequest, PreviewPane};
use crate::shell_logic::ShellLogicOps;
use katana_platform::FilesystemService;

use crate::app_state::*;
use std::ffi::OsStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Receiver;

pub(crate) trait ExportOps {
    fn handle_export_document(&mut self, ctx: &egui::Context, fmt: crate::app_state::ExportFormat);
    fn export_filename(&self, doc_path: &std::path::Path, ext: &str) -> String;
    fn export_as_html(&mut self, _ctx: &egui::Context, source: &str, doc_path: &std::path::Path);
    fn export_with_tool(
        &mut self,
        _ctx: &egui::Context,
        source: &str,
        ext: &str,
        doc_path: &std::path::Path,
    );
    fn poll_export(&mut self, ctx: &egui::Context);
}

impl ExportOps for KatanaApp {
    fn handle_export_document(&mut self, ctx: &egui::Context, fmt: crate::app_state::ExportFormat) {
        tracing::info!("Export document requested: {:?}", fmt);

        let Some(doc) = self.state.active_document() else {
            return;
        };
        let buffer = doc.buffer.clone();
        let doc_path = doc.path.clone();

        match fmt {
            crate::app_state::ExportFormat::Html => self.export_as_html(ctx, &buffer, &doc_path),
            crate::app_state::ExportFormat::Pdf => {
                self.export_with_tool(ctx, &buffer, "pdf", &doc_path)
            }
            crate::app_state::ExportFormat::Png => {
                self.export_with_tool(ctx, &buffer, "png", &doc_path)
            }
            crate::app_state::ExportFormat::Jpg => {
                self.export_with_tool(ctx, &buffer, "jpg", &doc_path)
            }
        }
    }
    fn export_filename(&self, doc_path: &std::path::Path, ext: &str) -> String {
        let (prefix, relative) = if let Some(ws) = &self.state.workspace.data {
            let initials: String = ws
                .root
                .components()
                .filter_map(|c| match c {
                    std::path::Component::Normal(s) => s.to_string_lossy().chars().next(),
                    _ => None,
                })
                .collect();

            let rel = doc_path.strip_prefix(&ws.root).unwrap_or(doc_path);
            (initials, rel.to_path_buf())
        } else {
            (String::new(), doc_path.to_path_buf())
        };

        let stem = relative
            .with_extension("")
            .to_string_lossy()
            .replace([std::path::MAIN_SEPARATOR, '/', ':'], "_");

        if stem.is_empty() {
            format!("export.{}", ext)
        } else if prefix.is_empty() {
            format!("{}.{}", stem, ext)
        } else {
            format!("{}_{}.{}", prefix, stem, ext)
        }
    }
    fn export_as_html(&mut self, _ctx: &egui::Context, source: &str, doc_path: &std::path::Path) {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::current().clone();
        let source = source.to_string();
        let base_dir = doc_path.parent().map(|p| p.to_path_buf());
        let filename = self.export_filename(doc_path, "html");

        let (tx, rx) = std::sync::mpsc::channel();

        let fname = filename.clone();
        std::thread::spawn(move || {
            let result = ShellLogicOps::export_named_html_to_tmp(
                &source,
                &fname,
                &preset,
                base_dir.as_deref(),
            );
            let _ = tx.send(result);
        });

        self.export_tasks.push(ExportTask {
            filename,
            rx,
            open_on_complete: true,
        });
    }
    fn export_with_tool(
        &mut self,
        _ctx: &egui::Context,
        source: &str,
        ext: &str,
        doc_path: &std::path::Path,
    ) {
        let (is_available, tool_name) = match ext {
            "pdf" => (true, "headless_chrome"),
            _ => (true, "headless_chrome"),
        };

        if !is_available {
            let msg = crate::i18n::I18nOps::tf(
                &crate::i18n::I18nOps::get().export.tool_missing,
                &[("tool", tool_name), ("format", &ext.to_uppercase())],
            );
            self.state.layout.status_message = Some((msg, crate::app_state::StatusType::Error));
            return;
        }

        let default_name = self.export_filename(doc_path, ext);

        if crate::shell_ui::ShellUiOps::is_headless() {
            self.pending_dialog_action = Some(crate::app_state::AppAction::PickExportDocument {
                doc_path: doc_path.to_path_buf(),
                ext: ext.to_string(),
                source: source.to_string(),
            });
            self.file_dialog.save_file();
            return;
        }

        let path = std::panic::catch_unwind(|| {
            rfd::FileDialog::new()
                .set_file_name(&default_name)
                .add_filter(ext, &[ext])
                .save_file()
        })
        .unwrap_or(None);

        if let Some(output_path) = path {
            crate::app::export_poll::ExportPoll::perform_tool_export(
                self,
                source,
                ext,
                output_path,
                doc_path,
            );
        } else {
            self.pending_dialog_action = Some(crate::app_state::AppAction::PickExportDocument {
                doc_path: doc_path.to_path_buf(),
                ext: ext.to_string(),
                source: source.to_string(),
            });
            self.file_dialog.save_file();
        }
    }
    fn poll_export(&mut self, ctx: &egui::Context) {
        ExportPoll::poll_export(self, ctx);
    }
}
