use eframe::egui;
use std::sync::mpsc::Sender;

mod types;
pub use types::{ChangelogEvent, ChangelogOps, ChangelogSection};

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

    fn handle_fetch_result(
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

    fn parse_changelog(
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

    fn is_newer_or_equal(ver_a: &str, ver_b: &str) -> bool {
        let a = ver_a.trim_start_matches('v');
        let b = ver_b.trim_start_matches('v');
        Self::compare_versions(a, b) >= 0
    }

    fn is_older(ver_a: &str, ver_b: &str) -> bool {
        let a = ver_a.trim_start_matches('v');
        let b = ver_b.trim_start_matches('v');
        Self::compare_versions(a, b) < 0
    }

    fn compare_versions(a: &str, b: &str) -> i32 {
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

    pub(crate) fn render_release_notes_tab(
        ui: &mut egui::Ui,
        sections: &[ChangelogSection],
        is_loading: bool,
    ) {
        if sections.is_empty() && !is_loading {
            return;
        }

        const TAB_OUTER_MARGIN_X: i8 = 32;
        const TAB_OUTER_MARGIN_Y: i8 = 24;
        const TAB_TITLE_SPACING: f32 = 16.0;
        const TAB_INNER_MARGIN_X: i8 = 16;
        const TAB_INNER_MARGIN_Y: i8 = 8;
        const TAB_BOTTOM_PADDING: f32 = 8.0;
        const TAB_SPINNER_SIZE: f32 = 32.0;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(TAB_BOTTOM_PADDING);

                if sections.is_empty() && is_loading {
                    ui.centered_and_justified(|ui| {
                        ui.add(egui::Spinner::new().size(TAB_SPINNER_SIZE));
                    });
                    return;
                }

                egui::Frame::default()
                    .inner_margin(egui::Margin::symmetric(
                        TAB_OUTER_MARGIN_X,
                        TAB_OUTER_MARGIN_Y,
                    ))
                    .show(ui, |ui| {
                        let title_text = format!(
                            "{} v{}",
                            crate::i18n::I18nOps::get().menu.release_notes,
                            env!("CARGO_PKG_VERSION")
                        );
                        ui.heading(egui::RichText::new(title_text));
                        ui.add_space(TAB_TITLE_SPACING);

                        for section in sections {
                            crate::widgets::Accordion::new(
                                &section.version,
                                egui::RichText::new(&section.heading).strong(),
                                |ui| {
                                    egui::Frame::default()
                                        .inner_margin(egui::Margin::symmetric(
                                            TAB_INNER_MARGIN_X,
                                            TAB_INNER_MARGIN_Y,
                                        ))
                                        .show(ui, |ui| {
                                            let mut cache = egui_commonmark::CommonMarkCache::default();
                                            egui_commonmark::CommonMarkViewer::new()
                                                .custom_task_box_fn(Some(
                                                    &crate::widgets::MarkdownHooksOps::katana_task_box,
                                                ))
                                                .custom_task_context_menu_fn(Some(
                                                    &crate::widgets::MarkdownHooksOps::katana_task_context_menu,
                                                ))
                                                .custom_emoji_fn(Some(
                                                    &katana_core::emoji::EmojiRasterOps::render_apple_color_emoji_png,
                                                ))
                                                .show(ui, &mut cache, &section.body);
                                        });
                                },
                            )
                            .default_open(section.default_open)
                            .show(ui);

                            ui.add_space(2.0);
                        }
                    });
            });
    }
}

#[cfg(test)]
mod tests;
