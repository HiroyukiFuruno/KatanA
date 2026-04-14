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
            last_scroll_to_line: None,
            hovered_preview_lines: Vec::new(),
            sync_override: None,
        }
    }
}
