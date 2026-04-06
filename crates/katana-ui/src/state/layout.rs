use crate::app_state::StatusType;
use std::path::PathBuf;

pub struct LayoutState {
    pub status_message: Option<(String, StatusType)>,
    pub show_workspace: bool,
    pub show_settings: bool,
    pub show_toc: bool,
    pub show_search_modal: bool,
    pub show_workspace_history_modal: bool,
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
}

impl Default for LayoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutState {
    pub fn new() -> Self {
        Self {
            status_message: None,
            show_workspace: true,
            show_settings: false,
            show_toc: false,
            show_search_modal: false,
            show_workspace_history_modal: false,
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
        }
    }
}
