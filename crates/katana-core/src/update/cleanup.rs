use super::types::UpdateCleanupOps;

#[cfg(all(target_os = "macos", not(coverage)))]
const CONFIG_DIR_ENV: &str = "KATANA_CONFIG_DIR";

#[cfg(all(target_os = "macos", any(not(coverage), test)))]
fn should_perform_background_cleanup(configured: Option<&std::ffi::OsStr>) -> bool {
    /* WHY: 隔離プロファイルの検査では利用者の Homebrew 設定を変更しない。 */
    configured.is_none_or(std::ffi::OsStr::is_empty)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::should_perform_background_cleanup;
    use std::ffi::OsStr;

    #[test]
    fn configured_profile_skips_global_homebrew_cleanup() {
        assert!(!should_perform_background_cleanup(Some(OsStr::new(
            "/tmp/katana-isolated"
        ))));
    }

    #[test]
    fn absent_or_empty_config_keeps_normal_cleanup() {
        assert!(should_perform_background_cleanup(None));
        assert!(should_perform_background_cleanup(Some(OsStr::new(""))));
    }
}

#[cfg(target_os = "macos")]
impl UpdateCleanupOps {
    #[cfg(not(coverage))]
    pub fn perform_background_cleanup() {
        if !should_perform_background_cleanup(std::env::var_os(CONFIG_DIR_ENV).as_deref()) {
            return;
        }

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
