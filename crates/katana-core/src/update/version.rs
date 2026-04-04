use super::types::{ReleaseInfo, UpdateManager, UpdateOps};
use anyhow::Result;

/// HTTP 200 OK status code.
const HTTP_OK: u16 = 200;

impl UpdateOps {
    pub fn check_for_updates(&self, manager: &UpdateManager) -> Result<Option<ReleaseInfo>> {
        let url = manager
            .api_url_override
            .as_deref()
            .unwrap_or("https://api.github.com/repos/HiroyukiFuruno/KatanA/releases/latest");

        let resp = ureq::get(url)
            .set("User-Agent", "KatanA-Update-Manager")
            .call()?;

        if resp.status() != HTTP_OK {
            return Ok(None);
        }

        let info: ReleaseInfo = resp.into_json()?;
        if info.tag_name == manager.current_version {
            Ok(None)
        } else {
            Ok(Some(info))
        }
    }

    pub fn is_newer_version(current: &str, latest: &str) -> bool {
        latest != current
    }

    pub fn check_for_updates_simple(current_version: &str) -> Result<Option<ReleaseInfo>> {
        let url = "https://api.github.com/repos/HiroyukiFuruno/KatanA/releases/latest";

        let resp = ureq::get(url)
            .set("User-Agent", "KatanA-Update-Manager")
            .call()?;

        if resp.status() != HTTP_OK {
            return Ok(None);
        }

        let info: ReleaseInfo = resp.into_json()?;
        if info.tag_name == current_version {
            Ok(None)
        } else {
            Ok(Some(info))
        }
    }
}
