use super::meta_info_logic::MetaInfoLogic;
use eframe::egui;
use std::path::Path;

/* WHY: Vertical spacing for metadata grids. */
const GRID_SPACING: f32 = 4.0;
/* WHY: Column width for metadata keys. */
const META_GRID_MIN_COL_WIDTH: f32 = 80.0;

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
                    .unwrap_or(&meta.unknown),
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

    pub fn render_filesystem_section(
        ui: &mut egui::Ui,
        path: &Path,
        meta: &crate::i18n::MetaInfoMessages,
    ) {
        if path.to_string_lossy().starts_with("Katana://") {
            Self::render_virtual_fs_grid(ui, meta);
            return;
        }

        let Ok(fs_meta) = std::fs::metadata(path) else {
            return;
        };

        ui.strong(&meta.dates_section);
        ui.add_space(GRID_SPACING);

        egui::Grid::new("meta_fs_grid")
            .min_col_width(META_GRID_MIN_COL_WIDTH)
            .show(ui, |ui| {
                ui.label(&meta.label_size);
                ui.label(MetaInfoLogic::format_file_size(fs_meta.len()));
                ui.end_row();

                Self::render_modified_row(ui, &fs_meta, meta);
                Self::render_created_row(ui, &fs_meta, meta);

                #[cfg(unix)]
                Self::render_unix_meta_fields(ui, &fs_meta, meta);
            });
    }

    fn render_virtual_fs_grid(ui: &mut egui::Ui, meta: &crate::i18n::MetaInfoMessages) {
        ui.strong(&meta.dates_section);
        ui.add_space(GRID_SPACING);
        egui::Grid::new("meta_fs_grid_virtual").show(ui, |ui| {
            ui.label(&meta.label_size);
            ui.label(&meta.virtual_embedded);
            ui.end_row();
            ui.label(&meta.label_kind);
            ui.label(&meta.kind_virtual);
            ui.end_row();
        });
    }

    fn render_modified_row(
        ui: &mut egui::Ui,
        fs_meta: &std::fs::Metadata,
        meta: &crate::i18n::MetaInfoMessages,
    ) {
        /* WHY: Modification time. Use let-else to skip if not available on platform. */
        let Ok(modified) = fs_meta.modified() else {
            return;
        };
        ui.label(&meta.label_updated);
        ui.label(MetaInfoLogic::format_system_time(modified));
        ui.end_row();
    }

    fn render_created_row(
        ui: &mut egui::Ui,
        fs_meta: &std::fs::Metadata,
        meta: &crate::i18n::MetaInfoMessages,
    ) {
        /* WHY: Creation time. Use let-else to skip if not available on platform. */
        let Ok(created) = fs_meta.created() else {
            return;
        };
        ui.label(&meta.label_created);
        ui.label(MetaInfoLogic::format_system_time(created));
        ui.end_row();
    }

    #[cfg(unix)]
    fn render_unix_meta_fields(
        ui: &mut egui::Ui,
        fs_meta: &std::fs::Metadata,
        meta: &crate::i18n::MetaInfoMessages,
    ) {
        use std::os::unix::fs::MetadataExt;
        let uid = fs_meta.uid();
        ui.label(&meta.label_owner);
        ui.label(MetaInfoLogic::resolve_unix_owner(uid));
        ui.end_row();

        let mode = fs_meta.mode();
        ui.label(&meta.label_permissions);
        ui.label(MetaInfoLogic::format_unix_permissions(mode));
        ui.end_row();
    }
}
