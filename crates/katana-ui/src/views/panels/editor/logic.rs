use crate::app_state::ScrollSource;
use eframe::egui;

/// Editor color tuple: (code_bg, code_text, code_selection, current_line_bg, hover_line_bg, ln_text, ln_active_text).
pub type EditorColors = (
    egui::Color32,
    egui::Color32,
    Option<egui::Color32>,
    Option<egui::Color32>,
    Option<egui::Color32>,
    Option<egui::Color32>,
    Option<egui::Color32>,
);

/// Resolve editor theme colors from the egui context's temporary data.
pub fn resolve_editor_colors(ui: &egui::Ui) -> EditorColors {
    ui.ctx().data(|d| {
        if let Some(tc) =
            d.get_temp::<katana_platform::theme::ThemeColors>(egui::Id::new("katana_theme_colors"))
        {
            (
                crate::theme_bridge::rgb_to_color32(tc.code.background),
                crate::theme_bridge::rgb_to_color32(tc.code.text),
                Some(crate::theme_bridge::rgb_to_color32(tc.code.selection)),
                Some(crate::theme_bridge::rgba_to_color32(
                    tc.code.current_line_background,
                )),
                Some(crate::theme_bridge::rgba_to_color32(
                    tc.code.hover_line_background,
                )),
                Some(crate::theme_bridge::rgb_to_color32(
                    tc.code.line_number_text,
                )),
                Some(crate::theme_bridge::rgb_to_color32(
                    tc.code.line_number_active_text,
                )),
            )
        } else {
            (
                ui.visuals().extreme_bg_color,
                ui.visuals().text_color(),
                None,
                None,
                None,
                None,
                None,
            )
        }
    })
}

/// Count the line (paragraph) number for a given character index in the buffer.
pub fn char_index_to_line(buffer: &str, char_idx: usize) -> usize {
    buffer
        .chars()
        .take(char_idx)
        .filter(|&ch| ch == '\n')
        .count()
}

/// Convert a line number to the character index at the start of that line.
pub fn line_to_char_index(buffer: &str, target_line: usize) -> Option<usize> {
    let mut current_line = 0;
    for (char_idx, c) in buffer.chars().enumerate() {
        if current_line == target_line {
            return Some(char_idx);
        }
        if c == '\n' {
            current_line += 1;
        }
    }
    None
}

/// Convert a line range to (start_char_index, end_char_index) in the buffer.
pub fn line_range_to_char_range(
    buffer: &str,
    line_start: usize,
    line_end: usize,
) -> Option<(usize, usize)> {
    let mut current_line = 0;
    let mut start_char = None;
    let mut end_char = None;

    for (char_idx, c) in buffer.chars().enumerate() {
        if current_line == line_start && start_char.is_none() {
            start_char = Some(char_idx);
        }
        if current_line == line_end + 1 {
            end_char = Some(char_idx.saturating_sub(1));
            break;
        }
        if c == '\n' {
            current_line += 1;
        }
    }
    if start_char.is_some() && end_char.is_none() {
        end_char = Some(buffer.chars().count().saturating_sub(1));
    }

    match (start_char, end_char) {
        (Some(s), Some(e)) => Some((s, e)),
        _ => None,
    }
}

/// Compute the current line highlight color, falling back to a semi-transparent overlay.
pub fn current_line_highlight_color(
    dark_mode: bool,
    themed_color: Option<egui::Color32>,
) -> egui::Color32 {
    const HIGHLIGHT_ALPHA: u8 = 15;
    themed_color.unwrap_or_else(|| {
        if dark_mode {
            crate::theme_bridge::from_white_alpha(HIGHLIGHT_ALPHA)
        } else {
            crate::theme_bridge::from_black_alpha(HIGHLIGHT_ALPHA)
        }
    })
}

/// Compute the hover highlight color for preview-linked lines.
pub fn hover_line_highlight_color(
    dark_mode: bool,
    themed_color: Option<egui::Color32>,
) -> egui::Color32 {
    const HOVER_HIGHLIGHT_ALPHA: u8 = 10;
    themed_color.unwrap_or_else(|| {
        if dark_mode {
            crate::theme_bridge::from_white_alpha(HOVER_HIGHLIGHT_ALPHA)
        } else {
            crate::theme_bridge::from_black_alpha(HOVER_HIGHLIGHT_ALPHA)
        }
    })
}

