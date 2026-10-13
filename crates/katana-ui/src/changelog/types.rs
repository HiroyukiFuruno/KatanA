#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangelogSection {
    pub version: String,
    pub heading: String,
    pub body: String,
    pub default_open: bool,
}

pub enum ChangelogEvent {
    Success(Vec<ChangelogSection>),
    Error(String),
}

pub struct ChangelogOps;
