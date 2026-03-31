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

/// One entry in the piecewise-linear mapping between editor and preview pixel offsets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapPoint {
    /// Editor pixel offset (0..=editor_max).
    pub editor_y: f32,
    /// Preview pixel offset (0..=preview_max).
    pub preview_y: f32,
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
        let mut points = Vec::with_capacity(anchors.len() + 2);
        points.push(MapPoint {
            editor_y: 0.0,
            preview_y: 0.0,
        });
        for (span, p_y) in anchors {
            let editor_y = span.start as f32 * row_height;
            // Clamp to avoid degenerate points outside the visible range.
            let editor_y = editor_y.min(editor_max.max(1.0));
            let preview_y = p_y.max(0.0).min(preview_max.max(1.0));
            points.push(MapPoint {
                editor_y,
                preview_y,
            });
        }
        // EOF anchor — always present regardless of heading count.
        points.push(MapPoint {
            editor_y: editor_max.max(1.0),
            preview_y: preview_max.max(1.0),
        });
        Self { points }
    }

    /// Map an editor pixel offset to a preview pixel offset.
    pub fn editor_to_preview(&self, editor_y: f32) -> f32 {
        self.interpolate(editor_y, |p| p.editor_y, |p| p.preview_y)
    }

    /// Map a preview pixel offset to an editor pixel offset.
    pub fn preview_to_editor(&self, preview_y: f32) -> f32 {
        self.interpolate(preview_y, |p| p.preview_y, |p| p.editor_y)
    }

    fn interpolate(
        &self,
        src: f32,
        get_src: impl Fn(&MapPoint) -> f32,
        get_dst: impl Fn(&MapPoint) -> f32,
    ) -> f32 {
        let pts = &self.points;
        if pts.is_empty() {
            return src;
        }
        // Clamp below first point.
        if src <= get_src(&pts[0]) {
            return get_dst(&pts[0]);
        }
        // Clamp above last point.
        let last = pts.last().unwrap();
        if src >= get_src(last) {
            return get_dst(last);
        }
        // Linear interpolation within the matching segment.
        for i in 0..pts.len() - 1 {
            let s0 = get_src(&pts[i]);
            let s1 = get_src(&pts[i + 1]);
            if src >= s0 && src <= s1 {
                let d0 = get_dst(&pts[i]);
                let d1 = get_dst(&pts[i + 1]);
                if s1 > s0 {
                    return d0 + (src - s0) / (s1 - s0) * (d1 - d0);
                } else {
                    return d0;
                }
            }
        }
        get_dst(last)
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

    fn simple_anchors() -> Vec<(std::ops::Range<usize>, f32)> {
        // Line 10 → preview y 100, line 20 → preview y 200
        vec![(10..11, 100.0), (20..21, 200.0)]
    }

    #[test]
    fn mapper_start_maps_to_zero() {
        let m = ScrollMapper::build(500.0, 400.0, 10.0, &simple_anchors());
        assert_eq!(m.editor_to_preview(0.0), 0.0);
        assert_eq!(m.preview_to_editor(0.0), 0.0);
    }

    #[test]
    fn mapper_eof_maps_to_eof() {
        let m = ScrollMapper::build(500.0, 400.0, 10.0, &simple_anchors());
        assert_eq!(m.editor_to_preview(500.0), 400.0);
        assert_eq!(m.preview_to_editor(400.0), 500.0);
    }

    #[test]
    fn mapper_no_headings_full_range() {
        // Document with no headings: single [start, EOF] segment.
        let m = ScrollMapper::build(1000.0, 800.0, 10.0, &[]);
        // Mid-point should map proportionally across the single segment.
        let preview_y = m.editor_to_preview(500.0);
        assert!(
            (preview_y - 400.0).abs() < 1.0,
            "expected ~400, got {preview_y}"
        );
        let editor_y = m.preview_to_editor(400.0);
        assert!(
            (editor_y - 500.0).abs() < 1.0,
            "expected ~500, got {editor_y}"
        );
    }

    #[test]
    fn mapper_tail_segment_reaches_eof() {
        // Last heading at editor line 40, preview y 300; editor_max=500, preview_max=400.
        let anchors = vec![(40..41, 300.0)];
        let m = ScrollMapper::build(500.0, 400.0, 10.0, &anchors);
        // Editor fully scrolled to tail end → preview fully scrolled too.
        assert_eq!(m.editor_to_preview(500.0), 400.0);
        assert_eq!(m.preview_to_editor(400.0), 500.0);
    }

    #[test]
    fn mapper_round_trip_no_drift() {
        let m = ScrollMapper::build(500.0, 400.0, 10.0, &simple_anchors());
        for editor_y in [0.0f32, 50.0, 150.0, 250.0, 499.0, 500.0] {
            let preview_y = m.editor_to_preview(editor_y);
            let back = m.preview_to_editor(preview_y);
            assert!(
                (back - editor_y).abs() < 1.0,
                "round-trip drift at editor_y={editor_y}: preview={preview_y}, back={back}"
            );
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
        // A genuine user scroll of 50px should not be suppressed.
        assert!(!echo.is_echo(300.0));
    }
}
