use super::types::UpdateCleanupOps;

#[cfg(target_os = "macos")]
impl UpdateCleanupOps {
    #[cfg(not(coverage))]
    pub fn perform_background_cleanup() {
        /* WHY: Untap the Homebrew tap on every macOS startup so that brew can no
         * longer push automatic updates. The app itself stays installed and running —
         * only the tap (update source) is removed. Future updates come through the
         * in-app updater instead. Errors are suppressed with `|| true`. */
        std::thread::spawn(|| {
            let script = r#"export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"
if command -v brew > /dev/null 2>&1; then
    brew unpin katana --force > /dev/null 2>&1 || true
    brew unpin katana-desktop --force > /dev/null 2>&1 || true
    brew untap hiroyukifuruno/katana > /dev/null 2>&1 || true
    brew untap HiroyukiFuruno/katana > /dev/null 2>&1 || true
fi
"#;
            let mut command = crate::system::ProcessService::create_command("/bin/bash");
            command
                .arg("-c")
                .arg(script)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());

            if let Ok(status) = crate::system::ProcessService::status(command)
                && !status.success()
            {
                tracing::warn!("brew untap script exited with status {:?}", status.code());
            }
        });
    }

    #[cfg(coverage)]
    pub fn perform_background_cleanup() {
        /* WHY: Coverage builds exclude the spawned thread to prevent false negatives. */
    }
}

#[cfg(not(target_os = "macos"))]
impl UpdateCleanupOps {
    pub fn perform_background_cleanup() {
        /* WHY: brew cleanup is macOS-only. No-op on other platforms. */
    }
}
