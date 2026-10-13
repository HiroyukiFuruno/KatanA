use crate::shell::KatanaApp;

pub(crate) struct UpdateHelpers;

impl UpdateHelpers {
    pub(crate) fn remember_update_check_timestamp(state: &mut KatanaApp) {
        let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => return,
        };
        state
            .state
            .config
            .settings
            .settings_mut()
            .updates
            .last_checked_timestamp_sec = Some(now);
        let _ = state.state.config.try_save_settings();
    }

    pub(crate) fn compute_update_phase(
        prog: katana_core::update::UpdateProgress,
    ) -> crate::app_state::UpdatePhase {
        match prog {
            katana_core::update::UpdateProgress::Downloading { downloaded, total } => {
                let progress = Self::compute_download_progress(downloaded, total);
                crate::app_state::UpdatePhase::Downloading { progress }
            }
            katana_core::update::UpdateProgress::Extracting { current, total } => {
                let progress = if total > 0 {
                    current as f32 / total as f32
                } else {
                    0.0
                };
                crate::app_state::UpdatePhase::Installing { progress }
            }
        }
    }

    fn compute_download_progress(downloaded: u64, total: Option<u64>) -> f32 {
        let Some(t) = total else { return 0.0 };
        if t > 0 {
            downloaded as f32 / t as f32
        } else {
            0.0
        }
    }
}
