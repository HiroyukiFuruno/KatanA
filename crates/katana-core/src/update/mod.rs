pub mod cleanup;
pub mod download;
pub mod installer;
pub mod scripts;
pub mod types;
pub mod version;

pub use types::*;

/// Default interval between automatic update checks (6 hours).
const DEFAULT_UPDATE_CHECK_INTERVAL_SECS: u64 = 6 * 60 * 60;

impl UpdateManager {
    pub fn new(current_version: String, app_path: std::path::PathBuf) -> Self {
        Self {
            current_version,
            api_url_override: None,
            target_app_path: app_path,
            state: UpdateState::Idle,
            last_checked: None,
            check_interval: std::time::Duration::from_secs(DEFAULT_UPDATE_CHECK_INTERVAL_SECS),
        }
    }

    pub fn should_check_for_updates(&self) -> bool {
        match self.last_checked {
            None => true,
            Some(last) => last.elapsed() >= self.check_interval,
        }
    }

    pub fn set_api_url_override(&mut self, url: String) {
        self.api_url_override = Some(url);
    }

    pub fn set_check_interval(&mut self, interval: std::time::Duration) {
        self.check_interval = interval;
    }

    pub fn transition_to(&mut self, state: UpdateState) {
        self.state = state;
        self.last_checked = Some(std::time::Instant::now());
    }
}

#[cfg(test)]
mod tests;
