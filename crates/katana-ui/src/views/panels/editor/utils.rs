impl super::types::EditorLogicOps {
    /// Count the line (paragraph) number for a given character index in the buffer.
    pub fn char_index_to_line(buffer: &str, char_idx: usize) -> usize {
        buffer
            .chars()
            .take(char_idx)
            .filter(|&ch| ch == '\n')
            .count()
    }

    /// Convert a line number to the byte index at the start of that line.
    pub fn line_to_byte_index(buffer: &str, target_line: usize) -> Option<usize> {
        let mut current_line = 0;
        for (byte_idx, c) in buffer.char_indices() {
            if current_line == target_line {
                return Some(byte_idx);
            }
            if c == '\n' {
                current_line += 1;
            }
        }
        /* WHY: If target_line is exactly at the end of the string (e.g. empty last line) */
        if current_line == target_line {
            return Some(buffer.len());
        }
        None
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
        if current_line == target_line {
            return Some(buffer.chars().count());
        }
        None
    }

    /// Convert a line number (0-indexed) and column number (1-indexed characters) to a byte index in the buffer.
    /// Note: Linter columns are 1-indexed characters, so target_col should be target_col - 1.
    pub fn line_col_to_byte_index(
        buffer: &str,
        target_line: usize,
        target_col_chars: usize, // 1-indexed column in characters
    ) -> Option<usize> {
        let line_start_byte = Self::line_to_byte_index(buffer, target_line)?;
        let mut byte_idx = line_start_byte;

        /* WHY: Target col is 1-indexed, so we want to skip `target_col_chars - 1` characters. */
        let target_col0 = target_col_chars.saturating_sub(1);

        for (col, (offset, c)) in buffer[line_start_byte..].char_indices().enumerate() {
            if col == target_col0 || c == '\n' {
                return Some(line_start_byte + offset);
            }
            byte_idx = line_start_byte + offset + c.len_utf8();
        }
        Some(byte_idx)
    }

    /// Convert a line number and column number to a character index in the buffer.
    pub fn line_col_to_char_index(
        buffer: &str,
        target_line: usize,
        target_col_chars: usize,
    ) -> Option<usize> {
        let line_start_char = Self::line_to_char_index(buffer, target_line)?;
        let mut char_idx = line_start_char;

        let target_col0 = target_col_chars.saturating_sub(1);
        /* WHY: buffer[line_start_byte..] is needed to skip characters.
        But since we want to iterate chars, we can just skip `line_start_char` */
        for (col, c) in buffer.chars().skip(line_start_char).enumerate() {
            if col == target_col0 || c == '\n' {
                return Some(char_idx);
            }
            char_idx += 1;
        }
        Some(char_idx)
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
            if current_line == line_end && end_char.is_none() {
                end_char = Some(char_idx);
            }
            if c == '\n' {
                current_line += 1;
            }
        }

        if start_char.is_some() && end_char.is_none() {
            end_char = Some(buffer.len());
        }

        if let (Some(s), Some(e)) = (start_char, end_char) {
            Some((s, e))
        } else {
            None
        }
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
