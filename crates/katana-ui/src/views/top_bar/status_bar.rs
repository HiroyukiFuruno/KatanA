use crate::app_state::AppAction;
use crate::shell_ui::{STATUS_BAR_ICON_SPACING, STATUS_SUCCESS_GREEN};
use eframe::egui;




pub(crate) struct StatusBar<'a> {
    pub status: Option<&'a (String, crate::app_state::StatusType)>,
    pub is_dirty: bool,
    pub export_filenames: &'a [String],
}

impl<'a> StatusBar<'a> {
    pub fn new(
        status: Option<&'a (String, crate::app_state::StatusType)>,
        is_dirty: bool,
        export_filenames: &'a [String],
    ) -> Self {
        Self {
            status,
            is_dirty,
            export_filenames,
        }
    }

    pub fn show(self, ui: &mut egui::Ui, problem_count: usize) -> Option<AppAction> {
        let export_filenames = self.export_filenames;
        let mut action = None;
        let row_height = ui.spacing().interact_size.y;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let (msg, kind) = if let Some((msg, kind)) = self.status {
                    (msg.as_str(), Some(kind))
                } else {
                    (crate::i18n::I18nOps::get().status.ready.as_str(), None)
                };

                let (color, icon) = match kind {
                    Some(crate::app_state::StatusType::Error) => (
                        ui.ctx()
                            .data(|d| {
                                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                                    "katana_theme_colors",
                                ))
                            })
                            .map_or(crate::theme_bridge::WHITE, |tc| {
                                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(
                                    tc.system.error_text,
                                )
                            }),
                        Some(crate::Icon::Error),
                    ),
                    Some(crate::app_state::StatusType::Warning) => {
                        (ui.visuals().warn_fg_color, Some(crate::Icon::Warning))
                    }
                    Some(crate::app_state::StatusType::Success) => (
                        crate::theme_bridge::ThemeBridgeOps::from_rgb(0, STATUS_SUCCESS_GREEN, 0),
                        Some(crate::Icon::Success),
                    ),
                    Some(crate::app_state::StatusType::Info) => {
                        (ui.visuals().text_color(), Some(crate::Icon::Info))
                    }
                    _ => (ui.visuals().text_color(), None),
                };

                ui.add_space(STATUS_BAR_ICON_SPACING);
                if let Some(i) = icon {
                    ui.add(i.image(crate::icon::IconSize::Medium).tint(color));
                }
                crate::icon::IconOps::render_str_with_icons(ui, msg, Some(color));

                let problem_text = crate::i18n::I18nOps::tf(
                    &crate::i18n::I18nOps::get().status.problems_count_format,
                    &[("count", &problem_count.to_string())],
                );
                let btn = egui::Button::image_and_text(
                    crate::Icon::Warning.ui_image(ui, crate::icon::IconSize::Small),
                    problem_text,
                );
                if ui
                    .add(btn)
                    .on_hover_text(
                        crate::i18n::I18nOps::get()
                            .status
                            .toggle_problems_panel
                            .clone(),
                    )
                    .clicked()
                {
                    action = Some(AppAction::ToggleProblemsPanel);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !export_filenames.is_empty() {
                        let total = export_filenames.len();
                        ui.spinner();
                        for (i, filename) in export_filenames.iter().enumerate() {
                            let numbered = crate::i18n::I18nOps::tf(
                                &crate::i18n::I18nOps::get().export.exporting,
                                &[("filename", &format!("({}/{}) {}", i + 1, total, filename))],
                            );
                            crate::icon::IconOps::render_str_with_icons(ui, &numbered, None);
                        }
                    }
                    const DIRTY_DOT_MAX_HEIGHT: f32 = 10.0;
                    if self.is_dirty {
                        ui.add(
                            egui::Image::new(crate::Icon::Dot.uri())
                                .tint(ui.visuals().text_color())
                                .fit_to_exact_size(egui::vec2(
                                    DIRTY_DOT_MAX_HEIGHT,
                                    DIRTY_DOT_MAX_HEIGHT,
                                )),
                        );
                    }
                });
            },
        );
        action
    }
}