/// Update scroll synchronization state after editor rendering.
pub fn update_scroll_sync(
    scroll: &mut crate::app_state::ScrollState,
    content_height: f32,
    inner_rect_height: f32,
    current_offset_y: f32,
    was_consuming_preview: bool,
    dead_zone: f32,
) {
    let max_scroll = (content_height - inner_rect_height).max(0.0);
    scroll.editor_max = max_scroll;

    if was_consuming_preview {
        scroll.source = ScrollSource::Neither;
        if max_scroll > 0.0 {
            scroll.fraction = (current_offset_y / max_scroll).clamp(0.0, 1.0);
        }
    } else if max_scroll > 0.0 {
        let current_fraction = (current_offset_y / max_scroll).clamp(0.0, 1.0);
        let diff = (current_fraction - scroll.fraction).abs();
        if diff > dead_zone {
            scroll.fraction = current_fraction;
            scroll.source = ScrollSource::Editor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_index_to_line_at_start() {
        assert_eq!(char_index_to_line("hello\nworld\n", 0), 0);
    }

    #[test]
    fn char_index_to_line_middle() {
        assert_eq!(char_index_to_line("line0\nline1\nline2\n", 6), 1);
    }

    #[test]
    fn char_index_to_line_end() {
        assert_eq!(char_index_to_line("a\nb\nc\n", 5), 2);
    }

    #[test]
    fn line_to_char_index_first_line() {
        assert_eq!(line_to_char_index("hello\nworld\n", 0), Some(0));
    }

    #[test]
    fn line_to_char_index_second_line() {
        assert_eq!(line_to_char_index("hello\nworld\n", 1), Some(6));
    }

    #[test]
    fn line_to_char_index_out_of_range() {
        assert_eq!(line_to_char_index("hello\n", 5), None);
    }

    #[test]
    fn line_range_to_char_range_single_line() {
        let buf = "line0\nline1\nline2\n";
        let result = line_range_to_char_range(buf, 1, 1);
        assert_eq!(result, Some((6, 11)));
    }

    #[test]
    fn line_range_to_char_range_multiple_lines() {
        let buf = "line0\nline1\nline2\n";
        let result = line_range_to_char_range(buf, 0, 1);
        assert_eq!(result, Some((0, 11)));
    }

    #[test]
    fn line_range_to_char_range_to_end() {
        let buf = "line0\nline1\nline2";
        let result = line_range_to_char_range(buf, 2, 2);
        assert_eq!(result, Some((12, 16)));
    }

    #[test]
    fn highlight_color_uses_themed_when_available() {
        // Use selection color from visuals to avoid hardcoded-color lint
        let custom = egui::Color32::PLACEHOLDER;
        assert_eq!(current_line_highlight_color(true, Some(custom)), custom);
        assert_eq!(hover_line_highlight_color(false, Some(custom)), custom);
    }

    #[test]
    fn highlight_color_falls_back_for_dark_mode() {
        let color = current_line_highlight_color(true, None);
        assert_ne!(color, egui::Color32::TRANSPARENT);
    }

    #[test]
    fn highlight_color_falls_back_for_light_mode() {
        let color = current_line_highlight_color(false, None);
        assert_ne!(color, egui::Color32::TRANSPARENT);
    }

    #[test]
    fn update_scroll_sync_consuming_preview_resets_source() {
        let mut scroll = crate::app_state::ScrollState::default();
        scroll.source = ScrollSource::Preview;
        update_scroll_sync(&mut scroll, 1000.0, 500.0, 250.0, true, 0.01);
        assert_eq!(scroll.source, ScrollSource::Neither);
        assert!((scroll.fraction - 0.5).abs() < 0.01);
    }

    #[test]
    fn update_scroll_sync_editor_scrolled_beyond_dead_zone() {
        let mut scroll = crate::app_state::ScrollState::default();
        scroll.fraction = 0.0;
        update_scroll_sync(&mut scroll, 1000.0, 500.0, 400.0, false, 0.01);
        assert_eq!(scroll.source, ScrollSource::Editor);
        assert!((scroll.fraction - 0.8).abs() < 0.01);
    }

    #[test]
    fn update_scroll_sync_within_dead_zone_no_change() {
        let mut scroll = crate::app_state::ScrollState::default();
        scroll.fraction = 0.5;
        scroll.source = ScrollSource::Neither;
        update_scroll_sync(&mut scroll, 1000.0, 500.0, 252.0, false, 0.01);
        assert_eq!(scroll.source, ScrollSource::Neither);
    }
}
