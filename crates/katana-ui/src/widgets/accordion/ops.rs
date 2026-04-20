use super::types::*;
use eframe::egui;

impl<'a> Accordion<'a> {
    pub fn new(
        id_source: impl std::hash::Hash,
        label: impl Into<egui::WidgetText>,
        body: impl FnOnce(&mut egui::Ui) + 'a,
    ) -> Self {
        Self {
            id_source: egui::Id::new(id_source),
            label: label.into(),
            default_open: false,
            force_open: None,
            show_vertical_line: false,
            active: false,
            primary: false,
            icon_only_toggle: false,
            indent: None,
            body: Box::new(body),
        }
    }

    pub fn icon_only_toggle(mut self, icon_only: bool) -> Self {
        self.icon_only_toggle = icon_only;
        self
    }

    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = Some(indent);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn show_vertical_line(mut self, show: bool) -> Self {
        self.show_vertical_line = show;
        self
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn force_open(mut self, force_open: Option<bool>) -> Self {
        self.force_open = force_open;
        self
    }

    pub fn open(mut self, open: Option<bool>) -> Self {
        self.force_open = open;
        self
    }
}
