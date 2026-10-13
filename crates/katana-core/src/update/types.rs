use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub html_url: String,
    pub body: String,
    pub download_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateProgress {
    Downloading { downloaded: u64, total: Option<u64> },
    Extracting { current: usize, total: usize },
}

#[derive(Debug, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpdateAvailable(ReleaseInfo),
    Downloading,
    ReadyToRestart(UpdatePreparation),
    Error(String),
}

#[derive(Debug)]
pub struct UpdatePreparation {
    pub temp_dir: tempfile::TempDir,
    pub app_bundle_path: PathBuf,
    pub script_path: PathBuf,
}

pub struct UpdateManager {
    pub current_version: String,
    pub api_url_override: Option<String>,
    pub target_app_path: PathBuf,
    pub state: UpdateState,
    pub last_checked: Option<std::time::Instant>,
    pub check_interval: std::time::Duration,
}

pub struct UpdateDownloadOps;
pub struct UpdateInstallerOps;
pub struct UpdateCleanupOps;
pub struct UpdateOps;
