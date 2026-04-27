use std::path::PathBuf;

use crate::app_action::AppAction;

const SECTION_SPACING: f32 = 8.0;

pub(crate) struct RendererUpdateSection<'a, ActionBuilder> {
    pub(crate) title: &'a str,
    pub(crate) installed_path: Option<PathBuf>,
    pub(crate) default_path: Option<PathBuf>,
    pub(crate) installed_template: &'a str,
    pub(crate) not_installed_message: &'a str,
    pub(crate) update_label: &'a str,
    pub(crate) action: ActionBuilder,
}

pub(crate) struct RendererUpdateSectionOps;

impl RendererUpdateSectionOps {
    pub(crate) fn render<ActionBuilder>(
        ui: &mut egui::Ui,
        section: RendererUpdateSection<'_, ActionBuilder>,
    ) -> Option<AppAction>
    where
        ActionBuilder: FnOnce(PathBuf) -> AppAction,
    {
        ui.heading(section.title);
        ui.add_space(SECTION_SPACING);

        let target_path = section
            .installed_path
            .clone()
            .or_else(|| section.default_path.clone());
        let Some(path) = target_path else {
            ui.label(
                egui::RichText::new(section.not_installed_message)
                    .color(ui.visuals().warn_fg_color),
            );
            return None;
        };

        Self::render_status(ui, &path, &section);
        ui.add_space(SECTION_SPACING);
        ui.button(section.update_label)
            .clicked()
            .then(|| (section.action)(path))
    }

    fn render_status<ActionBuilder>(
        ui: &mut egui::Ui,
        path: &std::path::Path,
        section: &RendererUpdateSection<'_, ActionBuilder>,
    ) {
        if path.exists() {
            let path_str = path.to_string_lossy().to_string();
            ui.label(
                egui::RichText::new(crate::i18n::I18nOps::tf(
                    section.installed_template,
                    &[("path", &path_str)],
                ))
                .color(ui.visuals().weak_text_color()),
            );
            return;
        }

        ui.label(
            egui::RichText::new(section.not_installed_message).color(ui.visuals().warn_fg_color),
        );
    }
}
