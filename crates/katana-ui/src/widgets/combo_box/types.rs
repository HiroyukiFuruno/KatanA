pub struct StyledComboBox<'a> {
    pub id: &'a str,
    pub selected_text: String,
    pub width: Option<f32>,
    pub truncate: bool,
}
