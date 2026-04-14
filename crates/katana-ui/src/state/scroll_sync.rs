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

/// Pixel distance within which a sync offset is snapped to the nearest heading anchor.
/// Roughly one line of monospace text — if the interpolated position is within this
/// distance of a heading, both panes should align their heading lines exactly.
pub const HEADING_SNAP_THRESHOLD: f32 = 20.0;

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
/// point.
#[derive(Debug, Default, Clone)]
pub struct ScrollMapper {
    /// Mapping points in ascending `editor_y` order.
    pub points: Vec<MapPoint>,
    /// The maximum vertical offset available in the editor during this build.
    pub editor_content_max: f32,
    /// The maximum vertical offset available in the preview during this build.
    pub preview_content_max: f32,
}

impl ScrollMapper {
    /// Build a mapper from the current geometry.
    ///
    /// `anchors` is a slice of `(editor_phys_y, preview_phys_y)` pairs.
    /// These are physical pixel offsets from the top of the respective scrollable content.
    pub fn build(editor_max: f32, preview_max: f32, anchors: &[(f32, f32)]) -> Self {
        let mut sorted_anchors: Vec<_> = anchors.to_vec();
        /* WHY: Sort by editor anchor y to build a monotonic mapping table. */
        sorted_anchors.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut points = Vec::with_capacity(sorted_anchors.len() + 2);
        points.push(MapPoint {
            editor_y: 0.0,
            preview_y: 0.0,
        });

        for (editor_y, preview_y) in sorted_anchors {
            // WHY: Only push if strictly monotonic to avoid degenerate linear segments
            if let Some(last) = points.last() {
                if editor_y > last.editor_y + DEGENERATE_EPSILON
                    && preview_y > last.preview_y + DEGENERATE_EPSILON
                {
                    points.push(MapPoint { editor_y, preview_y });
                }
            }
        }

        /* WHY: Always record a point at exactly EOF to ensure final coverage. */
        /* WHY: If both content are significantly longer than a single window height, */
        /* WHY: we map their respective bottom-most scroll limits together.            */
        let eof_editor = editor_max;
        let eof_preview = preview_max;

        if let Some(last) = points.last() {
            if (last.editor_y - eof_editor).abs() > DEGENERATE_EPSILON
                || (last.preview_y - eof_preview).abs() > DEGENERATE_EPSILON
            {
                // If EOF is strictly after the last anchor, add a segment to it
                if eof_editor > last.editor_y && eof_preview > last.preview_y {
                    points.push(MapPoint {
                        editor_y: eof_editor,
                        preview_y: eof_preview,
                    });
                } else {
                    // If the last anchor was already at or near EOF, force the last point to exactly EOF
                    let last_mut = points.last_mut().unwrap();
                    last_mut.editor_y = eof_editor;
                    last_mut.preview_y = eof_preview;
                }
            }
        }

        Self {
            points,
            editor_content_max: editor_max,
            preview_content_max: preview_max,
        }
    }

    #[rustfmt::skip]
    pub fn editor_to_logical(&self, y: f32) -> LogicalPosition { self.to_logical(y, |p| p.editor_y) }
    #[rustfmt::skip]
    pub fn preview_to_logical(&self, y: f32) -> LogicalPosition { self.to_logical(y, |p| p.preview_y) }
    #[rustfmt::skip]
    pub fn logical_to_editor(&self, p: LogicalPosition) -> f32 { self.eval_logical_to_offset(p, |p| p.editor_y) }
    #[rustfmt::skip]
    pub fn logical_to_preview(&self, p: LogicalPosition) -> f32 { self.eval_logical_to_offset(p, |p| p.preview_y) }
    #[rustfmt::skip]
    pub fn snap_to_heading_editor(&self, y: f32) -> f32 { self.snap_to_nearest(y, |p| p.editor_y) }
    #[rustfmt::skip]
    pub fn snap_to_heading_preview(&self, y: f32) -> f32 { self.snap_to_nearest(y, |p| p.preview_y) }

    pub fn editor_ghost_space(&self) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        let last = self.points.last().unwrap();
        (last.editor_y - self.editor_content_max).max(0.0)
    }

    pub fn preview_ghost_space(&self) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        let last = self.points.last().unwrap();
        (last.preview_y - self.preview_content_max).max(0.0)
    }

    fn snap_to_nearest(&self, offset: f32, get: impl Fn(&MapPoint) -> f32) -> f32 {
        /* WHY: Skip origin (index 0) and EOF (last) — they are synthetic anchors, */
        /* WHY: not real heading positions. Only snap to actual heading anchors.    */
        let len = self.points.len();
        if len <= 2 {
            return offset;
        }
        for pt in &self.points[1..len - 1] {
            let anchor = get(pt);
            if (offset - anchor).abs() <= HEADING_SNAP_THRESHOLD {
                return anchor;
            }
        }
        offset
    }
}

