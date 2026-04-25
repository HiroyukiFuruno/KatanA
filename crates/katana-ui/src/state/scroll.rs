use crate::state::scroll_sync::{LogicalPosition, ScrollMapper, SyncEcho};

#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum ScrollSource {
    #[default]
    Neither,
    Editor,
    Preview,
}

pub struct ScrollState {
    pub logical_position: LogicalPosition,
    pub mapper: ScrollMapper,
    pub source: ScrollSource,
    pub editor_y: f32,
    pub preview_y: f32,
    pub editor_max: f32,
    pub preview_max: f32,
    pub editor_echo: SyncEcho,
    pub preview_echo: SyncEcho,
    pub active_editor_line: Option<usize>,
    pub scroll_to_line: Option<usize>,
    pub editor_line_anchors: Vec<f32>,
    /* WHY: Tracks the last line that was actually scrolled to, so that
     * repeated Next/Prev on the same single match does not re-trigger
     * the forced scroll and cause jitter. */
    pub last_scroll_to_line: Option<usize>,
    pub hovered_preview_lines: Vec<std::ops::Range<usize>>,
    pub sync_override: Option<bool>,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            logical_position: LogicalPosition::default(),
            mapper: ScrollMapper::default(),
            source: ScrollSource::Neither,
            editor_y: 0.0,
            preview_y: 0.0,
            editor_max: 0.0,
            preview_max: 0.0,
            editor_echo: SyncEcho::default(),
            preview_echo: SyncEcho::default(),
            active_editor_line: None,
            scroll_to_line: None,
            editor_line_anchors: Vec::new(),
            last_scroll_to_line: None,
            hovered_preview_lines: Vec::new(),
            sync_override: None,
        }
    }

    pub fn reset_for_document_change(&mut self) {
        let sync_override = self.sync_override;
        *self = Self {
            sync_override,
            ..Self::new()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_for_document_change_clears_scroll_offsets() {
        let mut scroll = ScrollState {
            source: ScrollSource::Editor,
            editor_y: 320.0,
            preview_y: 180.0,
            scroll_to_line: Some(12),
            last_scroll_to_line: Some(12),
            sync_override: Some(false),
            ..Default::default()
        };

        scroll.reset_for_document_change();

        assert_eq!(scroll.source, ScrollSource::Neither);
        assert_eq!(scroll.editor_y, 0.0);
        assert_eq!(scroll.preview_y, 0.0);
        assert_eq!(scroll.scroll_to_line, None);
        assert_eq!(scroll.last_scroll_to_line, None);
        assert_eq!(scroll.sync_override, Some(false));
    }
}
