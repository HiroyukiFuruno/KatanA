use eframe::egui;

pub struct Accordion<'a> {
    pub(crate) id_source: egui::Id,
    pub(crate) label: egui::WidgetText,
    pub(crate) default_open: bool,
    pub(crate) force_open: Option<bool>,
    pub(crate) show_vertical_line: bool,
    pub(crate) active: bool,
    pub(crate) primary: bool,
    pub(crate) indent: Option<f32>,
    pub(crate) body: Box<dyn FnOnce(&mut egui::Ui) + 'a>,
}
