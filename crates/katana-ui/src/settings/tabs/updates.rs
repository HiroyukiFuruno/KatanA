use std::path::PathBuf;

use crate::app_action::AppAction;
use crate::state::update::UpdatePhase;

const SECTION_SPACING: f32 = 8.0;
const SECTION_SEPARATOR_SPACING: f32 = 24.0;
const SECTION_AFTER_SEPARATOR_SPACING: f32 = 16.0;
const PLANTUML_DOWNLOAD_URL: &str =
    "https://github.com/plantuml/plantuml/releases/latest/download/plantuml.jar";
const DRAWIO_DOWNLOAD_URL: &str = "https://viewer.diagrams.net/js/viewer-static.min.js";
const MERMAID_DOWNLOAD_URL: &str = "https://cdn.jsdelivr.net/npm/mermaid/dist/mermaid.min.js";

struct RendererUpdateSection<'a, ActionBuilder> {
    title: &'a str,
    installed_path: Option<PathBuf>,
    default_path: Option<PathBuf>,
    installed_template: &'a str,
    not_installed_message: &'a str,
    update_label: &'a str,
    action: ActionBuilder,
}

impl crate::settings::tabs::UpdatesTabOps {
    pub(crate) fn render_updates_tab(
        ui: &mut egui::Ui,
        state: &mut crate::app_state::AppState,
        jar_path: Option<PathBuf>,
        drawio_path: Option<PathBuf>,
        mermaid_path: Option<PathBuf>,
    ) -> Option<AppAction> {
        let mut pending_action = None;
        let i18n_root = crate::i18n::I18nOps::get();
        let i18n_update = &i18n_root.update;
        let i18n_settings = &i18n_root.settings.updates;

        ui.vertical(|ui| {
            /* 1. App Updates Section */
            ui.heading(&i18n_settings.section_title);
            ui.add_space(SECTION_SPACING);

            let current_version = env!("CARGO_PKG_VERSION");
            crate::widgets::AlignCenter::new()
                .content(|ui| {
                    ui.label(&i18n_root.about.version);
                    ui.strong(current_version);
                })
                .show(ui);

            if let Some(update) = &state.update.available {
                ui.add_space(SECTION_SPACING);
                ui.label(&i18n_update.update_available);
                ui.strong(&update.tag_name);

                match &state.update.phase {
                    Some(UpdatePhase::Downloading { .. }) => {
                        ui.add_space(SECTION_SPACING);
                        crate::widgets::AlignCenter::new()
                            .content(|ui| {
                                ui.spinner();
                                ui.label(&i18n_update.downloading);
                            })
                            .show(ui);
                    }
                    Some(UpdatePhase::ReadyToRelaunch) => {
                        ui.add_space(SECTION_SPACING);
                        if ui.button(&i18n_update.install_update).clicked() {
                            pending_action = Some(AppAction::InstallUpdateAndRestart);
                        }
                    }
                    _ => {
                        ui.add_space(SECTION_SPACING);
                        if ui.button(&i18n_update.download_update).clicked() {
                            pending_action = Some(AppAction::StartUpdateDownload);
                        }
                    }
                }
            } else if state.update.checking {
                ui.add_space(SECTION_SPACING);
                crate::widgets::AlignCenter::new()
                    .content(|ui| {
                        ui.spinner();
                        ui.label(&i18n_update.checking_for_updates);
                    })
                    .show(ui);
            } else {
                ui.add_space(SECTION_SPACING);
                ui.label(&i18n_update.up_to_date);
                if ui.button(&i18n_settings.check_now).clicked() {
                    pending_action = Some(AppAction::CheckForUpdates);
                }
            }

            ui.add_space(SECTION_SEPARATOR_SPACING);
            ui.separator();
            ui.add_space(SECTION_AFTER_SEPARATOR_SPACING);

            let section_action = Self::render_renderer_update_section(
                ui,
                RendererUpdateSection {
                    title: &i18n_settings.plantuml_section_title,
                    installed_path: jar_path,
                    default_path: state.config.try_get_plantuml_jar_path(),
                    installed_template: &i18n_settings.plantuml_installed,
                    not_installed_message: &i18n_settings.plantuml_not_installed,
                    update_label: &i18n_settings.plantuml_update_now,
                    action: |dest| AppAction::StartPlantumlDownload {
                        url: PLANTUML_DOWNLOAD_URL.to_string(),
                        dest,
                    },
                },
            );
            if pending_action.is_none() {
                pending_action = section_action;
            }

            ui.add_space(SECTION_SEPARATOR_SPACING);
            ui.separator();
            ui.add_space(SECTION_AFTER_SEPARATOR_SPACING);

            let section_action = Self::render_renderer_update_section(
                ui,
                RendererUpdateSection {
                    title: &i18n_settings.drawio_section_title,
                    installed_path: drawio_path,
                    default_path: state.config.try_get_drawio_js_path(),
                    installed_template: &i18n_settings.drawio_installed,
                    not_installed_message: &i18n_settings.drawio_not_installed,
                    update_label: &i18n_settings.drawio_update_now,
                    action: |dest| AppAction::StartDrawioDownload {
                        url: DRAWIO_DOWNLOAD_URL.to_string(),
                        dest,
                    },
                },
            );
            if pending_action.is_none() {
                pending_action = section_action;
            }

            ui.add_space(SECTION_SEPARATOR_SPACING);
            ui.separator();
            ui.add_space(SECTION_AFTER_SEPARATOR_SPACING);

            let section_action = Self::render_renderer_update_section(
                ui,
                RendererUpdateSection {
                    title: &i18n_settings.mermaid_section_title,
                    installed_path: mermaid_path,
                    default_path: state.config.try_get_mermaid_js_path(),
                    installed_template: &i18n_settings.mermaid_installed,
                    not_installed_message: &i18n_settings.mermaid_not_installed,
                    update_label: &i18n_settings.mermaid_update_now,
                    action: |dest| AppAction::StartMermaidDownload {
                        url: MERMAID_DOWNLOAD_URL.to_string(),
                        dest,
                    },
                },
            );
            if pending_action.is_none() {
                pending_action = section_action;
            }
        });

        pending_action
    }

    fn render_renderer_update_section<ActionBuilder>(
        ui: &mut egui::Ui,
        section: RendererUpdateSection<'_, ActionBuilder>,
    ) -> Option<AppAction>
    where
        ActionBuilder: FnOnce(PathBuf) -> AppAction,
    {
        ui.heading(section.title);
        ui.add_space(SECTION_SPACING);

        let target_path = section.installed_path.or(section.default_path);
        if let Some(path) = target_path {
            if path.exists() {
                let path_str = path.to_string_lossy().to_string();
                ui.label(
                    egui::RichText::new(crate::i18n::I18nOps::tf(
                        section.installed_template,
                        &[("path", &path_str)],
                    ))
                    .color(ui.visuals().weak_text_color()),
                );
            } else {
                ui.label(
                    egui::RichText::new(section.not_installed_message)
                        .color(ui.visuals().warn_fg_color),
                );
            }

            ui.add_space(SECTION_SPACING);
            if ui.button(section.update_label).clicked() {
                return Some((section.action)(path));
            }
        } else {
            ui.label(
                egui::RichText::new(section.not_installed_message)
                    .color(ui.visuals().warn_fg_color),
            );
        }
        None
    }
}
