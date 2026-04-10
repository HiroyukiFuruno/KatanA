#![allow(unused_imports)]
#![allow(dead_code)]
use crate::Icon;
use crate::app_state::{AppAction, AppState};
use crate::icon::IconSize;
use crate::shell::KatanaApp;
use crate::state::update::UpdatePhase;
use katana_core::update::ReleaseInfo;

use crate::i18n;
use egui::{Align, Layout};
use std::path::{Path, PathBuf};

pub(crate) struct MetaInfoModal<'a> {
    pub open: &'a mut bool,
    pub path: &'a std::path::Path,
}

const MODAL_DEFAULT_WIDTH: f32 = 420.0;
const GRID_SPACING: [f32; 2] = [12.0, 8.0];
const HEADER_SPACING: f32 = 8.0;
const SECTION_SPACING: f32 = 12.0;

impl<'a> MetaInfoModal<'a> {
    pub fn new(open: &'a mut bool, path: &'a std::path::Path) -> Self {
        Self { open, path }
    }

    pub fn show(self, ctx: &egui::Context) {
        let path = self.path.to_path_buf(); // Move to owned if necessary, or just use ref
        egui::Window::new(crate::i18n::I18nOps::get().action.show_meta_info.clone())
            .open(self.open)
            .collapsible(false)
            .resizable(true)
            .default_width(MODAL_DEFAULT_WIDTH)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    Self::render_content(ui, &path);
                });
            });
    }

    fn render_content(ui: &mut egui::Ui, path: &Path) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let is_dir = path.is_dir();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let icon = if is_dir {
            Icon::FolderClosed
        } else if extension == "md" {
            Icon::Markdown
        } else {
            Icon::Document
        };

        /* Header with Icon and Name */
        ui.vertical_centered(|ui| {
            ui.add_space(HEADER_SPACING);
            let img = icon.ui_image(ui, IconSize::Large);
            ui.add(img);
            ui.add_space(HEADER_SPACING);
            ui.heading(name);
            ui.add_space(SECTION_SPACING);
        });

        ui.separator();

        /* General Section */
        ui.collapsing("General", |ui| {
            egui::Grid::new("meta_general_grid")
                .num_columns(2)
                .spacing(GRID_SPACING)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Kind:").weak());
                    let kind = if is_dir {
                        "Folder"
                    } else if extension == "md" {
                        "Markdown document"
                    } else {
                        "Document"
                    };
                    ui.label(kind);
                    ui.end_row();

                    if let Ok(metadata) = path.metadata() {
                        if !is_dir {
                            ui.label(egui::RichText::new("Size:").weak());
                            ui.label(crate::shell_logic::ShellLogicOps::format_file_size(
                                metadata.len(),
                            ));
                            ui.end_row();
                        }

                        ui.label(egui::RichText::new("Where:").weak());
                        ui.label(path.to_string_lossy());
                        ui.end_row();
                    }
                });
        });

        ui.separator();

        /* Dates Section */
        ui.collapsing("Dates", |ui| {
            egui::Grid::new("meta_dates_grid")
                .num_columns(2)
                .spacing(GRID_SPACING)
                .show(ui, |ui| {
                    if let Ok(metadata) = path.metadata()
                        && let Ok(modified) = metadata.modified()
                    {
                        ui.label(egui::RichText::new("Modified:").weak());
                        ui.label(crate::shell_logic::ShellLogicOps::format_modified_time(
                            modified,
                        ));
                        ui.end_row();
                    }
                });
        });

        ui.add_space(HEADER_SPACING);
    }
}
