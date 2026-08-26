use eframe::egui;
use katana_document_viewer::{DocumentViewerCommand, ViewerDocumentFormat};

use super::types::DocumentSurface;
use super::worker::DocumentWorkerCommand;

pub(super) fn show_sheet_tabs(
    surface: &mut DocumentSurface,
    ui: &mut egui::Ui,
    frame: &katana_document_viewer::DocumentFrame,
) {
    if frame.format != ViewerDocumentFormat::Xlsx {
        return;
    }
    egui::ScrollArea::horizontal()
        .id_salt(("document-sheet-tabs", surface.generation))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                for (index, label) in
                    sheet_tab_labels(frame.surface.item_labels(), frame.state.item_count)
                {
                    let selected = index == frame.state.active_index;
                    if ui
                        .add(egui::Button::selectable(selected, label).frame_when_inactive(true))
                        .clicked()
                    {
                        surface.queue(DocumentWorkerCommand::Viewer(
                            DocumentViewerCommand::JumpTo(index),
                        ));
                    }
                }
            });
        });
}

fn sheet_tab_labels(labels: &[String], item_count: usize) -> Vec<(usize, String)> {
    (0..item_count)
        .map(|index| {
            let label = labels
                .get(index)
                .filter(|label| !label.trim().is_empty())
                .cloned()
                .unwrap_or_else(|| index.saturating_add(1).to_string());
            (index, label)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sheet_tab_labels;

    #[test]
    fn sheet_tabs_use_document_labels_with_a_bounded_fallback() {
        assert_eq!(
            sheet_tab_labels(&["Summary".to_owned(), "Data".to_owned()], 3),
            vec![
                (0, "Summary".to_owned()),
                (1, "Data".to_owned()),
                (2, "3".to_owned())
            ]
        );
    }
}
