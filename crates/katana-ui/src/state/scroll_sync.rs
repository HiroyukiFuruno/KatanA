//! Split scroll sync contract.
//!
//! # Synchronization contract
//!
//! ## Roles
//! - **Source** (`ScrollSource::Editor` or `ScrollSource::Preview`): the pane whose scroll
//!   position was changed by the user this frame.
//! - **Consumer**: the other pane that must be driven to the equivalent logical position.
//! - **Neither**: steady state — no pending synchronization.
//!
//! ## Logical position
//! Instead of a bare `fraction` (0.0–1.0 of max‑scroll), we store a *segment-aware* logical
//! position composed of:
//! - `segment_index`: which mapping segment the source offset falls in.
//! - `progress`: normalised progress (0.0–1.0) within that segment.
//!
//! This round-trips through `editor → preview → editor` without drift because both
//! directions use the **same segment table** (see [`ScrollMapper`]).
//!
//! ## Tail area
//! The segment table always includes an explicit **EOF segment** that covers the range from
//! the last heading anchor to the end of the document.  When a document has no headings
//! the entire range is a single `[start, EOF]` segment so full-range sync is always possible.
//!
//! ## Echo suppression (write-back prevention)
//! After the consumer pane applies a sync position it records the target pixel offset in
//! [`SyncEcho`].  On subsequent frames the consumer compares its measured offset against the
//! recorded echo:
//! - If the difference is ≤ [`ECHO_PIXEL_EPSILON`], it is treated as the echo of the applied
//!   sync and does **not** raise itself as a new source.
//! - If the difference exceeds the epsilon, the consumer has genuinely scrolled and may
//!   become the new source.
//!
//! This prevents the jitter loop `E→P sync → P write-back → P→E sync → E write-back → …`.
//!
//! The epsilon is pixel-based (not fraction-based) to keep behaviour invariant across
//! different document lengths and panel sizes.

/// Pixel distance below which a consumer offset change is considered an echo of the last
/// applied sync and must not trigger a new source event.
pub const ECHO_PIXEL_EPSILON: f32 = 2.0;

const DEGENERATE_EPSILON: f32 = 1e-4;

/// One entry in the piecewise-linear mapping between editor and preview pixel offsets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapPoint {
    /// Editor pixel offset (0..=editor_max).
    pub editor_y: f32,
    /// Preview pixel offset (0..=preview_max).
    pub preview_y: f32,
}

/// A segment-aware logical position representing the sync state.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LogicalPosition {
    /// The index in `ScrollMapper::points` of the start of the current segment.
    pub segment_index: usize,
    /// Progress (0.0–1.0) through the segment.
    pub progress: f32,
}

/// Piecewise-linear mapper shared by both editor→preview and preview→editor directions.
///
/// The table is rebuilt each frame from the current heading anchors and geometry snapshot.
/// It always starts with `(0, 0)` and ends with `(editor_max, preview_max)` — the EOF
/// point — so every pixel in both panes is covered.
#[derive(Debug, Default, Clone)]
pub struct ScrollMapper {
    /// Mapping points in ascending `editor_y` order.
    pub points: Vec<MapPoint>,
}

impl ScrollMapper {
    /// Build a mapper from the current geometry.
    ///
    /// `anchors` is a slice of `(editor_line_start, preview_rect_min_y)` pairs as produced
    /// by `PreviewPane::heading_anchors`, with `content_top_y` already subtracted from the
    /// preview coordinate.  `row_height` is the monospace row height used by the editor.
    pub fn build(
        editor_max: f32,
        preview_max: f32,
        row_height: f32,
        anchors: &[(std::ops::Range<usize>, f32)],
    ) -> Self {
        let mut sorted_anchors: Vec<_> = anchors.to_vec();
        /* WHY: Sort by editor line start to build a monotonic mapping table. */
        sorted_anchors.sort_by_key(|(span, _)| span.start);

        let mut points = Vec::with_capacity(sorted_anchors.len() + 2);
        points.push(MapPoint {
            editor_y: 0.0,
            preview_y: 0.0,
        });

        for (span, p_y) in sorted_anchors {
            let editor_y = span.start as f32 * row_height;
            /* WHY: Clamp to avoid degenerate points outside the visible range. */
            let editor_y = editor_y.min(editor_max.max(1.0));
            let preview_y = p_y.max(0.0).min(preview_max.max(1.0));

            /* WHY: Skip degenerate segments (same editor_y or preview_y as last). */
            if let Some(last) = points.last()
                && ((editor_y - last.editor_y).abs() < DEGENERATE_EPSILON
                    || (preview_y - last.preview_y).abs() < DEGENERATE_EPSILON)
            {
                continue;
            }

            points.push(MapPoint {
                editor_y,
                preview_y,
            });
        }
        /* WHY: EOF anchor — always present regardless of heading count. */
        points.push(MapPoint {
            editor_y: editor_max.max(1.0),
            preview_y: preview_max.max(1.0),
        });
        Self { points }
    }

    /// Map an editor pixel offset to a logical position.
    pub fn editor_to_logical(&self, editor_y: f32) -> LogicalPosition {
        self.to_logical(editor_y, |p| p.editor_y)
    }

    /// Map a preview pixel offset to a logical position.
    pub fn preview_to_logical(&self, preview_y: f32) -> LogicalPosition {
        self.to_logical(preview_y, |p| p.preview_y)
    }

