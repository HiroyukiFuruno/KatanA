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
        /* WHY: Determine the archive extension from the URL or fallback to platform standard. */
        let extension = if download_url.ends_with(".tar.gz") {
            "tar.gz"
        } else {
            "zip"
        };
        let archive_path = temp_dir.path().join(format!("update.{}", extension));

        UpdateDownloadOps::download_update(download_url, &archive_path, &mut on_progress)?;

        let extract_dir = temp_dir.path().join("extracted");
        std::fs::create_dir_all(&extract_dir)?;
        UpdateDownloadOps::extract_update(&archive_path, &extract_dir, &mut on_progress)?;

        #[cfg(target_os = "macos")]
        {
            let app_name = target_app_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("KatanA.app"));
            let extracted_app_path = extract_dir.join(app_name);

            /* WHY: Verify the extracted bundle contains Info.plist to guard against corrupted or incomplete downloads. */
            if !extracted_app_path.exists()
                || !extracted_app_path.join("Contents/Info.plist").exists()
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

        #[cfg(target_os = "windows")]
        {
            let app_name = target_app_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("KatanA.exe"));
            let extracted_app_path = extract_dir.join(app_name);

            if !extracted_app_path.exists() {
                anyhow::bail!("Extracted update does not contain a valid executable");
            }

            let script_path = temp_dir.path().join("relauncher.bat");
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

        #[cfg(target_os = "linux")]
        {
            let app_name = target_app_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("KatanA"));
            let extracted_app_path = extract_dir.join(app_name);

            if !extracted_app_path.exists() {
                anyhow::bail!("Extracted update does not contain a valid executable");
            }

            #[cfg(target_os = "windows")]
            let script_name = "relauncher.ps1";
            #[cfg(not(target_os = "windows"))]
            let script_name = "relauncher.sh";

            let script_path = temp_dir.path().join(script_name);
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
    }

    #[cfg(not(test))]
    #[cfg(not(coverage))]
    pub fn execute_relauncher(prep: UpdatePreparation) -> anyhow::Result<()> {
        #[allow(deprecated)]
        let _temp_path = prep.temp_dir.into_path();

        #[cfg(target_os = "windows")]
        {
            let parent_pid = std::process::id().to_string();
            crate::system::ProcessService::create_command("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                ])
                .arg(&prep.script_path)
                .arg(&parent_pid)
                .spawn()?;
        }
        #[cfg(not(target_os = "windows"))]
        {
            crate::system::ProcessService::create_command(prep.script_path.to_str().unwrap_or(""))
                .spawn()?;
        }
        std::process::exit(0);
    }

    pub fn generate_relauncher_script(
        extracted_app: &Path,
        target_app: &Path,
        script_path: &Path,
        temp_dir_path: &Path,
    ) -> anyhow::Result<()> {
        let content = crate::update::scripts::UpdateScriptOps::generate_script_content(
            target_app,
            extracted_app,
            temp_dir_path,
        );
        std::fs::write(script_path, content)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            const RELAUNCHER_SCRIPT_PERMISSIONS: u32 = 0o755;

            let mut perms = std::fs::metadata(script_path)?.permissions();
            perms.set_mode(RELAUNCHER_SCRIPT_PERMISSIONS);
            std::fs::set_permissions(script_path, perms)?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg(unix)]
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

        #[cfg(target_os = "macos")]
        {
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
        }

        #[cfg(target_os = "linux")]
        {
            assert!(content.contains(&format!("TARGET_BAK=\"{}.bak\"", target_path.display())));
            assert!(content.contains(&format!("mv \"{}\" \"$TARGET_BAK\"", target_path.display())));
            /* WHY: New script uses `if mv ... then` (success branch) instead of `if ! mv ... then`
             * (failure branch) to correctly place chmod +x before launching. */
            assert!(content.contains(&format!(
                "if mv \"{}\" \"{}\"",
                extracted_path.display(),
                target_path.display()
            )));
            assert!(content.contains(&format!("chmod +x \"{}\"", target_path.display())));
            assert!(content.contains(&format!("\"{}\" &", target_path.display())));
            assert!(content.contains(&format!("rm -rf \"{}\"", temp_dir.path().display())));
        }

        let perms = std::fs::metadata(&script_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111, "Script must be executable");
    }
}
