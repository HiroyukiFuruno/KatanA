/// HTTP download logic for updates.
pub mod download;
/// DMG extraction and relaunch script logic.
pub mod installer;
/// Version comparison and update checks.
pub mod version;

pub use download::*;
pub use installer::*;
pub use version::*;

/// Represents the progress of a download or extraction operation.
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateProgress {
    /// In progress downloading.
    Downloading {
        /// Bytes downloaded so far.
        downloaded: u64,
        /// Total bytes expected if known.
        total: Option<u64>,
    },
    /// In progress extracting.
    Extracting {
        /// Current file index.
        current: usize,
        /// Total files in the archive.
        total: usize,
    },
}

/// The state of the update manager.
#[derive(Debug, Default)]
pub enum UpdateState {
    /// Initial idle state.
    #[default]
    Idle,
    /// Currently checking for updates.
    Checking,
    /// An update is available holding the release info.
    UpdateAvailable(ReleaseInfo),
    /// Currently downloading an update.
    Downloading,
    /// Downloaded and prepared, ready to restart the application.
    ReadyToRestart(UpdatePreparation),
    /// Emits a string-based error message.
    Error(String),
}

/// The central manager handling the update lifecycle.
pub struct UpdateManager {
    /// The current application version string.
    pub current_version: String,
    /// An optional API URL override for checking updates (useful for testing).
    pub api_url_override: Option<String>,
    /// The internal path of the target application bundle to update.
    pub target_app_path: std::path::PathBuf,
    /// The current operational state of the manager.
    pub state: UpdateState,
    /// When the update was last checked.
    pub last_checked: Option<std::time::Instant>,
    /// The interval duration between automatic checks.
    pub check_interval: std::time::Duration,
}

impl UpdateManager {
    /// Creates a new `UpdateManager` with the specified version and target application path.
    pub fn new(current_version: String, target_app_path: std::path::PathBuf) -> Self {
        const DEFAULT_CHECK_INTERVAL_SECS: u64 = 86_400;
        Self {
            current_version,
            api_url_override: None,
            target_app_path,
            state: UpdateState::Idle,
            last_checked: None,
            check_interval: std::time::Duration::from_secs(DEFAULT_CHECK_INTERVAL_SECS),
        }
    }

    /// Returns true if it is time to check for updates again based on the interval.
    pub fn should_check_for_updates(&self) -> bool {
        match self.last_checked {
            Some(last) => last.elapsed() >= self.check_interval,
            None => true,
        }
    }

    /// Overrides the URL to check for updates.
    pub fn set_api_url_override(&mut self, url: String) {
        self.api_url_override = Some(url);
    }

    /// Sets the interval at which to automatically check for updates.
    pub fn set_check_interval(&mut self, interval: std::time::Duration) {
        self.check_interval = interval;
    }

    /// Transitions the manager to a new state, updating 'last_checked' if transitioned to Checking.
    pub fn transition_to(&mut self, new_state: UpdateState) {
        if matches!(new_state, UpdateState::Checking) {
            self.last_checked = Some(std::time::Instant::now());
        }
        self.state = new_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_manager_and_state() {
        let target = std::path::PathBuf::from("/Applications/KatanA.app");
        let mut manager = UpdateManager::new("0.6.4".to_string(), target.clone());

        assert_eq!(manager.current_version, "0.6.4");
        assert_eq!(manager.target_app_path, target);
        assert!(matches!(manager.state, UpdateState::Idle));

        assert!(manager.should_check_for_updates());

        manager.set_api_url_override("http://localhost".to_string());
        assert_eq!(
            manager.api_url_override.as_deref(),
            Some("http://localhost")
        );

        manager.set_check_interval(std::time::Duration::from_secs(3600));
        assert_eq!(manager.check_interval, std::time::Duration::from_secs(3600));

        manager.transition_to(UpdateState::Checking);
        assert!(matches!(manager.state, UpdateState::Checking));
        assert!(manager.last_checked.is_some());

        assert!(!manager.should_check_for_updates());

        manager.last_checked =
            Some(std::time::Instant::now() - std::time::Duration::from_secs(4000));
        assert!(manager.should_check_for_updates());

        manager.transition_to(UpdateState::Error("dummy error".to_string()));
        assert!(matches!(manager.state, UpdateState::Error(_)));

        let default_state = UpdateState::default();
        assert!(matches!(default_state, UpdateState::Idle));
    }
}
