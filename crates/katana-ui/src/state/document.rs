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

impl TabGroup {
    pub const DEMO_ID: &'static str = "demo";

    /// Whether this is the built-in Demo group that users cannot add files to.
    pub fn is_demo(&self) -> bool {
        self.id == Self::DEMO_ID
    }
}

pub trait VirtualPathExt {
    /// `Katana://Demo/` prefix — part of the built-in Demo bundle.
    fn is_demo_path(&self) -> bool;
    /// `Katana://` prefix (includes Demo, Welcome, Guide, ChangeLog …).
    fn is_virtual_path(&self) -> bool;
}

impl VirtualPathExt for std::path::Path {
    fn is_demo_path(&self) -> bool {
        self.to_string_lossy().starts_with("Katana://Demo/")
    }

    fn is_virtual_path(&self) -> bool {
        self.to_string_lossy().starts_with("Katana://")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn demo_group() -> TabGroup {
        TabGroup {
            id: TabGroup::DEMO_ID.to_string(),
            name: "Demo".to_string(),
            color_hex: "#808080".to_string(),
            collapsed: false,
            members: vec!["Katana://Demo/feature_walkthrough.md".to_string()],
        }
    }

    fn user_group(name: &str) -> TabGroup {
        TabGroup {
            id: format!("group_{name}"),
            name: name.to_string(),
            color_hex: "#4A90D9".to_string(),
            collapsed: false,
            members: vec!["/some/file.md".to_string()],
        }
    }

    /* WHY: TabGroup::is_demo tests */

    #[test]
    fn is_demo_true_for_demo_id() {
        assert!(demo_group().is_demo());
    }

    #[test]
    fn is_demo_false_for_user_group_named_demo() {
        /* WHY: A user-created group named 'Demo' must NOT be treated as system Demo */
        let g = TabGroup {
            id: "group_123456".to_string(),
            name: "Demo".to_string(),
            color_hex: "#4A90D9".to_string(),
            collapsed: false,
            members: vec![],
        };
        assert!(!g.is_demo());
    }

    #[test]
    fn is_demo_false_for_regular_group() {
        assert!(!user_group("Work").is_demo());
    }

    /* WHY: VirtualPathExt::is_demo_path tests */

    #[test]
    fn is_demo_path_true_for_katana_demo_prefix() {
        assert!(Path::new("Katana://Demo/feature_walkthrough.md").is_demo_path());
        assert!(Path::new("Katana://Demo/rendering_features.md").is_demo_path());
    }

    #[test]
    fn is_demo_path_false_for_welcome() {
        assert!(!Path::new("Katana://Welcome.md").is_demo_path());
    }

    #[test]
    fn is_demo_path_false_for_regular_file() {
        assert!(!Path::new("/home/user/notes.md").is_demo_path());
    }

    /* WHY: VirtualPathExt::is_virtual_path tests */

    #[test]
    fn is_virtual_path_true_for_demo() {
        assert!(Path::new("Katana://Demo/feature_walkthrough.md").is_virtual_path());
    }

    #[test]
    fn is_virtual_path_true_for_welcome() {
        assert!(Path::new("Katana://Welcome.md").is_virtual_path());
    }

    #[test]
    fn is_virtual_path_true_for_guide() {
        assert!(Path::new("Katana://Guide.md").is_virtual_path());
    }

    #[test]
    fn is_virtual_path_true_for_changelog() {
        assert!(Path::new("Katana://ChangeLog v0.18.6").is_virtual_path());
    }

    #[test]
    fn is_virtual_path_false_for_real_file() {
        assert!(!Path::new("/home/user/notes.md").is_virtual_path());
    }

    /* WHY: addable_groups filter — system Demo excluded, user-created 'Demo' name group included */

    #[test]
    fn addable_groups_excludes_demo_group_but_includes_user_demo_named_group() {
        let system_demo = demo_group();
        let user_demo_named = TabGroup {
            id: "group_abc".to_string(),
            name: "Demo".to_string(),
            color_hex: "#4A90D9".to_string(),
            collapsed: false,
            members: vec![],
        };
        let work = user_group("Work");
        let all_groups = [system_demo, user_demo_named, work];

        let addable: Vec<_> = all_groups.iter().filter(|g| !g.is_demo()).collect();
        assert_eq!(addable.len(), 2, "only system Demo should be excluded");
        assert!(
            addable.iter().any(|g| g.name == "Demo"),
            "user-created 'Demo' group must remain addable"
        );
        assert!(
            addable.iter().any(|g| g.name == "Work"),
            "Work group must remain addable"
        );
    }
}
