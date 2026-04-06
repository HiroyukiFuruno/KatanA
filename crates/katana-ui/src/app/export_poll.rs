use crate::app_state::*;
use crate::shell::*;

pub(super) trait ExportPoll {
    fn perform_tool_export(
        &mut self,
        source: &str,
        ext: &str,
        output_path: std::path::PathBuf,
        doc_path: &std::path::Path,
    );
    fn poll_export(&mut self, ctx: &egui::Context);
}

impl ExportPoll for KatanaApp {
    fn perform_tool_export(
        &mut self,
        source: &str,
        ext: &str,
        output_path: std::path::PathBuf,
        doc_path: &std::path::Path,
    ) {
        let preset = katana_core::markdown::color_preset::DiagramColorPreset::current().clone();
        let source = source.to_string();
        let ext = ext.to_string();
        let base_dir = doc_path.parent().map(|p| p.to_path_buf());
        let filename = output_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "export".to_string());
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let renderer = katana_core::markdown::KatanaRenderer;
            let html = match katana_core::markdown::HtmlExporter::export(
                &source,
                &renderer,
                &preset,
                base_dir.as_deref(),
            ) {
                Ok(h) => h,
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };
            let result = match ext.as_str() {
                "pdf" => katana_core::markdown::PdfExporter::export(&html, &output_path),
                _ => katana_core::markdown::ImageExporter::export(&html, &output_path),
            };
            let _ = tx.send(
                result
                    .map(|()| output_path.clone())
                    .map_err(|e| e.to_string()),
            );
        });
        self.export_tasks.push(ExportTask {
            filename,
            rx,
            open_on_complete: false,
        });
    }

    fn poll_export(&mut self, ctx: &egui::Context) {
        const EXPORT_POLL_INTERVAL_MS: u64 = 50;
        let mut has_pending = false;
        let mut completed: Vec<(usize, Result<std::path::PathBuf, String>)> = Vec::new();
        for (i, task) in self.export_tasks.iter().enumerate() {
            match task.rx.try_recv() {
                Ok(result) => {
                    completed.push((i, result));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    has_pending = true;
                }
                Err(_) => {
                    completed.push((i, Err("Export thread disconnected".to_string())));
                }
            }
        }
        for (i, result) in completed.into_iter().rev() {
            let task = self.export_tasks.remove(i);
            match result {
                Ok(output_path) => {
                    let ext = output_path
                        .extension()
                        .map(|e| e.to_string_lossy().to_uppercase())
                        .unwrap_or_default();
                    let msg = crate::i18n::I18nOps::tf(
                        &crate::i18n::I18nOps::get().export.success,
                        &[
                            ("format", &ext),
                            ("path", &output_path.display().to_string()),
                        ],
                    );
                    self.state.layout.status_message = Some((msg, StatusType::Success));
                    if task.open_on_complete
                        && let Err(e) = open::that(&output_path)
                    {
                        tracing::warn!("Failed to open {}: {e}", output_path.display());
                    }
                    tracing::info!(
                        "Export complete: {} → {}",
                        task.filename,
                        output_path.display()
                    );
                }
                Err(error) => {
                    let msg = crate::i18n::I18nOps::tf(
                        &crate::i18n::I18nOps::get().export.failed,
                        &[("format", &task.filename), ("error", &error)],
                    );
                    self.state.layout.status_message = Some((msg, StatusType::Error));
                }
            }
        }
        if has_pending {
            ctx.request_repaint_after(std::time::Duration::from_millis(EXPORT_POLL_INTERVAL_MS));
        }
    }
}
