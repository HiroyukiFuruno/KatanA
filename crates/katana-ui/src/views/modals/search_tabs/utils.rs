/* WHY: Common utility functions shared across search tabs to maintain consistency and reduce code duplication. */

use eframe::egui;

pub struct SearchUtilsOps;

impl SearchUtilsOps {
    pub(crate) fn build_regexes(pattern: &str) -> (Vec<regex::Regex>, bool) {
        let mut regexes = Vec::new();
        let mut valid = true;
        if pattern.is_empty() {
            return (regexes, valid);
        }
        for pat in pattern.split(',') {
            let pat = pat.trim();
            if pat.is_empty() {
                continue;
            }
            match regex::Regex::new(pat) {
                Ok(re) => regexes.push(re),
                Err(_) => valid = false,
            }
        }
        (regexes, valid)
    }

    pub(crate) fn get_error_color(ui: &egui::Ui) -> egui::Color32 {
        ui.ctx()
            .data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            })
            .map_or(crate::theme_bridge::WHITE, |tc| {
                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.system.error_text)
            })
    }

    pub(crate) fn build_snippet_job(
        ui: &egui::Ui,
        result: &katana_core::search::SearchResult,
    ) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();
        let (start, end) = (result.start_col, result.end_col);
        if start <= end && end <= result.snippet.len() {
            job.append(&result.snippet[..start], 0.0, egui::TextFormat::default());
            job.append(
                &result.snippet[start..end],
                0.0,
                egui::TextFormat {
                    color: ui.visuals().strong_text_color(),
                    background: ui.visuals().selection.bg_fill,
                    ..Default::default()
                },
            );
            job.append(&result.snippet[end..], 0.0, egui::TextFormat::default());
        } else {
            job.append(&result.snippet, 0.0, egui::TextFormat::default());
        }
        job
    }
}
