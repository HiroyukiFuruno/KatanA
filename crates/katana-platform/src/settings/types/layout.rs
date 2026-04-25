use super::behavior::ActivityRailItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum PaneOrder {
    #[default]
    EditorFirst,
    PreviewFirst,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SplitDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TocPosition {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub toc_visible: bool,
    #[serde(default)]
    pub toc_position: TocPosition,
    #[serde(default)]
    pub split_direction: SplitDirection,
    #[serde(default)]
    pub pane_order: PaneOrder,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub sidebar_visible: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub toolbar_visible: bool,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub status_bar_visible: bool,
    #[serde(default)]
    pub active_pane_idx: usize,
    #[serde(default)]
    pub activity_rail_order: Vec<ActivityRailItem>,
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub accordion_vertical_line: bool,
    /* WHY: Controls the default pin state of the TOC panel at startup.
     * false = starts collapsed (hover-only); true = starts pinned.
     * Key absent from JSON → false (never written back to settings). */
    #[serde(default)]
    pub toc_default_visible: bool,
    /* WHY: Controls the default pin state of the Explorer panel at startup.
     * true = starts pinned (default); false = starts collapsed (hover-only).
     * Key absent from JSON → true (never written back to settings). */
    #[serde(default = "super::super::defaults::SettingsDefaultOps::true_default")]
    pub explorer_default_visible: bool,
}

impl LayoutSettings {
    pub(crate) fn normalize(&mut self) {
        let mut new_order = Vec::new();

        /* WHY: Ensure required new workspace items are always at the top if they were previously missing */
        let missing_add = !self
            .activity_rail_order
            .contains(&ActivityRailItem::AddWorkspace);
        let missing_toggle = !self
            .activity_rail_order
            .contains(&ActivityRailItem::WorkspaceToggle);

        if missing_add {
            new_order.push(ActivityRailItem::AddWorkspace);
        }
        if missing_toggle {
            new_order.push(ActivityRailItem::WorkspaceToggle);
        }

        /* WHY: Keep the user's existing order for the rest */
        for item in &self.activity_rail_order {
            if !new_order.contains(item) {
                new_order.push(*item);
            }
        }

        /* WHY: Append any other missing defaults at the bottom */
        let defaults = vec![
            ActivityRailItem::AddWorkspace,
            ActivityRailItem::WorkspaceToggle,
            ActivityRailItem::ExplorerToggle,
            ActivityRailItem::Search,
            ActivityRailItem::Chat,
            ActivityRailItem::History,
        ];

        for item in defaults {
            if !new_order.contains(&item) {
                new_order.push(item);
            }
        }

        self.activity_rail_order = new_order;
    }
}
