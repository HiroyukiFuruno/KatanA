#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TogglePosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToggleAlignment {
    Attached(f32),
    SpaceBetween,
}

pub struct LabeledToggle<'a> {
    pub(crate) text: egui::WidgetText,
    pub(crate) on: &'a mut bool,
    pub(crate) position: TogglePosition,
    pub alignment: ToggleAlignment,
}

pub struct ToggleOps;

impl<'a> LabeledToggle<'a> {
    pub fn new(text: impl Into<egui::WidgetText>, on: &'a mut bool) -> Self {
        Self {
            text: text.into(),
            on,
            position: TogglePosition::Right,
            alignment: ToggleAlignment::SpaceBetween,
        }
    }

    pub fn position(mut self, position: TogglePosition) -> Self {
        self.position = position;
        self
    }

    pub fn alignment(mut self, alignment: ToggleAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}
