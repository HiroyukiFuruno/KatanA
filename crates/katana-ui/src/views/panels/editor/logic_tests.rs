#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_index_to_line_at_start() {
        assert_eq!(EditorLogicOps::char_index_to_line("hello\nworld\n", 0), 0);
    }

    #[test]
    fn char_index_to_line_middle() {
        assert_eq!(
            EditorLogicOps::char_index_to_line("line0\nline1\nline2\n", 6),
            1
        );
    }

    #[test]
    fn char_index_to_line_end() {
        assert_eq!(EditorLogicOps::char_index_to_line("a\nb\nc\n", 5), 2);
    }

    #[test]
    fn line_to_char_index_first_line() {
        assert_eq!(
            EditorLogicOps::line_to_char_index("hello\nworld\n", 0),
            Some(0)
        );
    }

    #[test]
    fn line_to_char_index_second_line() {
        assert_eq!(
            EditorLogicOps::line_to_char_index("hello\nworld\n", 1),
            Some(6)
        );
    }

    #[test]
    fn line_to_char_index_out_of_range() {
        assert_eq!(EditorLogicOps::line_to_char_index("hello\n", 5), None);
    }

    #[test]
    fn line_range_to_char_range_single_line() {
        let buf = "line0\nline1\nline2\n";
        let result = EditorLogicOps::line_range_to_char_range(buf, 1, 1);
        assert_eq!(result, Some((6, 11)));
    }

    #[test]
    fn line_range_to_char_range_multiple_lines() {
        let buf = "line0\nline1\nline2\n";
        let result = EditorLogicOps::line_range_to_char_range(buf, 0, 1);
        assert_eq!(result, Some((0, 11)));
    }

    #[test]
    fn line_range_to_char_range_to_end() {
        let buf = "line0\nline1\nline2";
        let result = EditorLogicOps::line_range_to_char_range(buf, 2, 2);
        assert_eq!(result, Some((12, 16)));
    }

    #[test]
    fn highlight_color_uses_themed_when_available() {
        /* WHY: Use selection color from visuals to avoid hardcoded-color lint */
        let custom = egui::Color32::PLACEHOLDER;
        assert_eq!(
            EditorLogicOps::current_line_highlight_color(true, Some(custom)),
            custom
        );
        assert_eq!(
            EditorLogicOps::hover_line_highlight_color(false, Some(custom)),
            custom
        );
    }

    #[test]
    fn highlight_color_falls_back_for_dark_mode() {
        let color = EditorLogicOps::current_line_highlight_color(true, None);
        assert_ne!(color, egui::Color32::TRANSPARENT);
    }

    #[test]
    fn highlight_color_falls_back_for_light_mode() {
        let color = EditorLogicOps::current_line_highlight_color(false, None);
        assert_ne!(color, egui::Color32::TRANSPARENT);
    }

    #[test]
    fn update_scroll_sync_consuming_preview_resets_source() {
        let mut scroll = crate::app_state::ScrollState {
            source: ScrollSource::Preview,
            ..Default::default()
        };
        EditorLogicOps::update_scroll_sync(&mut scroll, 1000.0, 500.0, 250.0, true, 0.01);
        assert_eq!(scroll.source, ScrollSource::Neither);
        assert!(scroll.editor_echo.is_echo(250.0));
    }

    #[test]
    fn update_scroll_sync_editor_scrolled_beyond_epsilon() {
        let mut scroll = crate::app_state::ScrollState {
            mapper: crate::state::scroll_sync::ScrollMapper::build(500.0, 500.0, 20.0, &[]),
            ..Default::default()
        };
        /* WHY: 400.0 offset on 500.0 max_scroll means progress=0.8 */
        EditorLogicOps::update_scroll_sync(&mut scroll, 1000.0, 500.0, 400.0, false, 0.01);
        assert_eq!(scroll.source, ScrollSource::Editor);
    }

    #[test]
    fn update_scroll_sync_within_echo_no_change() {
        let mut scroll = crate::app_state::ScrollState {
            source: ScrollSource::Neither,
            ..Default::default()
        };
        scroll.editor_echo.record(250.0);
        EditorLogicOps::update_scroll_sync(&mut scroll, 1000.0, 500.0, 251.0, false, 0.01);
        assert_eq!(scroll.source, ScrollSource::Neither);
    }

    /// RED test: When `scroll_to_line` is active (e.g. TOC navigation in split mode),
    /// `update_scroll_sync` must NOT set `ScrollSource::Editor`, because the preview
    /// pane has its own independent `scroll_request` that would be overwritten by
    /// `compute_forced_offset` in the next frame through the mapper approximation.
    ///
    /// Root cause: TOC click sets both `scroll_request` (preview) and `scroll_to_line`
    /// (editor). The editor's `scroll_to_rect` changes `editor_y`, which triggers
    /// `ScrollSource::Editor` in `update_scroll_sync`. Next frame, preview's
    /// `compute_forced_offset` overwrites the scroll position via mapper, causing
    /// the heading to be positioned differently from the direct `scroll_request`.
    #[test]
    fn update_scroll_sync_with_scroll_to_line_does_not_set_editor_source() {
        let mut scroll = crate::app_state::ScrollState {
            mapper: crate::state::scroll_sync::ScrollMapper::build(500.0, 500.0, 20.0, &[]),
            scroll_to_line: Some(52), /* WHY: TOC click sets this */
            ..Default::default()
        };
        /* WHY: Editor scrolls to line 52 → offset changes to 400.0 */
        EditorLogicOps::update_scroll_sync(&mut scroll, 1000.0, 500.0, 400.0, false, 0.01);
        assert_ne!(
            scroll.source,
            ScrollSource::Editor,
            "When scroll_to_line is active, source must NOT be set to Editor; \
             otherwise the preview's scroll_request position will be overwritten by mapper"
        );
    }
    }
