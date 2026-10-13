use crate::i18n::I18nOps;
use crate::views::modals::meta_info_fields::MetaInfoFields;
use eframe::egui;
use std::path::Path;

const SPACING_SMALL: f32 = 10.0;
const SPACING_MEDIUM: f32 = 16.0;
const SPACING_LARGE: f32 = 20.0;

pub(crate) struct MetaInfoModal<'a> {
    pub is_open: &'a mut bool,
    pub path: &'a Path,
    pub actual_doc: Option<&'a katana_core::Document>,
}

impl<'a> MetaInfoModal<'a> {
    pub fn new(
        is_open: &'a mut bool,
        path: &'a Path,
        actual_doc: Option<&'a katana_core::Document>,
    ) -> Self {
        Self {
            is_open,
            path,
            actual_doc,
        }
    }

    pub fn show(self, ctx: &egui::Context) {
        let mut open = *self.is_open;
        let mut should_close = false;

        egui::Window::new(I18nOps::get().meta_info.title.clone())
            .open(&mut open)
            .show(ctx, |ui| {
                let i18n = I18nOps::get();
                let meta = &i18n.meta_info;

                let mock_doc = katana_core::Document::new(self.path.to_path_buf(), "");
                let doc = self.actual_doc.unwrap_or(&mock_doc);

                ui.vertical(|ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("meta_info_scroll")
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                MetaInfoFields::render_general_section(ui, doc, meta);
                                ui.add_space(SPACING_MEDIUM);
                                MetaInfoFields::render_filesystem_section(ui, self.path, meta);
                                ui.add_space(SPACING_MEDIUM);
                                MetaInfoFields::render_status_section(ui, doc, meta);
                            });
                        });

                    ui.add_space(SPACING_LARGE);
                    ui.separator();
                    ui.add_space(SPACING_SMALL);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(&i18n.tab.close).clicked() {
                            should_close = true;
                        }
                    });
                });
            });

        if should_close {
            *self.is_open = false;
        } else {
            *self.is_open = open;
        }
    }
}
