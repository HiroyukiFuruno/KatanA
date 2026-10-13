use eframe::egui;

pub(super) fn apply_extension_toggle(
    ui: &mut egui::Ui,
    ext: &str,
    is_enabled: bool,
    extensions: &mut Vec<String>,
) -> bool {
    if is_enabled {
        if ext.is_empty() {
            ui.data_mut(|data| data.insert_temp(egui::Id::new("show_no_extension_warning"), true));
            false
        } else if !extensions.contains(&ext.to_string()) {
            extensions.push(ext.to_string());
            true
        } else {
            false
        }
    } else {
        let before = extensions.len();
        extensions.retain(|extension| extension != ext);
        extensions.len() != before
    }
}

pub(super) fn is_standard_visible_extension(ext: &str) -> bool {
    katana_core::workspace::TreeEntry::standard_visible_extensions()
        .iter()
        .any(|standard| standard.eq_ignore_ascii_case(ext))
}
