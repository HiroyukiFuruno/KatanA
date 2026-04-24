use crate::app_state::AppAction;
use eframe::egui;

pub(crate) struct ReferencedImagesSection<'a> {
    pub paths: &'a [std::path::PathBuf],
    pub action: &'a mut AppAction,
}

impl<'a> ReferencedImagesSection<'a> {
    pub fn new(paths: &'a [std::path::PathBuf], action: &'a mut AppAction) -> Self {
        Self { paths, action }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        if self.paths.is_empty() {
            return;
        }
        ui.separator();
        let section_label = &crate::i18n::I18nOps::get()
            .workspace
            .referenced_images_section;
        egui::CollapsingHeader::new(section_label.as_str())
            .default_open(true)
            .show(ui, |ui| {
                for path in self.paths {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let resp = ui.add(
                        egui::Label::new(&name)
                            .truncate()
                            .selectable(false)
                            .sense(egui::Sense::click()),
                    );
                    if resp.clicked() {
                        *self.action = AppAction::RevealImageAsset(path.clone());
                    }
                    resp.on_hover_text(path.to_string_lossy().as_ref());
                }
            });
    }
}
