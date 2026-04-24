impl super::types::EditorLogicOps {
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
        if line_start > line_end {
            return None;
        }

        let start = Self::line_to_char_index(buffer, line_start)?;
        let mut current_line = 0;
        let mut end = None;
        for (char_idx, c) in buffer.chars().enumerate() {
            if current_line == line_end && c == '\n' {
                end = Some(char_idx);
                break;
            }
            if c == '\n' {
                current_line += 1;
            }
        }
        let end = end.unwrap_or_else(|| buffer.chars().count());
        Some((start, end))
    }

    /* WHY: Extract physical row Y positions for the start of each logical line. */
    pub fn extract_line_anchors(galley: &eframe::egui::text::Galley) -> Vec<f32> {
        let mut anchors = Vec::new();
        let mut is_first_row_of_line = true;
        for row in &galley.rows {
            if is_first_row_of_line {
                anchors.push(row.rect().min.y);
            }
            is_first_row_of_line = row.ends_with_newline;
        }
        anchors
    }

    /* WHY: Adds ghost space and default scroll-past-end padding to the editor. */
    pub fn render_editor_padding(
        ui: &mut eframe::egui::Ui,
        scroll: &crate::app_state::ScrollState,
    ) {
        let ghost_space = scroll.mapper.editor_ghost_space();
        if ghost_space > 0.0 {
            ui.add_space(ghost_space);
        }
        const MIN_EXTRA_PADDING: f32 = 0.5;
        ui.allocate_space(eframe::egui::vec2(
            0.0,
            ui.clip_rect().height() * MIN_EXTRA_PADDING,
        ));
    }
}