/// Records the most recently applied sync position on the consumer pane.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SyncEcho {
    pub applied_offset: f32, /* WHY: The pixel offset the consumer was last set to by a sync operation. */
    pub generation: u32, /* WHY: Generation counter — incremented each time a new sync target is applied. */
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
        let mapper = ScrollMapper::build(1000.0, 2000.0, &[(400.0, 800.0), (800.0, 1600.0)]);

        /* WHY: 1. Before first anchor (0,0 -> 400,800) */
        let pos = mapper.editor_to_logical(200.0);
        assert_eq!(pos.segment_index, 0);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_preview(pos), 400.0);

        /* WHY: 2. Middle segment (400,800 -> 800,1600) */
        let pos = mapper.editor_to_logical(600.0);
        assert_eq!(pos.segment_index, 1);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_preview(pos), 1200.0);

        /* WHY: 3. Tail segment (after 800 -> 1000,2000) */
        let pos = mapper.editor_to_logical(900.0);
        assert_eq!(pos.segment_index, 2);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_preview(pos), 1800.0);
    }

    #[test]
    fn test_preview_to_logical() {
        let mapper = ScrollMapper::build(1000.0, 2000.0, &[(400.0, 800.0), (800.0, 1600.0)]);

        /* WHY: 1. Before first segment */
        let pos = mapper.preview_to_logical(400.0);
        assert_eq!(pos.segment_index, 0);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_editor(pos), 200.0);

        /* WHY: 2. Middle segment */
        let pos = mapper.preview_to_logical(1200.0);
        assert_eq!(pos.segment_index, 1);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_editor(pos), 600.0);

        /* WHY: 3. Tail segment */
        let pos = mapper.preview_to_logical(1800.0);
        assert_eq!(pos.segment_index, 2);
        assert_eq!(pos.progress, 0.5);
        assert_eq!(mapper.logical_to_editor(pos), 900.0);
    }

    #[test]
    fn mapper_no_headings_full_range() {
        /* WHY: Document with no headings: single [start, EOF] segment. */
        let m = ScrollMapper::build(1000.0, 800.0, &[]);
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
        /* WHY: Last heading at editor pixel 400, preview y 300; editor_max=500, preview_max=400. */
        let anchors = vec![(400.0, 300.0)];
        let _m = ScrollMapper::build(500.0, 400.0, &anchors);
        /* WHY: Editor fully scrolled to tail end → preview fully scrolled too. */
        let mapper = ScrollMapper::build(1000.0, 2000.0, &[(800.0, 1500.0)]);
        /* WHY: Editor pixel 1000 is EOF. Preview pixel 2000 is EOF. */
        let pos = mapper.editor_to_logical(1000.0);
        assert_eq!(mapper.logical_to_preview(pos), 2000.0);
    }

    #[test]
    fn test_roundtrip_stability() {
        let mapper = ScrollMapper::build(1000.0, 2000.0, &[(200.0, 400.0), (600.0, 800.0)]);

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

    #[test]
    fn mapper_skips_non_monotonic_segments() {
        /* WHY: Ensure that backward or zero-height anchors in preview_y or editor_y are properly discarded. */
        let anchors = vec![(200.0, 400.0), (400.0, 200.0), (1000.0, 800.0)];
        let m = ScrollMapper::build(1200.0, 2000.0, &anchors);

        /* WHY: editor y=600 is halfway between line 10 (200px) and line 50 (1000px).
        Since the middle anchor is non-monotonic, it gets skipped.
        Preview midpoint between 400 and 800 is 600. */
        let pos = m.editor_to_logical(600.0);
        assert_eq!(pos.segment_index, 1);
        let p_y = m.logical_to_preview(pos);
        assert_eq!(p_y, 600.0);
    }

    #[test]
    fn snap_to_heading_within_threshold() {
        /* WHY: Heading at editor_y=200, preview_y=400. */
        let mapper = ScrollMapper::build(1000.0, 2000.0, &[(200.0, 400.0)]);
        /* WHY: Offset 195 is 5px from anchor 200 → should snap. */
        assert_eq!(mapper.snap_to_heading_editor(195.0), 200.0);
        /* WHY: Offset 215 is 15px from anchor 200 → should snap. */
        assert_eq!(mapper.snap_to_heading_editor(215.0), 200.0);
        /* WHY: Preview: offset 385 is 15px from anchor 400 → should snap. */
        assert_eq!(mapper.snap_to_heading_preview(385.0), 400.0);
    }

    #[test]
    fn snap_to_heading_beyond_threshold() {
        let mapper = ScrollMapper::build(1000.0, 2000.0, &[(200.0, 400.0)]);
        /* WHY: Offset 170 is 30px from anchor 200 → should NOT snap. */
        assert_eq!(mapper.snap_to_heading_editor(170.0), 170.0);
    }

    #[test]
    fn snap_no_headings_passthrough() {
        let mapper = ScrollMapper::build(1000.0, 800.0, &[]);
        /* WHY: No heading anchors → offset returned as-is. */
        assert_eq!(mapper.snap_to_heading_editor(500.0), 500.0);
    }
}
