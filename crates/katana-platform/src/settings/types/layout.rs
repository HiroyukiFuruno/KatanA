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
}
