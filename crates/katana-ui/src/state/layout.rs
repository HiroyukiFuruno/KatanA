use crate::app_state::StatusType;
use std::path::PathBuf;

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum RailPopup {
    Search,
    History,
    AddWorkspace,
    Help,
}

pub struct LayoutState {
    pub status_message: Option<(String, StatusType)>,
    pub show_workspace_panel: bool,
    pub show_explorer: bool,
    pub show_settings: bool,
    pub show_toc: bool,
    pub show_search_modal: bool,

    pub show_history_panel: bool,
    pub show_export_panel: bool,
    pub show_story_panel: bool,
    pub workspace_toggle_y: f32,
    pub history_toggle_y: f32,
    pub scale_override: f32,
    pub last_window_title: String,
    pub create_fs_node_modal: Option<(PathBuf, String, Option<String>, bool)>,
    pub rename_modal: Option<(PathBuf, String)>,
    pub delete_modal: Option<PathBuf>,
    pub pending_close_confirm: Option<usize>,
    pub rename_tab_group_modal: Option<(usize, String)>,
    pub inline_rename_group: Option<String>,
    pub show_slideshow: bool,
    pub slideshow_page: usize,
    pub was_os_fullscreen_before_slideshow: bool,
    pub slideshow_last_active_time: f64,
    pub slideshow_settings_open: bool,
    pub slideshow_hover_highlight: bool,
    pub slideshow_show_diagram_controls: bool,
    pub active_rail_popup: Option<RailPopup>,
    pub toc_force_open: Option<bool>,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutState {
    const DEFAULT_TOGGLE_Y: f32 = 60.0;

    pub fn new() -> Self {
        Self {
            status_message: None,
            show_workspace_panel: false,
            show_explorer: true,
            show_settings: false,
            show_toc: false,
            show_search_modal: false,

            show_history_panel: false,
            show_export_panel: false,
            show_story_panel: false,
            workspace_toggle_y: Self::DEFAULT_TOGGLE_Y,
            history_toggle_y: Self::DEFAULT_TOGGLE_Y,
            scale_override: 1.0,
            last_window_title: String::new(),
            create_fs_node_modal: None,
            rename_modal: None,
            delete_modal: None,
            pending_close_confirm: None,
            rename_tab_group_modal: None,
            inline_rename_group: None,
            show_slideshow: false,
            slideshow_page: 0,
            was_os_fullscreen_before_slideshow: false,
            slideshow_last_active_time: 0.0,
            slideshow_settings_open: false,
            slideshow_hover_highlight: false,
            slideshow_show_diagram_controls: false,
            active_rail_popup: None,
            toc_force_open: None,
        }
    }
}
