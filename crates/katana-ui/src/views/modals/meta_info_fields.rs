use eframe::egui;

const GRID_SPACING: f32 = 4.0;

pub struct MetaInfoFields;

impl MetaInfoFields {
    pub fn render_general_section(
        ui: &mut egui::Ui,
        doc: &katana_core::Document,
        meta: &crate::i18n::MetaInfoMessages,
    ) {
        ui.strong(&meta.general_section);
        ui.add_space(GRID_SPACING);
        egui::Grid::new("meta_general_grid").show(ui, |ui| {
            ui.label(&meta.label_name);
            ui.label(
                doc.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown"),
            );
            ui.end_row();

            ui.label(&meta.label_path);
            ui.label(doc.path.display().to_string());
            ui.end_row();

            ui.label(&meta.label_kind);
            ui.label(&meta.kind_markdown);
            ui.end_row();
        });
    }

    pub fn render_status_section(
        ui: &mut egui::Ui,
        doc: &katana_core::Document,
        meta: &crate::i18n::MetaInfoMessages,
    ) {
        ui.strong(&meta.status_section);
        ui.add_space(GRID_SPACING);
        egui::Grid::new("meta_status_grid").show(ui, |ui| {
            ui.label(&meta.label_dirty);
            ui.label(if doc.is_dirty { &meta.yes } else { &meta.no });
            ui.end_row();

            ui.label(&meta.label_loaded);
            ui.label(if doc.is_loaded { &meta.yes } else { &meta.no });
            ui.end_row();

            ui.label(&meta.label_pinned);
            ui.label(if doc.is_pinned { &meta.yes } else { &meta.no });
            ui.end_row();
        });
    }

    pub fn render_dates_section(
        _ui: &mut egui::Ui,
        _doc: &katana_core::Document,
        _meta: &crate::i18n::MetaInfoMessages,
    ) {
    }
}
