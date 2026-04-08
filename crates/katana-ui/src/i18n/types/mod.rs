use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct LanguageEntry {
    pub(crate) code: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct I18nMessages {
    pub menu: MenuMessages,
    pub workspace: WorkspaceMessages,
    pub preview: PreviewMessages,
    pub plantuml: PlantumlMessages,
    pub view_mode: ViewModeMessages,
    pub split_toggle: SplitToggleMessages,
    pub error: ErrorMessages,
    pub status: StatusMessages,
    pub action: ActionMessages,
    pub ai: AiMessages,
    pub tool: ToolMessages,
    pub settings: SettingsMessages,
    pub tab: TabMessages,
    pub search: SearchMessages,
    pub about: AboutMessages,
    pub update: UpdateMessages,
    pub toc: TocMessages,
    #[serde(default)]
    pub export: ExportMessages,
    #[serde(default)]
    pub terms: TermsMessages,
    #[serde(default)]
    pub dialog: DialogMessages,
    #[serde(default)]
    pub markdown: MarkdownMessages,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExportMessages {
    pub success: String,
    pub failed: String,
    pub tool_missing: String,
    pub temp_file_error: String,
    pub write_error: String,
    pub persist_error: String,
    pub exporting: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TermsMessages {
    pub title: String,
    pub version_label: String,
    pub content: String,
    pub accept: String,
    pub decline: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TocMessages {
    pub title: String,
    pub empty: String,
}

mod search;
pub use search::SearchMessages;

mod update;
pub use update::{AboutMessages, UpdateMessages};

mod menu;
pub use menu::MenuMessages;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceMessages {
    pub no_workspace_open: String,
    pub no_document_selected: String,
    #[serde(alias = "workspace_title")]
    pub explorer_title: String,
    pub workspace_history_title: String,
    pub recent_workspaces: String,
    #[serde(default)]
    pub sidebar_workspace_tooltip: String,
    #[serde(default)]
    pub sidebar_history_tooltip: String,
    #[serde(default)]
    pub no_recent_workspaces: String,
    #[serde(default = "default_no_saved_workspaces")]
    pub no_saved_workspaces: String,
    pub open_folder_hint: String,
    pub open_workspace_button: String,
    pub remove_history_tooltip: String,
    #[serde(default = "default_metadata_tooltip")]
    pub metadata_tooltip: String,
    pub path_label: String,
    #[serde(default = "default_flat_view")]
    pub flat_view: String,
    #[serde(default)]
    pub filter_regex_hint: String,
}

fn default_no_saved_workspaces() -> String {
    "No saved workspaces".to_string()
}

fn default_flat_view() -> String {
    "Flat View".to_string()
}

#[cfg(test)]
mod default_flat_view_tests {
    use super::*;
    #[test]
    fn test_default_flat_view() {
        assert_eq!(default_flat_view(), "Flat View".to_string());
    }
}

fn default_metadata_tooltip() -> String {
    "Size: {size} B\nModified: {mod_time}".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagramControllerMessages {
    pub pan_up: String,
    pub pan_down: String,
    pub pan_left: String,
    pub pan_right: String,
    pub zoom_in: String,
    pub zoom_out: String,
    pub reset: String,
    pub fullscreen: String,
    pub close: String,
    #[serde(default = "default_trackpad_help")]
    pub trackpad_help: String,
}

fn default_trackpad_help() -> String {
    "Trackpad: 2-finger pinch to zoom, 1-finger drag to pan".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewMessages {
    pub preview_title: String,
    pub refresh_diagrams: String,
    pub rendering: String,
    pub no_preview: String,
    #[serde(default = "default_diagram_controller")]
    pub diagram_controller: DiagramControllerMessages,
}

fn default_diagram_controller() -> DiagramControllerMessages {
    DiagramControllerMessages {
        pan_up: "Move up".to_string(),
        pan_down: "Move down".to_string(),
        pan_left: "Move left".to_string(),
        pan_right: "Move right".to_string(),
        zoom_in: "Zoom in".to_string(),
        zoom_out: "Zoom out".to_string(),
        reset: "Reset position and size".to_string(),
        fullscreen: "Fullscreen".to_string(),
        close: "Close".to_string(),
        trackpad_help: default_trackpad_help(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlantumlMessages {
    pub downloading_plantuml: String,
    pub plantuml_installed: String,
    pub download_error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewModeMessages {
    pub preview: String,
    pub code: String,
    pub split: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SplitToggleMessages {
    pub horizontal: String,
    pub vertical: String,
    pub editor_first: String,
    pub preview_first: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMessages {
    pub missing_dependency: String,
    pub curl_launch_failed: String,
    pub download_failed: String,
    pub render_error: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusMessages {
    pub ready: String,
    pub saved: String,
    pub save_failed: String,
    pub opened_workspace: String,
    pub cannot_open_workspace: String,
    pub cannot_open_file: String,
    #[serde(default = "default_refresh_success")]
    pub refresh_success: String,
    #[serde(default = "default_refresh_skipped_dirty")]
    pub refresh_skipped_dirty: String,
    #[serde(default = "default_refresh_no_changes")]
    pub refresh_no_changes: String,
    #[serde(default = "default_refresh_failed")]
    pub refresh_failed: String,
    #[serde(default = "default_problems_panel_title")]
    pub problems_panel_title: String,
    #[serde(default = "default_problems_panel_close")]
    pub problems_panel_close: String,
    #[serde(default = "default_no_problems_found")]
    pub no_problems_found: String,
    #[serde(default = "default_toggle_problems_panel")]
    pub toggle_problems_panel: String,
    #[serde(default = "default_problems_count_format")]
    pub problems_count_format: String,
    #[serde(default = "default_removed_workspace")]
    pub removed_workspace: String,
    #[serde(default = "default_error_save_settings")]
    pub error_save_settings: String,
    #[serde(default = "default_closed_workspace")]
    pub closed_workspace: String,
}

fn default_closed_workspace() -> String {
    "Workspace closed.".to_string()
}

fn default_refresh_success() -> String {
    "Document refreshed from disk.".to_string()
}
fn default_refresh_skipped_dirty() -> String {
    "Refresh skipped to preserve unsaved changes.".to_string()
}
fn default_refresh_no_changes() -> String {
    "No external changes detected.".to_string()
}
fn default_refresh_failed() -> String {
    "Failed to refresh document: {error}".to_string()
}
fn default_problems_panel_title() -> String {
    "Problems".to_string()
}
fn default_problems_panel_close() -> String {
    "Close".to_string()
}
fn default_no_problems_found() -> String {
    "No problems found in the workspace.".to_string()
}
fn default_toggle_problems_panel() -> String {
    "Toggle Problems Panel".to_string()
}
fn default_problems_count_format() -> String {
    "Problems: {count}".to_string()
}
fn default_removed_workspace() -> String {
    "Workspace '{path}' removed.".to_string()
}
fn default_error_save_settings() -> String {
    "Failed to save settings.".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ActionMessages {
    pub expand_all: String,
    pub collapse_all: String,
    pub collapse_sidebar: String,
    #[serde(alias = "refresh_workspace")]
    pub refresh_explorer: String,
    pub toggle_filter: String,
    pub remove_workspace: String,
    #[serde(default)]
    pub recursive_expand: String,
    #[serde(default)]
    pub recursive_open_all: String,
    #[serde(default)]
    pub toggle_toc: String,
    #[serde(default = "default_action_toggle_slideshow")]
    pub toggle_slideshow: String,
    pub show_meta_info: String,
    #[serde(default = "default_action_new_file")]
    pub new_file: String,
    #[serde(default = "default_action_new_directory")]
    pub new_directory: String,
    #[serde(default = "default_action_open")]
    pub open: String,
    #[serde(default = "default_action_rename")]
    pub rename: String,
    #[serde(default = "default_action_delete")]
    pub delete: String,
    #[serde(default = "default_action_copy_path")]
    pub copy_path: String,
    #[serde(default = "default_action_copy_relative_path")]
    pub copy_relative_path: String,
    #[serde(default = "default_action_reveal_in_os")]
    pub reveal_in_os: String,
    #[serde(default = "default_action_save")]
    pub save: String,
    #[serde(default = "default_action_refresh_document")]
    pub refresh_document: String,
    #[serde(default = "default_action_cancel")]
    pub cancel: String,
    #[serde(default = "default_action_discard")]
    pub discard: String,
    #[serde(default = "default_action_confirm")]
    pub confirm: String,
}

fn default_action_confirm() -> String {
    "Confirm".to_string()
}

fn default_action_toggle_slideshow() -> String {
    "Slideshow".to_string()
}

fn default_action_new_file() -> String {
    "New File".to_string()
}
fn default_action_new_directory() -> String {
    "New Folder".to_string()
}
fn default_action_open() -> String {
    "Open".to_string()
}
fn default_action_rename() -> String {
    "Rename".to_string()
}
fn default_action_delete() -> String {
    "Delete".to_string()
}
fn default_action_copy_path() -> String {
    "Copy Path".to_string()
}
fn default_action_copy_relative_path() -> String {
    "Copy Relative Path".to_string()
}
fn default_action_reveal_in_os() -> String {
    "Reveal in OS".to_string()
}
fn default_action_save() -> String {
    "Save".to_string()
}
fn default_action_refresh_document() -> String {
    "Refresh Document".to_string()
}
fn default_action_cancel() -> String {
    "Cancel".to_string()
}
fn default_action_discard() -> String {
    "Discard".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct DialogMessages {
    pub new_file_title: String,
    pub new_directory_title: String,
    pub rename_title: String,
    pub delete_title: String,
    pub delete_confirm_msg: String,
    #[serde(default = "default_unsaved_changes_title")]
    pub unsaved_changes_title: String,
    #[serde(default = "default_unsaved_changes_msg")]
    pub unsaved_changes_msg: String,
    #[serde(default)]
    pub name_hint: String,
    #[serde(default)]
    pub new_name_hint: String,
}

fn default_unsaved_changes_title() -> String {
    "Unsaved Changes".to_string()
}
fn default_unsaved_changes_msg() -> String {
    "Do you want to save the changes you made to {name}?\n\nYour changes will be lost if you don't save them.".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiMessages {
    pub ai_unconfigured: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolMessages {
    pub not_installed: String,
    pub install_path: String,
    pub download: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsMessages {
    pub title: String,
    pub tabs: Vec<SettingsTabMessage>,
    pub toc_visible: String,
    pub theme: SettingsThemeMessages,
    pub font: SettingsFontMessages,
    pub layout: SettingsLayoutMessages,
    pub workspace: SettingsWorkspaceMessages,
    #[serde(default)]
    pub updates: SettingsUpdatesMessages,
    #[serde(default)]
    pub behavior: SettingsBehaviorMessages,
    pub preview: SettingsPreviewMessages,
    pub color: SettingsColorMessages,
    #[serde(default)]
    pub icons: SettingsIconsMessages,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SettingsIconsMessages {
    #[serde(default = "default_icons_preset_label")]
    pub preset_label: String,
    #[serde(default = "default_icons_custom_preset")]
    pub custom_preset: String,
    #[serde(default = "default_icons_save_preset")]
    pub save_preset: String,
    #[serde(default = "default_icons_revert_default")]
    pub revert_default: String,
    #[serde(default = "default_icons_advanced_settings")]
    pub advanced_settings: String,
    #[serde(default = "default_colorful_vendor_icons_label")]
    pub colorful_vendor_icons_label: String,
    #[serde(default = "default_table_header_icon")]
    pub table_header_icon: String,
    #[serde(default = "default_table_header_vendor")]
    pub table_header_vendor: String,
    #[serde(default = "default_table_header_color")]
    pub table_header_color: String,
    #[serde(default = "default_table_header_border")]
    pub table_header_border: String,
    #[serde(default = "default_table_header_preview")]
    pub table_header_preview: String,
    #[serde(default = "default_icons_preset_name")]
    pub preset_name: String,
}

fn default_icons_preset_name() -> String {
    "Preset Name:".to_string()
}

fn default_colorful_vendor_icons_label() -> String {
    "Apply default colours to non-Katana icons".to_string()
}
fn default_table_header_icon() -> String {
    "Icon".to_string()
}
fn default_table_header_vendor() -> String {
    "Vendor".to_string()
}
fn default_table_header_color() -> String {
    "Text Color".to_string()
}
fn default_table_header_border() -> String {
    "Frame Color".to_string()
}
fn default_table_header_preview() -> String {
    "Preview".to_string()
}

fn default_icons_preset_label() -> String {
    "Preset:".to_string()
}
fn default_icons_custom_preset() -> String {
    "Custom".to_string()
}
fn default_icons_save_preset() -> String {
    "Save Preset As...".to_string()
}
fn default_icons_revert_default() -> String {
    "Revert to Default".to_string()
}
fn default_icons_advanced_settings() -> String {
    "Advanced Settings".to_string()
}

impl SettingsMessages {
    pub fn tab_name(&self, key: &str) -> String {
        self.tabs
            .iter()
            .find(|t| t.key == key)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| key.to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsThemeMessages {
    pub preset: String,
    pub dark_section: String,
    pub light_section: String,
    #[serde(default = "default_icon_pack_msg")]
    pub icon_pack: String,
    pub custom_colors: String,
    pub reset_custom: String,
    #[serde(default = "default_custom_section")]
    pub custom_section: String,
    #[serde(default = "default_delete_custom")]
    pub delete_custom: String,
    #[serde(default = "default_save_custom_theme")]
    pub save_custom_theme: String,
    #[serde(default = "default_save_custom_theme_title")]
    pub save_custom_theme_title: String,
    #[serde(default = "default_theme_name_label")]
    pub theme_name_label: String,
    #[serde(default = "default_duplicate")]
    pub duplicate: String,
    #[serde(default = "default_ui_contrast_offset")]
    pub ui_contrast_offset: String,
    #[serde(default = "default_show_more")]
    pub show_more: String,
    #[serde(default = "default_show_less")]
    pub show_less: String,
}

fn default_show_more() -> String {
    "Show more...".to_string()
}

fn default_icon_pack_msg() -> String {
    "Icon Pack".to_string()
}

fn default_show_less() -> String {
    "Show less...".to_string()
}

fn default_ui_contrast_offset() -> String {
    "UI Contrast Offset".to_string()
}

fn default_duplicate() -> String {
    "Duplicate...".to_string()
}

fn default_custom_section() -> String {
    "Custom".to_string()
}

fn default_delete_custom() -> String {
    "Delete Custom Theme".to_string()
}

fn default_save_custom_theme() -> String {
    "Save as Custom Theme...".to_string()
}

fn default_save_custom_theme_title() -> String {
    "Save Custom Theme".to_string()
}

fn default_theme_name_label() -> String {
    "Theme Name:".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsTabMessage {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsFontMessages {
    pub size: String,
    pub family: String,
    pub size_slider_hint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsLayoutMessages {
    pub split_direction: String,
    pub horizontal: String,
    pub vertical: String,
    pub pane_order: String,
    pub editor_first: String,
    pub preview_first: String,
    pub toc_position: String,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsWorkspaceMessages {
    pub max_depth: String,
    pub ignored_directories: String,
    pub ignored_directories_hint: String,
    #[serde(default = "default_visible_extensions_msg")]
    pub visible_extensions: String,
    #[serde(default = "default_no_extension_label")]
    pub no_extension_label: String,
    #[serde(default = "default_no_extension_warning_title")]
    pub no_extension_warning_title: String,
    #[serde(default = "default_no_extension_warning")]
    pub no_extension_warning: String,
    #[serde(default = "default_extensionless_excludes")]
    pub extensionless_excludes: String,
    #[serde(default = "default_extensionless_excludes_hint")]
    pub extensionless_excludes_hint: String,
}

fn default_extensionless_excludes() -> String {
    "Ignored Extensionless Files".to_string()
}
fn default_extensionless_excludes_hint() -> String {
    "Comma-separated list of exact file names to ignore when 'No Extension' is enabled (e.g., .DS_Store, .gitignore).".to_string()
}

fn default_no_extension_label() -> String {
    "No Extension".to_string()
}
fn default_no_extension_warning_title() -> String {
    "Warning".to_string()
}
fn default_no_extension_warning() -> String {
    "There is no guarantee that files without extensions can be displayed correctly as Markdown. Furthermore, the application may crash due to unexpected behavior. Are you sure you want to enable this?".to_string()
}

fn default_visible_extensions_msg() -> String {
    "Visible Extensions".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SettingsUpdatesMessages {
    pub section_title: String,
    pub interval: String,
    pub never: String,
    pub daily: String,
    pub weekly: String,
    pub monthly: String,
    pub check_now: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SettingsBehaviorMessages {
    pub section_title: String,
    pub confirm_close_dirty_tab: String,
    pub scroll_sync: String,
    pub auto_save: String,
    pub auto_save_interval: String,
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: String,
    #[serde(default = "default_auto_refresh_interval")]
    pub auto_refresh_interval: String,
    pub seconds: String,
    pub close_confirm_title: String,
    pub close_confirm_msg: String,
    pub close_confirm_discard: String,
    pub close_confirm_cancel: String,

    #[serde(default = "default_clear_http_cache")]
    pub clear_http_cache: String,

    #[serde(default = "default_cache_retention_days")]
    pub cache_retention_days: String,

    #[serde(default = "default_days_suffix")]
    pub days_suffix: String,
}

fn default_clear_http_cache() -> String {
    "Clear All Caches".to_string()
}
fn default_cache_retention_days() -> String {
    "Cache Retention Days".to_string()
}
fn default_days_suffix() -> String {
    " days".to_string()
}
fn default_auto_refresh() -> String {
    "Auto-refresh".to_string()
}
fn default_auto_refresh_interval() -> String {
    "Auto-refresh interval".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsPreviewMessages {
    pub title: String,
    pub heading: String,
    pub normal_text: String,
    pub accent_link: String,
    pub secondary_text: String,
    pub code_sample: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SettingsColorMessages {
    pub background: String,
    pub panel_background: String,
    pub text: String,
    pub text_secondary: String,
    pub accent: String,
    pub border: String,
    pub selection: String,
    pub code_background: String,
    pub preview_background: String,
    #[serde(default = "default_section_system")]
    pub section_system: String,
    #[serde(default = "default_section_code")]
    pub section_code: String,
    #[serde(default = "default_section_preview")]
    pub section_preview: String,
    #[serde(default = "default_group_basic")]
    pub group_basic: String,
    #[serde(default = "default_group_text")]
    pub group_text: String,
    #[serde(default = "default_group_ui_elements")]
    pub group_ui_elements: String,
    #[serde(default = "default_highlight")]
    pub highlight: String,
    #[serde(default = "default_code_text")]
    pub code_text: String,
    #[serde(default = "default_preview_text")]
    pub preview_text: String,

    #[serde(default = "d_title_bar_text")]
    pub title_bar_text: String,

    #[serde(default = "d_active_file_highlight")]
    pub active_file_highlight: String,
    #[serde(default = "d_file_tree_text")]
    pub file_tree_text: String,

    #[serde(default = "d_success_text")]
    pub success_text: String,

    #[serde(default = "d_warning_text")]
    pub warning_text: String,

    #[serde(default = "d_error_text")]
    pub error_text: String,

    #[serde(default = "d_button_bg")]
    pub button_background: String,

    #[serde(default = "d_button_active")]
    pub button_active_background: String,

    #[serde(default = "d_splash_bg")]
    pub splash_background: String,

    #[serde(default = "d_splash_prog")]
    pub splash_progress: String,

    #[serde(default = "d_line_num")]
    pub line_number_text: String,

    #[serde(default = "d_line_num_act")]
    pub line_number_active_text: String,

    #[serde(default = "d_curr_bg")]
    pub current_line_background: String,

    #[serde(default = "d_hover_bg")]
    pub hover_line_background: String,

    #[serde(default = "d_search_match")]
    pub search_match: String,

    #[serde(default = "d_search_active")]
    pub search_active: String,
}

fn d_search_match() -> String {
    "Search Match".to_string()
}

fn d_search_active() -> String {
    "Search Active".to_string()
}

fn default_highlight() -> String {
    "Highlight".to_string()
}

fn default_section_system() -> String {
    "System".to_string()
}
fn default_section_code() -> String {
    "Code".to_string()
}
fn default_section_preview() -> String {
    "Preview".to_string()
}
fn default_group_basic() -> String {
    "Basic".to_string()
}
fn default_group_text() -> String {
    "Text & Typography".to_string()
}
fn default_group_ui_elements() -> String {
    "UI Elements".to_string()
}
fn default_code_text() -> String {
    "Code Text".to_string()
}
fn default_preview_text() -> String {
    "Preview Text".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct TabMessages {
    pub nav_prev: String,
    pub nav_next: String,
    #[serde(default)]
    pub close: String,
    #[serde(default)]
    pub close_others: String,
    #[serde(default)]
    pub close_all: String,
    #[serde(default)]
    pub close_right: String,
    #[serde(default)]
    pub close_left: String,
    #[serde(default)]
    pub pin: String,
    #[serde(default)]
    pub unpin: String,
    #[serde(default)]
    pub restore_closed: String,
    #[serde(default = "default_tab_group")]
    pub tab_group: String,
    #[serde(default = "default_new_group")]
    pub new_group: String,
    #[serde(default = "default_create_new_group")]
    pub create_new_group: String,
    #[serde(default = "default_add_to_group")]
    pub add_to_group: String,
    #[serde(default = "default_added_to_group")]
    pub added_to_group: String,
    #[serde(default = "default_remove_from_group")]
    pub remove_from_group: String,
    #[serde(default = "default_rename_group")]
    pub rename_group: String,
    #[serde(default = "default_group_name_placeholder")]
    pub group_name_placeholder: String,
    #[serde(default = "default_create_group_button")]
    pub create_group_button: String,
    /* WHY: Note: add_tab_to_group replaces create_new_group from English, acting as the parent */
    #[serde(default = "default_add_tab_to_group")]
    pub add_tab_to_group: String,
    #[serde(default = "default_close_group")]
    pub close_group: String,
    #[serde(default = "default_ungroup")]
    pub ungroup: String,
}

fn default_tab_group() -> String {
    "Tab Group".to_string()
}
fn default_new_group() -> String {
    "New Group".to_string()
}
fn default_create_new_group() -> String {
    "Create New Group".to_string()
}
fn default_add_to_group() -> String {
    "Add to '{group_name}'".to_string()
}
fn default_added_to_group() -> String {
    "✓ Add to '{group_name}'".to_string()
}
fn default_remove_from_group() -> String {
    "Remove from Group".to_string()
}
fn default_rename_group() -> String {
    "Rename Group".to_string()
}
fn default_group_name_placeholder() -> String {
    "Group name".to_string()
}
fn default_create_group_button() -> String {
    "Create".to_string()
}
fn default_add_tab_to_group() -> String {
    "Add tab to group".to_string()
}
fn default_close_group() -> String {
    "Close group".to_string()
}
fn default_ungroup() -> String {
    "Ungroup".to_string()
}
fn d_title_bar_text() -> String {
    "Title Bar Text".to_string()
}
fn d_active_file_highlight() -> String {
    "Active File".to_string()
}
fn d_success_text() -> String {
    "Success Text".to_string()
}
fn d_warning_text() -> String {
    "Warning Text".to_string()
}
fn d_error_text() -> String {
    "Error Text".to_string()
}
fn d_button_bg() -> String {
    "Button Background".to_string()
}
fn d_button_active() -> String {
    "Active Button".to_string()
}
fn d_splash_bg() -> String {
    "Splash Background".to_string()
}
fn d_splash_prog() -> String {
    "Splash Progress".to_string()
}
fn d_line_num() -> String {
    "Line Number".to_string()
}
fn d_line_num_act() -> String {
    "Active Line Num".to_string()
}
fn d_curr_bg() -> String {
    "Current Line".to_string()
}
fn d_hover_bg() -> String {
    "Hover Line".to_string()
}
fn d_file_tree_text() -> String {
    "File Tree Text".to_string()
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MarkdownMessages {
    #[serde(default = "default_markdown_task_todo")]
    pub task_todo: String,
    #[serde(default = "default_markdown_task_in_progress")]
    pub task_in_progress: String,
    #[serde(default = "default_markdown_task_done")]
    pub task_done: String,
}

fn default_markdown_task_todo() -> String {
    "Todo [ ]".to_string()
}

fn default_markdown_task_in_progress() -> String {
    "In Progress [/]".to_string()
}

fn default_markdown_task_done() -> String {
    "Done [x]".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_task_defaults_are_non_empty() {
        assert!(!default_markdown_task_todo().is_empty());
        assert!(!default_markdown_task_in_progress().is_empty());
        assert!(!default_markdown_task_done().is_empty());
    }

    #[test]
    fn test_tab_group_defaults() {
        assert_eq!(default_create_new_group(), "Create New Group");
        assert_eq!(default_add_to_group(), "Add to '{group_name}'");
        assert_eq!(default_remove_from_group(), "Remove from Group");
        assert_eq!(default_group_name_placeholder(), "Group name");
        assert_eq!(default_create_group_button(), "Create");
        assert_eq!(default_add_tab_to_group(), "Add tab to group");
        assert_eq!(default_close_group(), "Close group");
        assert_eq!(default_ungroup(), "Ungroup");
    }

    #[test]
    fn test_search_defaults() {
        use super::search::*;
        assert_eq!(default_doc_search_title(), "Search Document");
        assert_eq!(default_doc_search_prev(), "Previous");
        assert_eq!(default_doc_search_next(), "Next");
        assert_eq!(default_doc_search_close(), "Close");
        assert_eq!(default_doc_search_count(), "{index}/{total}");
        assert_eq!(d_search_match(), "Search Match");
        assert_eq!(d_search_active(), "Search Active");
        assert_eq!(default_palette_no_results(), "No results found.");
    }
}

#[cfg(test)]
mod default_coverage_tests {
    use super::search::*;
    use super::*;
    #[test]
    fn test_all_defaults() {
        assert_eq!(default_command_type_action(), "Command");
        assert_eq!(default_palette_query_hint(), "Type a command or search...");
        assert_eq!(default_tab_file_name(), "File Name");
        assert_eq!(default_tab_markdown_content(), "Markdown Content");
        assert_eq!(default_md_query_hint(), "Search markdown files...");
        assert_eq!(default_recent_searches(), "Recent Searches");
        assert_eq!(default_clear_history(), "Clear");
        assert_eq!(default_ln_prefix(), "Ln ");
        assert_eq!(default_menu_command_palette(), "Command Palette…");
        assert_eq!(default_menu_view(), "View");
        assert_eq!(default_problems_panel_title(), "Problems");
        assert_eq!(default_problems_panel_close(), "Close");
        assert_eq!(
            default_no_problems_found(),
            "No problems found in the workspace."
        );
        assert_eq!(default_toggle_problems_panel(), "Toggle Problems Panel");
        assert_eq!(default_problems_count_format(), "Problems: {count}");
        assert_eq!(default_command_settings(), "Toggle Settings");
        assert_eq!(default_command_explorer(), "Toggle Workspace Panel");
        assert_eq!(default_command_close_all(), "Close All Documents");
        assert_eq!(default_command_refresh_explorer(), "Refresh Workspace");
        assert_eq!(default_command_updates(), "Check for Updates");
        assert_eq!(default_command_about(), "Toggle About");
    }

    #[test]
    fn test_default_action_toggle_slideshow() {
        assert_eq!(super::default_action_toggle_slideshow(), "Slideshow");
    }
}
