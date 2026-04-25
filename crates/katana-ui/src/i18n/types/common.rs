use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AboutMessages {
    pub basic_info: String,
    pub version: String,
    pub build: String,
    pub copyright: String,
    pub runtime: String,
    pub platform: String,
    pub architecture: String,
    pub rust: String,
    pub license: String,
    pub links: String,
    pub source_code: String,
    pub documentation: String,
    pub report_issue: String,
    pub support: String,
    pub sponsor: String,
    #[serde(default = "default_about_website")]
    pub official_website: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessages {
    pub title: String,
    pub checking_for_updates: String,
    pub update_available: String,
    pub update_available_desc: String,
    pub release_notes_template: String,
    pub up_to_date: String,
    pub up_to_date_desc: String,
    pub failed_to_check: String,
    pub action_close: String,
    pub install_update: String,
    pub downloading: String,
    pub installing: String,
    pub restart_confirm: String,
    pub action_later: String,
    pub action_skip_version: String,
    pub action_restart: String,
    pub download_update: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessages {
    pub ai_unconfigured: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMessages {
    pub not_installed: String,
    pub install_path: String,
    pub download: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocMessages {
    pub title: String,
    pub empty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMessages {
    pub success: String,
    pub failed: String,
    pub tool_missing: String,
    pub temp_file_error: String,
    pub write_error: String,
    pub persist_error: String,
    pub exporting: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorToolbarMessages {
    pub heading1: String,
    pub heading2: String,
    pub heading3: String,
    pub bullet_list: String,
    pub numbered_list: String,
    pub blockquote: String,
    pub code_block: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorMessages {
    pub toolbar: EditorToolbarMessages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsMessages {
    pub title: String,
    pub version_label: String,
    pub content: String,
    pub accept: String,
    pub decline: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogMessages {
    pub new_file_title: String,
    pub new_directory_title: String,
    pub rename_title: String,
    pub delete_title: String,
    pub delete_confirm_msg: String,
    pub name_hint: String,
    pub new_name_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownMessages {
    pub task_todo: String,
    pub task_in_progress: String,
    pub task_done: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonMessages {
    pub close: String,
    pub save: String,
    pub advanced_settings: String,
    #[serde(default = "default_expand_all")]
    pub expand_all: String,
    #[serde(default = "default_collapse_all")]
    pub collapse_all: String,
}

fn default_expand_all() -> String {
    "Expand All".to_string()
}
fn default_collapse_all() -> String {
    "Collapse All".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageEntry {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpMessages {
    pub section_general: String,
    pub section_editor: String,
    pub section_behavior: String,
    pub shortcut_command_palette: String,
    pub shortcut_search: String,
    pub shortcut_sidebar: String,
    pub shortcut_save: String,
    pub shortcut_refresh: String,
    pub shortcut_toggle_split: String,
    pub shortcut_toggle_code_preview: String,
    pub shortcut_tab_next: String,
    pub shortcut_tab_prev: String,
    pub shortcut_toggle_slideshow: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewModeMessages {
    pub preview: String,
    pub code: String,
    pub split: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitToggleMessages {
    pub horizontal: String,
    pub vertical: String,
    pub editor_first: String,
    pub preview_first: String,
}

fn default_about_website() -> String {
    "Official Website".to_string()
}
