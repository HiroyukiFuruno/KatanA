use eframe::egui;
use katana_core::canvas::{CanvasDiagnostic, CanvasDocument};

use super::{CanvasLoadError, CanvasSurfaceLabels};

const DIAGNOSTIC_MAX_HEIGHT: f32 = 120.0;

pub(super) struct CanvasNotices;

impl CanvasNotices {
    pub(super) fn diagnostics_ui(
        ui: &mut egui::Ui,
        document: &CanvasDocument,
        labels: &CanvasSurfaceLabels<'_>,
    ) {
        if document.diagnostics.is_empty() {
            return;
        }
        let messages = crate::i18n::I18nOps::get();
        Self::unsupported_fields_ui(ui, document, labels);
        let count = document.diagnostics.len().to_string();
        let title = crate::i18n::I18nOps::tf(
            &messages.preview.document_controller.rendering_notes,
            &[("count", &count)],
        );
        ui.collapsing(title, |ui| {
            egui::ScrollArea::vertical()
                .max_height(DIAGNOSTIC_MAX_HEIGHT)
                .show(ui, |ui| {
                    for diagnostic in &document.diagnostics {
                        Self::diagnostic_ui(ui, diagnostic, labels);
                    }
                });
        });
    }

    fn unsupported_fields_ui(
        ui: &mut egui::Ui,
        document: &CanvasDocument,
        labels: &CanvasSurfaceLabels<'_>,
    ) {
        if document
            .diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic, CanvasDiagnostic::UnsupportedFields { .. }))
        {
            ui.label(labels.unsupported_fields);
        }
    }

    fn diagnostic_ui(
        ui: &mut egui::Ui,
        diagnostic: &CanvasDiagnostic,
        labels: &CanvasSurfaceLabels<'_>,
    ) {
        match diagnostic {
            CanvasDiagnostic::UnsupportedNodeType { index, node_type } => {
                ui.label(labels.unsupported_node);
                ui.monospace(index.to_string());
                ui.monospace(node_type);
            }
            CanvasDiagnostic::UnsupportedFields { location, fields } => {
                ui.label(labels.unsupported_fields);
                /* WHY: 翻訳文と機械的な入力位置・キー名を分け、未対応データを欠落なく示す。 */
                ui.monospace(format!("{location:?}"));
                ui.monospace(fields.join(", "));
            }
        }
    }

    pub(super) fn error_ui(
        ui: &mut egui::Ui,
        error: &CanvasLoadError,
        labels: &CanvasSurfaceLabels<'_>,
    ) {
        ui.colored_label(ui.visuals().error_fg_color, labels.load_error);
        let messages = crate::i18n::I18nOps::get();
        ui.collapsing(&messages.preview.document_controller.error_details, |ui| {
            let metadata = match error {
                CanvasLoadError::Request(error) => format!("{error:?}"),
                CanvasLoadError::InvalidCanvas(error) => format!("{error:?}"),
                CanvasLoadError::UnrepresentableGeometry { index } => index.to_string(),
            };
            ui.label(metadata);
        });
    }
}
