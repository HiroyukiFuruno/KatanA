#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineItem {
    pub level: u8,
    pub text: String,
    pub index: usize,
}
