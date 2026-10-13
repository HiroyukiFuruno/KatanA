/* WHY: Decoupled utility functions for coordinate mapping and safety checks in section rendering. */

use eframe::egui;

pub struct SectionRenderUtilsOps;

impl SectionRenderUtilsOps {
    pub fn safe_byte_clamp(s: &str, idx: usize) -> usize {
        /* WHY: span indices from pulldown-cmark are byte offsets. When the source contains
        multi-byte characters (e.g. Japanese), a span boundary may fall inside a character.
        Slicing at that point causes a panic. We walk backwards from the given index to find
        the nearest valid char boundary before indexing into the string. */
        let clamped = idx.min(s.len());
        let mut safe = clamped;
        while safe > 0 && !s.is_char_boundary(safe) {
            safe -= 1;
        }
        safe
    }

    pub fn span_to_range(
        md: &str,
        span: &std::ops::Range<usize>,
        global_line_offset: usize,
        ensure_non_empty: bool,
        exclude_trailing_newline: bool,
    ) -> std::ops::Range<usize> {
        let safe_start = Self::safe_byte_clamp(md, span.start);
        let start_line =
            global_line_offset + md[..safe_start].chars().filter(|c| *c == '\n').count();

        let end_pos = if exclude_trailing_newline {
            span.end.saturating_sub(1).max(span.start)
        } else {
            span.end
        };

        let safe_end = Self::safe_byte_clamp(md, end_pos);
        let end_line = global_line_offset + md[..safe_end].chars().filter(|c| *c == '\n').count();

        if ensure_non_empty {
            let range_end = if end_line > start_line {
                end_line
            } else {
                start_line + 1
            };
            start_line..range_end
        } else {
            start_line..end_line
        }
    }

    pub fn byte_range_for_line(md: &str, local_line: usize) -> Option<std::ops::Range<usize>> {
        let mut current_line = 0;
        let mut start_byte = None;
        let mut end_byte = None;
        for (i, c) in md.char_indices() {
            if current_line == local_line && start_byte.is_none() {
                start_byte = Some(i);
            }
            if current_line == local_line + 1 {
                end_byte = Some(i);
                break;
            }
            if c == '\n' {
                current_line += 1;
            }
        }
        if current_line == local_line && start_byte.is_none() {
            start_byte = Some(0);
        }
        start_byte.map(|s| s..end_byte.unwrap_or(md.len()))
    }

    pub fn warning_color(ui: &egui::Ui) -> egui::Color32 {
        ui.ctx()
            .data(|d| {
                d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new(
                    "katana_theme_colors",
                ))
            })
            .map_or(crate::theme_bridge::WHITE, |tc| {
                crate::theme_bridge::ThemeBridgeOps::rgb_to_color32(tc.preview.warning_text)
            })
    }
}