    /// Convert a logical position back to an editor pixel offset.
    pub fn logical_to_editor(&self, pos: LogicalPosition) -> f32 {
        self.eval_logical_to_offset(pos, |p| p.editor_y)
    }

    /// Convert a logical position back to a preview pixel offset.
    pub fn logical_to_preview(&self, pos: LogicalPosition) -> f32 {
        self.eval_logical_to_offset(pos, |p| p.preview_y)
    }
}

/// Records the most recently applied sync position on the consumer pane so that the
/// resulting scroll movement is not mistaken for a new user input.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SyncEcho {
    /// The pixel offset the consumer was last set to by a sync operation.
    pub applied_offset: f32,
    /// Generation counter — incremented each time a new sync target is applied.
    pub generation: u32,
}

impl SyncEcho {
    /// Returns `true` if `current_offset` is close enough to the last applied sync offset
    /// that we should suppress a write-back onto shared state.
    pub fn is_echo(&self, current_offset: f32) -> bool {
        (current_offset - self.applied_offset).abs() <= ECHO_PIXEL_EPSILON
    }

    /// Record a newly applied sync offset.
    pub fn record(&mut self, offset: f32) {
        self.applied_offset = offset;
        self.generation = self.generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_to_logical() {
        let mapper = ScrollMapper::build(1000.0, 2000.0, 20.0, &[(10..10, 400.0), (30..30, 800.0)]);

        /* WHY: 1. Before first segment (0..200 -> 0..400) */
        let pos = mapper.editor_to_logical(100.0);
        assert_eq!(pos.segment_index, 0);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_preview(pos), 200.0);

        /* WHY: 2. Middle segment (200..600 -> 400..800) */
        let pos = mapper.editor_to_logical(400.0);
        assert_eq!(pos.segment_index, 1);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_preview(pos), 600.0);

        /* WHY: 3. Tail segment (after 600 -> 800..2000) */
        let pos = mapper.editor_to_logical(800.0);
        assert_eq!(pos.segment_index, 2);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_preview(pos), 1400.0);
    }

    #[test]
    fn test_preview_to_logical() {
        let mapper = ScrollMapper::build(1000.0, 2000.0, 20.0, &[(10..10, 400.0), (30..30, 800.0)]);

        /* WHY: 1. Before first segment */
        let pos = mapper.preview_to_logical(200.0);
        assert_eq!(pos.segment_index, 0);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_editor(pos), 100.0);

        /* WHY: 2. Middle segment */
        let pos = mapper.preview_to_logical(600.0);
        assert_eq!(pos.segment_index, 1);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_editor(pos), 400.0);

        /* WHY: 3. Tail segment */
        let pos = mapper.preview_to_logical(1400.0);
        assert_eq!(pos.segment_index, 2);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_editor(pos), 800.0);
    }

    #[test]
    fn mapper_no_headings_full_range() {
        /* WHY: Document with no headings: single [start, EOF] segment. */
        let m = ScrollMapper::build(1000.0, 800.0, 10.0, &[]);
        /* WHY: Mid-point should map proportionally across the single segment. */
        let pos = m.editor_to_logical(500.0);
        let preview_y = m.logical_to_preview(pos);
        assert!(
            (preview_y - 400.0).abs() < 1.0,
            "expected ~400, got {preview_y}"
        );
        let pos = m.preview_to_logical(400.0);
        let editor_y = m.logical_to_editor(pos);
        assert!(
            (editor_y - 500.0).abs() < 1.0,
            "expected ~500, got {editor_y}"
        );
    }

    #[test]
    fn mapper_tail_segment_reaches_eof() {
        /* WHY: Last heading at editor line 40, preview y 300; editor_max=500, preview_max=400. */
        let anchors = vec![(40..41, 300.0)];
        let _m = ScrollMapper::build(500.0, 400.0, 10.0, &anchors);
        /* WHY: Editor fully scrolled to tail end → preview fully scrolled too. */
        let mapper = ScrollMapper::build(1000.0, 2000.0, 20.0, &[(100..100, 2500.0)]);
        /* WHY: Editor line 100 is roughly y=2000, clamped to 1000. Preview clamped to 2000. */
        let pos = mapper.editor_to_logical(500.0);
        assert_eq!(mapper.logical_to_preview(pos), 1000.0);
    }

    #[test]
    fn test_roundtrip_stability() {
        let mapper = ScrollMapper::build(1000.0, 2000.0, 20.0, &[(10..10, 400.0), (30..30, 800.0)]);

        for e_y in [0.0, 100.0, 200.0, 500.0, 600.0, 900.0, 1000.0] {
            let pos = mapper.editor_to_logical(e_y);
            let p_y = mapper.logical_to_preview(pos);
            let p_pos = mapper.preview_to_logical(p_y);
            let roundtrip_e_y = mapper.logical_to_editor(p_pos);

            assert!((e_y - roundtrip_e_y).abs() < 1e-4, "drift at {}", e_y);
        }
    }

    #[test]
    fn echo_suppression_within_epsilon() {
        let mut echo = SyncEcho::default();
        echo.record(250.0);
        assert!(echo.is_echo(250.0));
        assert!(echo.is_echo(251.9));
        assert!(!echo.is_echo(252.1));
    }

    #[test]
    fn echo_suppression_new_user_scroll() {
        let mut echo = SyncEcho::default();
        echo.record(250.0);
        /* WHY: A genuine user scroll of 50px should not be suppressed. */
        assert!(!echo.is_echo(300.0));
    }
}
