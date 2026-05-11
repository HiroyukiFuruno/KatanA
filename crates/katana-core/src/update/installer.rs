use super::types::{UpdateDownloadOps, UpdateInstallerOps, UpdatePreparation};
use crate::update::UpdateProgress;
#[cfg(target_os = "macos")]
use std::ffi::OsStr;
use std::path::Path;

#[cfg(any(target_os = "windows", target_os = "linux", test))]
mod extracted_file;
#[cfg(test)]
#[cfg(unix)]
mod tests;

#[cfg(target_os = "macos")]
const MACOS_APP_BUNDLE_NAME: &str = "KatanA Desktop.app";

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
                .unwrap_or_else(|| OsStr::new(MACOS_APP_BUNDLE_NAME));
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
            let extracted_app_path = extracted_file::resolve_extracted_file(
                &extract_dir,
                extracted_file::WINDOWS_EXECUTABLE_NAME,
                extracted_file::ExtractedFileFallback::RegularFile,
            )?;

            let script_path = temp_dir.path().join("relauncher.ps1");
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
            let extracted_app_path = extracted_file::resolve_extracted_file(
                &extract_dir,
                extracted_file::LINUX_EXECUTABLE_NAME,
                extracted_file::ExtractedFileFallback::ExecutableRegularFile,
            )?;

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
                    prep.script_path.to_str().unwrap_or(""),
                    &parent_pid,
                ])
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
