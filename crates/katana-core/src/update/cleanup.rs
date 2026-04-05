use super::types::UpdateCleanupOps;

impl UpdateCleanupOps {
    #[cfg(not(coverage))]
    pub fn perform_background_cleanup() {
        let exe_path = match std::env::current_exe() {
            Ok(p) => p,
            Err(_) => return,
        };

        let mut app_path = exe_path.clone();
        let mut is_app_bundle = false;
        while app_path.pop() {
            if let Some(ext) = app_path.extension()
                && ext == "app"
            {
                is_app_bundle = true;
                break;
            }
        }

        if !is_app_bundle {
            return;
        }

        let app_path_str = app_path.to_string_lossy().to_string();
        let app_bak_str = format!("{}.bak", app_path_str);

        std::thread::spawn(move || {
            let script = format!(
                r#"#!/bin/bash
    export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"
    if command -v brew >/dev/null 2>&1; then
        if brew list --cask | grep -q "^katana-desktop$"; then
            echo "Removing KatanA from Homebrew management..."

            # We must prevent Homebrew from deleting the running app
            if [ -d "{app_path}" ]; then
                mv "{app_path}" "{app_bak}"
            fi

            brew uninstall --cask katana-desktop --force >/dev/null 2>&1 || true
            brew untap HiroyukiFuruno/katana --force >/dev/null 2>&1 || true

            # Restore the app
            if [ -d "{app_bak}" ]; then
                mv "{app_bak}" "{app_path}"
            fi
        fi
    fi
    "#,
                app_path = app_path_str,
                app_bak = app_bak_str,
            );
            let _ = std::process::Command::new("bash")
                .arg("-c")
                .arg(&script)
                .status();
        });
    }

    #[cfg(coverage)]
    pub fn perform_background_cleanup() {
        // WHY: Coverage builds exclude the spawned thread to prevent false negatives from unreachable branches.
    }
}
