#![allow(unused_imports)]
#![allow(dead_code)]
use crate::Icon;
use crate::app_state::{AppAction, AppState};
use crate::shell::KatanaApp;
use crate::state::update::UpdatePhase;
use katana_core::update::ReleaseInfo;

use crate::i18n;
use egui::{Align, Layout};
use std::path::{Path, PathBuf};

fn resolve_actual_name(name: &str, selected_ext: &Option<String>, is_dir: bool) -> String {
    if is_dir {
        return name.to_string();
    }
    let Some(ext) = selected_ext else {
        return name.to_string();
    };
    if name.ends_with(&format!(".{}", ext)) {
        name.to_string()
    } else {
        format!("{}.{}", name, ext)
    }
}

pub(crate) struct CreateFsNodeModal<'a> {
    pub modal_data: &'a mut (PathBuf, String, Option<String>, bool),
    pub visible_extensions: &'a [String],
    pub pending_action: &'a mut crate::app_state::AppAction,
}

impl<'a> CreateFsNodeModal<'a> {
    pub fn new(
        modal_data: &'a mut (PathBuf, String, Option<String>, bool),
        visible_extensions: &'a [String],
        pending_action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self {
            modal_data,
            visible_extensions,
            pending_action,
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let (parent_dir, name, selected_ext, is_dir) = self.modal_data;
        let pending_action = self.pending_action;
        let mut close = false;
        let mut do_create = false;

        let title = if *is_dir {
            crate::i18n::I18nOps::get()
                .dialog
                .new_directory_title
                .clone()
        } else {
            crate::i18n::I18nOps::get().dialog.new_file_title.clone()
        };

        let mut is_open = true;
        egui::Window::new(title)
            .open(&mut is_open)
            .collapsible(false)
            .resizable(false)
            .max_width({
                const MAX_MODAL_WIDTH: f32 = 300.0;
                MAX_MODAL_WIDTH
            })
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        const MODAL_INPUT_WIDTH: f32 = 200.0;
                        let re = ui.add(
                            egui::TextEdit::singleline(name)
                                .hint_text(&crate::i18n::I18nOps::get().dialog.name_hint)
                                .desired_width(MODAL_INPUT_WIDTH),
                        );
                        re.request_focus();

                        if !*is_dir && let Some(ext) = selected_ext {
                            const EXT_COMBOBOX_WIDTH: f32 = 80.0;
                            let options = self.visible_extensions;
                            crate::widgets::StyledComboBox::new("new_file_ext", ext.as_str())
                                .width(EXT_COMBOBOX_WIDTH)
                                .show(ui, |ui| {
                                    for opt in options {
                                        /* WHY: in popup/list context; future: standardize as atom */
                                        if ui
                                            .add(
                                                egui::Button::selectable(*ext == *opt, opt.clone())
                                                    .frame_when_inactive(true),
                                            )
                                            .clicked()
                                        {
                                            *ext = opt.clone();
                                        }
                                    }
                                });
                        }

                        if re.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            do_create = true;
                        }
                    })
                    .show(ui);
                const SPACING_SMALL: f32 = 8.0;
                ui.add_space(SPACING_SMALL);
                crate::widgets::AlignCenter::new()
                    .shrink_to_fit(true)
                    .content(|ui| {
                        if ui
                            .button(crate::i18n::I18nOps::get().action.cancel.clone())
                            .clicked()
                        {
                            close = true;
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(crate::i18n::I18nOps::get().action.save.clone())
                                .clicked()
                            {
                                do_create = true;
                            }
                        });
                    })
                    .show(ui);
            });

        if !is_open {
            close = true;
        }

        if do_create && !name.is_empty() {
            let actual_name = resolve_actual_name(name, selected_ext, *is_dir);

            let target_path = parent_dir.join(&actual_name);
            *pending_action = crate::app_state::AppAction::CreateFsNode {
                target_path,
                is_dir: *is_dir,
                parent_dir: parent_dir.clone(),
            };
            close = true;
        }

        close
    }
}

pub(crate) use super::file_ops_rename_delete::{DeleteModal, RenameModal};
