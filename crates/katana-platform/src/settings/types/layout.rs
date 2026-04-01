use serde::{Deserialize, Serialize};

// WHY: Split direction for editor/preview layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SplitDirection {
    // WHY: Editor on left, preview on right.
    #[default]
    Horizontal,
    // WHY: Editor on top, preview on bottom.
    Vertical,
}

// WHY: Position of the Table of Contents panel in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TocPosition {
    // WHY: Left side of the workspace.
    #[default]
    Left,
    // WHY: Right side of the workspace.
    Right,
}

// WHY: Pane order within the split view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PaneOrder {
    // WHY: Editor first (left or top), preview second.
    #[default]
    EditorFirst,
    // WHY: Preview first (left or top), editor second.
    PreviewFirst,
}

// WHY: Items that can be placed in the left activity rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityRailItem {
    // WHY: Toggle workspace tree pane
    WorkspaceToggle,
    // WHY: Open search modal
    Search,
    // WHY: Open history menu / workspace switcher
    History,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    #[serde(default)]
    pub split_direction: SplitDirection,
    #[serde(default)]
    pub pane_order: PaneOrder,
    #[serde(default = "super::super::defaults::default_true")]
    pub toc_visible: bool,
    #[serde(default)]
    pub toc_position: TocPosition,
    #[serde(default = "default_activity_rail_order")]
    pub activity_rail_order: Vec<ActivityRailItem>,
}

fn default_activity_rail_order() -> Vec<ActivityRailItem> {
    vec![
        ActivityRailItem::History,
        ActivityRailItem::WorkspaceToggle,
        ActivityRailItem::Search,
    ]
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            split_direction: Default::default(),
            pane_order: Default::default(),
            toc_visible: true,
            toc_position: Default::default(),
            activity_rail_order: default_activity_rail_order(),
        }
    }
}
