pub struct Modal<'a> {
    pub(crate) id: &'a str,
    pub(crate) title: &'a str,
    pub(crate) progress: Option<f32>,
    pub(crate) show_pct: bool,
    pub(crate) bar_width: f32,
    pub(crate) width: Option<f32>,
}
