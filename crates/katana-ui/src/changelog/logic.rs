use std::sync::mpsc::Sender;

use super::types::{ChangelogEvent, ChangelogOps, ChangelogSection};

const GITHUB_RAW_BASE: &str =
    "https://raw.githubusercontent.com/HiroyukiFuruno/KatanA/refs/heads/master";

impl ChangelogOps {
    pub(crate) fn get_changelog_url(language: &str, current_version: &str) -> String {
        let filename = if language.starts_with("ja") {
            "CHANGELOG.ja.md"
        } else {
            "CHANGELOG.md"
        };

        use std::time::{SystemTime, UNIX_EPOCH};
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        format!(
            "{}/{}?v={}&t={}",
            GITHUB_RAW_BASE, filename, current_version, ts
        )
    }

    pub fn fetch_changelog(
        language: &str,
        current_version: String,
        previous_version: Option<String>,
        tx: Sender<ChangelogEvent>,
    ) {
        let url = Self::get_changelog_url(language, &current_version);
        let request = ehttp::Request::get(&url);
        ehttp::fetch(request, move |result| {
            Self::handle_fetch_result(result, &tx, &current_version, previous_version.as_deref());
        });
    }

    pub(super) fn handle_fetch_result(
        result: Result<ehttp::Response, String>,
        tx: &Sender<ChangelogEvent>,
        current_version: &str,
        previous_version: Option<&str>,
    ) {
        match result {
            Ok(response) => {
                let text = match response.text() {
                    Some(t) => t.to_string(),
                    None => {
                        if response.ok {
                            let _ = tx.send(ChangelogEvent::Error(
                                "Failed to decode response text".to_string(),
                            ));
                        } else {
                            let _ = tx.send(ChangelogEvent::Error(format!(
                                "HTTP error: {}",
                                response.status
                            )));
                        }
                        return;
                    }
                };
                if !response.ok {
                    let _ = tx.send(ChangelogEvent::Error(format!(
                        "HTTP error {}: {}",
                        response.status,
                        text.chars().take(200).collect::<String>()
                    )));
                    return;
                }
                let sections = Self::parse_changelog(&text, current_version, previous_version);
                let _ = tx.send(ChangelogEvent::Success(sections));
            }
            Err(err) => {
                let _ = tx.send(ChangelogEvent::Error(err));
            }
        }
    }

    pub(super) fn parse_changelog(
        raw_markdown: &str,
        current_version: &str,
        previous_version: Option<&str>,
    ) -> Vec<ChangelogSection> {
        let prev_ver = previous_version.unwrap_or("0.0.0");
        let mut sections = Vec::new();
        let mut parts = raw_markdown.split("\n## [");
        let _ = parts.next();

        for part in parts {
            let bracket_end = part.find(']').unwrap_or(0);
            let version_str = part[..bracket_end].trim().to_string();
            let heading_end = part.find('\n').unwrap_or(part.len());
            let date_part = if bracket_end + 1 < heading_end {
                part[bracket_end + 1..heading_end]
                    .trim_start_matches([']', ' ', '-'])
                    .trim()
            } else {
                ""
            };
            let heading = if date_part.is_empty() {
                format!("v{}", version_str)
            } else {
                format!("v{} - {}", version_str, date_part)
            };
            let body = if heading_end < part.len() {
                part[heading_end..].trim_end().to_string()
            } else {
                String::new()
            };
            let default_open = version_str != "Unreleased"
                && Self::is_newer_or_equal(current_version, &version_str)
                && Self::is_older(prev_ver, &version_str);
            sections.push(ChangelogSection {
                version: version_str,
                heading,
                body,
                default_open,
            });
        }
        sections
    }

    pub(super) fn is_newer_or_equal(ver_a: &str, ver_b: &str) -> bool {
        Self::compare_versions(ver_a.trim_start_matches('v'), ver_b.trim_start_matches('v')) >= 0
    }

    pub(super) fn is_older(ver_a: &str, ver_b: &str) -> bool {
        Self::compare_versions(ver_a.trim_start_matches('v'), ver_b.trim_start_matches('v')) < 0
    }

    pub(super) fn compare_versions(a: &str, b: &str) -> i32 {
        let a_parts: Vec<u32> = a.split(['.', '-']).filter_map(|s| s.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split(['.', '-']).filter_map(|s| s.parse().ok()).collect();
        for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
            let va = a_parts.get(i).unwrap_or(&0);
            let vb = b_parts.get(i).unwrap_or(&0);
            if va > vb {
                return 1;
            }
            if va < vb {
                return -1;
            }
        }
        0
    }
}
