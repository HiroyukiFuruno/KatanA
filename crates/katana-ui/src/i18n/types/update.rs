use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateMessages {
    pub title: String,
    pub checking_for_updates: String,
    pub update_available: String,
    pub update_available_desc: String,
    #[serde(default = "default_release_notes_template")]
    pub release_notes_template: String,
    pub up_to_date: String,
    pub up_to_date_desc: String,
    pub failed_to_check: String,
    pub action_close: String,
    #[serde(default = "default_install_update")]
    pub install_update: String,
    #[serde(default = "default_downloading")]
    pub downloading: String,
    #[serde(default = "default_installing")]
    pub installing: String,
    #[serde(default = "default_restart_confirm")]
    pub restart_confirm: String,
    #[serde(default = "default_action_later")]
    pub action_later: String,
    #[serde(default = "default_action_skip_version")]
    pub action_skip_version: String,
    #[serde(default = "default_action_restart")]
    pub action_restart: String,
    #[serde(default = "default_download_update")]
    pub download_update: String,
}

fn default_release_notes_template() -> String {
    "### New version {version} is available\n\nPlease check the [GitHub Releases page]({url}) for detailed changes and release notes.\nClick \"Install and Restart\" to automatically apply the update.".to_string()
}
fn default_install_update() -> String {
    "Install and Relaunch".to_string()
}
fn default_downloading() -> String {
    "Downloading update...".to_string()
}
fn default_installing() -> String {
    "Installing update...".to_string()
}
fn default_restart_confirm() -> String {
    "Update is ready. Restart now?".to_string()
}
fn default_action_later() -> String {
    "Later".to_string()
}
fn default_action_skip_version() -> String {
    "Skip This Version".to_string()
}
fn default_action_restart() -> String {
    "Restart Now".to_string()
}
fn default_download_update() -> String {
    "Download from GitHub".to_string()
}
