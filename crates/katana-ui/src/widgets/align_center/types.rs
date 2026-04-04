use eframe::egui;

pub type AlignCenterNode<'a> = Box<dyn FnOnce(&mut egui::Ui) -> egui::Response + 'a>;

pub struct AlignCenter<'a> {
    pub(crate) left_nodes: Vec<AlignCenterNode<'a>>,
    pub(crate) right_nodes: Vec<AlignCenterNode<'a>>,
    pub(crate) spacing: f32,
    pub(crate) interactive: bool,
    pub(crate) width: Option<f32>,
    pub(crate) shrink_to_fit: bool,
}
