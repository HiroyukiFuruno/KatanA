use super::types::{ReleaseInfo, UpdateManager, UpdateOps};
use anyhow::Result;
use ureq::ResponseExt;

const HTTP_OK: u16 = 200;

impl UpdateOps {
    pub fn check_for_updates(&self, manager: &UpdateManager) -> Result<Option<ReleaseInfo>> {
        let url = manager
            .api_url_override
            .as_deref()
            .unwrap_or("https://github.com/HiroyukiFuruno/KatanA/releases/latest");

        let resp = ureq::get(url)
            .header("User-Agent", "KatanA-Update-Manager")
            .header("Accept", "text/html")
            .call()?;

        if resp.status() != HTTP_OK {
            return Ok(None);
        }

        let final_url = resp.get_uri().to_string();
        let tag_name = if let Some(idx) = final_url.rfind("releases/tag/") {
            final_url[idx + "releases/tag/".len()..].to_string()
        } else {
            return Ok(None);
        };

        let tag_version = tag_name.trim_start_matches('v');
        let curr_version = manager.current_version.trim_start_matches('v');

        #[cfg(target_os = "macos")]
        const ASSET_NAME: &str = "KatanA-macOS.zip";

        #[cfg(target_os = "windows")]
        const ASSET_NAME: &str = "KatanA-windows-x86_64.zip";

        #[cfg(target_os = "linux")]
        const ASSET_NAME: &str = "KatanA-linux-x86_64.zip";

        if !Self::is_newer_version(curr_version, tag_version) {
            Ok(None)
        } else {
            let html_url = final_url.to_string();
            let download_url = format!(
                "https://github.com/HiroyukiFuruno/KatanA/releases/download/{}/{}",
                tag_name, ASSET_NAME
            );
            Ok(Some(ReleaseInfo {
                tag_name,
                html_url,
                body: String::new(),
                download_url,
            }))
        }
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
        let url = "https://github.com/HiroyukiFuruno/KatanA/releases/latest";

        let resp = ureq::get(url)
            .header("User-Agent", "KatanA-Update-Manager")
            .header("Accept", "text/html")
            .call()?;

        if resp.status() != HTTP_OK {
            return Ok(None);
        }

        let final_url = resp.get_uri().to_string();
        let tag_name = if let Some(idx) = final_url.rfind("releases/tag/") {
            final_url[idx + "releases/tag/".len()..].to_string()
        } else {
            return Ok(None);
        };

        let tag_version = tag_name.trim_start_matches('v');
        let curr_version = current_version.trim_start_matches('v');

        #[cfg(target_os = "macos")]
        const ASSET_NAME: &str = "KatanA-macOS.zip";

        #[cfg(target_os = "windows")]
        const ASSET_NAME: &str = "KatanA-windows-x86_64.zip";

        #[cfg(target_os = "linux")]
        const ASSET_NAME: &str = "KatanA-linux-x86_64.zip";

        if !Self::is_newer_version(curr_version, tag_version) {
            Ok(None)
        } else {
            let html_url = final_url.to_string();
            let download_url = format!(
                "https://github.com/HiroyukiFuruno/KatanA/releases/download/{}/{}",
                tag_name, ASSET_NAME
            );
            Ok(Some(ReleaseInfo {
                tag_name,
                html_url,
                body: String::new(),
                download_url,
            }))
        }
    }
}
