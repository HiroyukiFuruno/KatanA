use katana_core::document::Document;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewMode {
    PreviewOnly,
    CodeOnly,
    Split,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SplitViewState {
    pub direction: katana_platform::SplitDirection,
    pub order: katana_platform::PaneOrder,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TabViewMode {
    pub path: PathBuf,
    pub mode: ViewMode,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct TabGroup {
    pub id: String,
    pub name: String,
    pub color_hex: String,
    pub collapsed: bool,
    pub members: Vec<String>,
}

impl<'de> serde::Deserialize<'de> for TabGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TabGroupHelper {
            id: String,
            name: String,
            color_hex: Option<String>,
            color: Option<String>,
            #[serde(default)]
            collapsed: bool,
            members: Vec<String>,
        }

        let helper = TabGroupHelper::deserialize(deserializer)?;
        let mut color_hex = helper.color_hex.unwrap_or_else(|| "#4A90D9".to_string());

        if let Some(c) = helper.color {
            color_hex = match c.as_str() {
                "Blue" => "#4A90D9".to_string(),
                "Red" => "#D94A4A".to_string(),
                "Green" => "#4AD97A".to_string(),
                "Yellow" => "#D9A04A".to_string(),
                "Purple" => "#9B59B6".to_string(),
                "Gold" => "#F1C40F".to_string(),
                "Teal" => "#1ABC9C".to_string(),
                "Grey" | "Gray" => "#95A5A6".to_string(),
                _ => color_hex,
            };
        }

        Ok(TabGroup {
            id: helper.id,
            name: helper.name,
            color_hex,
            collapsed: helper.collapsed,
            members: helper.members,
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TabSplitState {
    pub path: PathBuf,
    pub state: SplitViewState,
}

pub struct DocumentState {
    pub open_documents: Vec<Document>,
    pub active_doc_idx: Option<usize>,
    pub tab_view_modes: Vec<TabViewMode>,
    pub tab_split_states: Vec<TabSplitState>,
    pub tab_groups: Vec<TabGroup>,
    pub recently_closed_tabs: VecDeque<(PathBuf, bool)>,
    pub last_auto_save: Option<Instant>,
    pub last_auto_refresh: Option<Instant>,
}

impl Default for DocumentState {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentState {
    pub const MAX_RECENTLY_CLOSED_TABS: usize = 10;

    pub fn new() -> Self {
        Self {
            open_documents: Vec::new(),
            active_doc_idx: None,
            tab_view_modes: Vec::new(),
            tab_split_states: Vec::new(),
            tab_groups: Vec::new(),
            recently_closed_tabs: VecDeque::with_capacity(Self::MAX_RECENTLY_CLOSED_TABS),
            last_auto_save: None,
            last_auto_refresh: None,
        }
    }

    pub fn active_document(&self) -> Option<&Document> {
        self.active_doc_idx
            .and_then(|idx| self.open_documents.get(idx))
    }

    pub fn cleanup_empty_groups(&mut self) {
        let open_paths: std::collections::HashSet<String> = self
            .open_documents
            .iter()
            .map(|d| d.path.to_string_lossy().to_string())
            .collect();
        for g in &mut self.tab_groups {
            g.members.retain(|m| open_paths.contains(m));
        }
        self.tab_groups.retain(|g| !g.members.is_empty());
    }
}
