use eframe::egui;

pub(crate) struct BreadcrumbMenu<'a> {
    pub entries: &'a [katana_core::workspace::TreeEntry],
    pub action: &'a mut crate::app_state::AppAction,
}

impl<'a> BreadcrumbMenu<'a> {
    pub fn new(
        entries: &'a [katana_core::workspace::TreeEntry],
        action: &'a mut crate::app_state::AppAction,
    ) -> Self {
        Self { entries, action }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let entries = self.entries;
        let action = self.action;
        for entry in entries {
            match entry {
                katana_core::workspace::TreeEntry::Directory { path, children } => {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    ui.menu_button(name, |ui| {
                        BreadcrumbMenu::new(children, action).show(ui);
                    });
                }
                katana_core::workspace::TreeEntry::File { path } => {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                    if ui.button(name).clicked() {
                        *action = crate::app_state::AppAction::SelectDocument(path.clone());
                        ui.close();
                    }
                }
            }
        }
    }
}
