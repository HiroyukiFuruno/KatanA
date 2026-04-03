pub struct LabeledColorPicker<'a> {
    pub(crate) label: &'a str,
    pub(crate) label_width: f32,
    pub(crate) spacing: f32,
    pub(crate) offset_y: f32,
    pub(crate) is_rgba: bool,
}
