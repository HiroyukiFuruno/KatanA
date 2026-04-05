use super::types::{UpdateDownloadOps, UpdateInstallerOps, UpdatePreparation};
use crate::update::UpdateProgress;
use std::path::Path;

impl UpdateInstallerOps {
    pub fn prepare_update<F>(
        download_url: &str,
        target_app_path: &Path,
        mut on_progress: F,
    ) -> anyhow::Result<UpdatePreparation>
    where
        F: FnMut(UpdateProgress),
    {
        let temp_dir = tempfile::tempdir()?;
        let zip_path = temp_dir.path().join("update.zip");
        UpdateDownloadOps::download_update(download_url, &zip_path, &mut on_progress)?;

        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir_all(&extract_dir)?;
        UpdateDownloadOps::extract_update(&zip_path, &extract_dir, &mut on_progress)?;

        let app_name = target_app_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("KatanA.app"));
        let extracted_app_path = extract_dir.join(app_name);

        // WHY: Verify the extracted bundle contains Info.plist to guard against corrupted or incomplete downloads.
        if !extracted_app_path.exists() || !extracted_app_path.join("Contents/Info.plist").exists()
        {
            anyhow::bail!("Extracted update does not contain a valid application bundle");
        }

        let script_path = temp_dir.path().join("relauncher.sh");
        Self::generate_relauncher_script(
            &extracted_app_path,
            target_app_path,
            &script_path,
            temp_dir.path(),
        )?;

        Ok(UpdatePreparation {
            temp_dir,
            app_bundle_path: extracted_app_path,
            script_path,
        })
    }

    #[cfg(not(test))]
    #[cfg(not(coverage))]
    pub fn execute_relauncher(prep: UpdatePreparation) -> anyhow::Result<()> {
        #[allow(deprecated)]
        let _temp_path = prep.temp_dir.into_path();

        std::process::Command::new(&prep.script_path).spawn()?;
        std::process::exit(0);
    }

    pub fn generate_relauncher_script(
        extracted_app: &Path,
        target_app: &Path,
        script_path: &Path,
        temp_dir_path: &Path,
    ) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let content = generate_script_content(target_app, extracted_app, temp_dir_path);
        std::fs::write(script_path, content)?;

        const RELAUNCHER_SCRIPT_PERMISSIONS: u32 = 0o755;

        let mut perms = std::fs::metadata(script_path)?.permissions();
        perms.set_mode(RELAUNCHER_SCRIPT_PERMISSIONS);
        std::fs::set_permissions(script_path, perms)?;

        Ok(())
    }
}

fn generate_script_content(
    target_app: &Path,
    extracted_app: &Path,
    temp_dir_path: &Path,
) -> String {
    format!(
        r#"#!/bin/bash
set -e
sleep 1
TARGET_BAK="{target}.bak"
rm -rf "$TARGET_BAK"

if [ -d "{target}" ]; then
    echo "Backing up existing installation..."
    mv "{target}" "$TARGET_BAK"
fi

if ! mv "{extracted}" "{target}"; then
    echo "Swap failed! Rolling back..."
    osascript -e 'display alert "Update Failed" message "Could not complete the application update. The original version has been restored." as critical' || true
    rm -rf "{target}"

    if [ -d "$TARGET_BAK" ]; then
        mv "$TARGET_BAK" "{target}"
    fi

    open "{target}" || true
    rm -rf "{temp_dir}"
    exit 1
fi

xattr -cr "{target}" || true
open "{target}"
rm -rf "$TARGET_BAK"
rm -rf "{temp_dir}"
"#,
        target = target_app.display(),
        extracted = extracted_app.display(),
        temp_dir = temp_dir_path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_relauncher_script() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let extracted_path = temp_dir.path().join("KatanA-extract.app");
        let target_path = temp_dir.path().join("KatanA.app");
        let script_path = temp_dir.path().join("relauncher.sh");

        UpdateInstallerOps::generate_relauncher_script(
            &extracted_path,
            &target_path,
            &script_path,
            temp_dir.path(),
        )
        .unwrap();

        assert!(script_path.exists());

        let content = std::fs::read_to_string(&script_path).unwrap();
        assert!(content.contains(&format!("TARGET_BAK=\"{}.bak\"", target_path.display())));
        assert!(content.contains(&format!("mv \"{}\" \"$TARGET_BAK\"", target_path.display())));
        assert!(content.contains(&format!(
            "if ! mv \"{}\" \"{}\"; then",
            extracted_path.display(),
            target_path.display()
        )));
        assert!(content.contains("display alert \"Update Failed\""));
        assert!(content.contains("Swap failed! Rolling back..."));
        assert!(content.contains(&format!("xattr -cr \"{}\"", target_path.display())));
        assert!(content.contains(&format!("rm -rf \"{}\"", temp_dir.path().display())));

        let perms = std::fs::metadata(&script_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111, "Script must be executable");
    }
}
