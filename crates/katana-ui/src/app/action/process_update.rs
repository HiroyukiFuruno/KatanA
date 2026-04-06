use crate::shell::*;

impl KatanaApp {
    pub(super) fn handle_action_install_update(&mut self) {
        let Some(release) = &self.state.update.available else {
            return;
        };
        self.state.update.checking = true;
        self.state.update.phase =
            Some(crate::app_state::UpdatePhase::Downloading { progress: 0.0 });
        let exe_path = std::env::current_exe().unwrap();
        const MACOS_BUNDLE_LEVELS: usize = 3;
        let target_app_path = if exe_path.to_string_lossy().contains("MacOS") {
            exe_path
                .ancestors()
                .nth(MACOS_BUNDLE_LEVELS)
                .unwrap()
                .to_path_buf()
        } else {
            exe_path.clone()
        };
        let download_url = release.download_url.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.update_install_rx = Some(rx);
        std::thread::spawn(move || {
            let tx_clone = tx.clone();
            let res = katana_core::update::UpdateInstallerOps::prepare_update(
                &download_url,
                &target_app_path,
                move |progress| {
                    let _ = tx_clone.send(UpdateInstallEvent::Progress(progress));
                },
            )
            .map_err(|e| e.to_string());
            let _ = tx.send(UpdateInstallEvent::Finished(res));
        });
    }
}
