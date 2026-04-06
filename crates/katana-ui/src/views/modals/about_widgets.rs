pub(super) fn about_section_header(ui: &mut egui::Ui, title: &str, size: f32, bottom: f32) {
    ui.separator();
    ui.label(egui::RichText::new(title).strong().size(size));
    ui.add_space(bottom);
}

pub(super) fn about_row(ui: &mut egui::Ui, label: &str, value: &str) {
    crate::widgets::AlignCenter::new()
        .interactive(false)
        .left(|ui| ui.label(egui::RichText::new(label).weak()))
        .right(|ui| ui.add(egui::Label::new(value).truncate()))
        .show(ui);
}

pub(super) fn about_link_row(ui: &mut egui::Ui, label: &str, url: &str, icon: crate::Icon) {
    let url_copy = url.to_string();
    crate::widgets::AlignCenter::new()
        .interactive(false)
        .left(move |ui| {
            ui.vertical(|ui| {
                ui.add_space(2.0);
                ui.add(icon.ui_image(ui, crate::icon::IconSize::Medium))
            })
            .inner
        })
        .left(|ui| ui.label(egui::RichText::new(label).weak()))
        .right(move |ui| {
            let btn = ui
                .add(
                    crate::Icon::ExternalLink
                        .button(ui, crate::icon::IconSize::Small)
                        .frame(false),
                )
                .on_hover_text(&url_copy);
            if btn.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(&url_copy));
            }
            btn
        })
        .show(ui);
}
