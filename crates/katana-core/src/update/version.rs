use super::release_client::{LATEST_RELEASE_API_URL, ReleaseClient};
use super::types::{ReleaseInfo, UpdateManager, UpdateOps};
use anyhow::Result;

impl UpdateOps {
    fn platform_asset_name() -> &'static str {
        #[cfg(target_os = "macos")]
        return "KatanA-macOS.zip";
        #[cfg(target_os = "windows")]
        return "KatanA-windows-x86_64.zip";
        #[cfg(target_os = "linux")]
        return "KatanA-linux-x86_64.tar.gz";
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return "KatanA-macOS.zip";
    }

    fn build_download_url(tag_name: &str) -> String {
        format!(
            "https://github.com/HiroyukiFuruno/KatanA/releases/download/{}/{}",
            tag_name,
            Self::platform_asset_name()
        )
    }

    pub fn check_for_updates(&self, manager: &UpdateManager) -> Result<Option<ReleaseInfo>> {
        let url = manager
            .api_url_override
            .as_deref()
            .unwrap_or(LATEST_RELEASE_API_URL);

        Self::check_for_updates_from_url(&manager.current_version, url)
    }

    pub fn is_newer_version(current: &str, latest: &str) -> bool {
        Self::compare_versions(current, latest) < 0
    }

    pub fn compare_versions(a: &str, b: &str) -> i8 {
        let a = a.trim_start_matches('v');
        let b = b.trim_start_matches('v');

        if a == b {
            return 0;
        }

        let parse_parts = |s: &str| -> Vec<u32> {
            s.split(['.', '-'])
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<u32>().unwrap_or(0))
                .collect()
        };

        let a_parts = parse_parts(a);
        let b_parts = parse_parts(b);

        for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
            let va = a_parts.get(i).unwrap_or(&0);
            let vb = b_parts.get(i).unwrap_or(&0);
            if va != vb {
                return if va > vb { 1 } else { -1 };
            }
        }

        if a_parts.len() != b_parts.len() {
            return if a_parts.len() > b_parts.len() { 1 } else { -1 };
        }

        0
    }

    pub fn check_for_updates_simple(current_version: &str) -> Result<Option<ReleaseInfo>> {
        Self::check_for_updates_from_url(current_version, LATEST_RELEASE_API_URL)
    }

    fn check_for_updates_from_url(current_version: &str, url: &str) -> Result<Option<ReleaseInfo>> {
        let Some(release) = ReleaseClient::fetch_latest_release(url)? else {
            return Ok(None);
        };
        let tag_version = release.tag_name.trim_start_matches('v');
        let curr_version = current_version.trim_start_matches('v');

        if !Self::is_newer_version(curr_version, tag_version) {
            Ok(None)
        } else {
            let download_url = Self::build_download_url(&release.tag_name);
            Ok(Some(ReleaseInfo {
                tag_name: release.tag_name,
                html_url: release.html_url,
                body: release.body,
                download_url,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_update_asset_is_tar_gz() {
        let tag = "v0.22.11";
        assert_eq!(
            super::UpdateOps::platform_asset_name(),
            "KatanA-linux-x86_64.tar.gz"
        );
        assert_eq!(
            super::UpdateOps::build_download_url(tag),
            "https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.11/KatanA-linux-x86_64.tar.gz"
        );
        assert!(super::UpdateOps::build_download_url(tag).ends_with(".tar.gz"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_macos_update_asset_is_zip() {
        let tag = "v0.22.11";
        assert_eq!(super::UpdateOps::platform_asset_name(), "KatanA-macOS.zip");
        assert_eq!(
            super::UpdateOps::build_download_url(tag),
            "https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.11/KatanA-macOS.zip"
        );
        assert!(super::UpdateOps::build_download_url(tag).ends_with(".zip"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_update_asset_is_zip() {
        let tag = "v0.22.11";
        assert_eq!(
            super::UpdateOps::platform_asset_name(),
            "KatanA-windows-x86_64.zip"
        );
        assert_eq!(
            super::UpdateOps::build_download_url(tag),
            "https://github.com/HiroyukiFuruno/KatanA/releases/download/v0.22.11/KatanA-windows-x86_64.zip"
        );
        assert!(super::UpdateOps::build_download_url(tag).ends_with(".zip"));
    }
}
