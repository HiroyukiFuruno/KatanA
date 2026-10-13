pub mod types;
pub use types::*;

pub const ECHO_PIXEL_EPSILON: f32 = 2.0;
const DEGENERATE_EPSILON: f32 = 1e-4;
pub const HEADING_SNAP_THRESHOLD: f32 = 20.0;

impl ScrollMapper {
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
            /* WHY: Only push if strictly monotonic to avoid degenerate linear segments */
            if points.last().is_some_and(|last| {
                editor_y > last.editor_y + DEGENERATE_EPSILON
                    && preview_y > last.preview_y + DEGENERATE_EPSILON
            }) {
                points.push(MapPoint {
                    editor_y,
                    preview_y,
                });
            }
        }

        /* WHY: Always record a point at exactly EOF to ensure final coverage. */
        /* WHY: If both content are significantly longer than a single window height, */
        /* WHY: we map their respective bottom-most scroll limits together.            */
        let eof_editor = editor_max;
        let eof_preview = preview_max;

        if points.last().is_some_and(|last| {
            (last.editor_y - eof_editor).abs() > DEGENERATE_EPSILON
                || (last.preview_y - eof_preview).abs() > DEGENERATE_EPSILON
        }) {
            /* WHY: If EOF is strictly after the last anchor, add a segment to it */
            let last = points.last().unwrap();
            if eof_editor > last.editor_y && eof_preview > last.preview_y {
                points.push(MapPoint {
                    editor_y: eof_editor,
                    preview_y: eof_preview,
                });
            } else {
                /* WHY: If the last anchor was already at or near EOF, force the last point to exactly EOF */
                let last_mut = points.last_mut().unwrap();
                last_mut.editor_y = eof_editor;
                last_mut.preview_y = eof_preview;
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

    fn to_logical(&self, y: f32, get: impl Fn(&MapPoint) -> f32) -> LogicalPosition {
        if self.points.len() < 2 {
            return LogicalPosition::default();
        }

        for i in 0..self.points.len() - 1 {
            let p1 = get(&self.points[i]);
            let p2 = get(&self.points[i + 1]);

            if y >= p1 && y <= p2 {
                let range = p2 - p1;
                let progress = if range > DEGENERATE_EPSILON {
                    (y - p1) / range
                } else {
                    0.0
                };
                return LogicalPosition {
                    segment_index: i,
                    progress,
                };
            }
        }

        if y > get(self.points.last().unwrap()) {
            LogicalPosition {
                segment_index: self.points.len() - 2,
                progress: 1.0,
            }
        } else {
            LogicalPosition::default()
        }
    }

    fn eval_logical_to_offset(&self, p: LogicalPosition, get: impl Fn(&MapPoint) -> f32) -> f32 {
        if p.segment_index + 1 >= self.points.len() {
            return p.progress;
        }
        let p1 = get(&self.points[p.segment_index]);
        let p2 = get(&self.points[p.segment_index + 1]);
        p1 + (p2 - p1) * p.progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_to_logical() {
        let anchors = vec![(100.0, 200.0)];
        let mapper = ScrollMapper::build(1000.0, 1000.0, &anchors);

        let p = mapper.editor_to_logical(50.0);
        assert_eq!(p.segment_index, 0);
        assert_eq!(p.progress, 0.5);

        let p2 = mapper.editor_to_logical(550.0);
        assert_eq!(p2.segment_index, 1);
        assert_eq!(p2.progress, 0.5);
    }

    #[test]
    fn test_preview_to_logical() {
        let anchors = vec![(100.0, 200.0)];
        let mapper = ScrollMapper::build(1000.0, 1000.0, &anchors);

        let p = mapper.preview_to_logical(100.0);
        assert_eq!(p.segment_index, 0);
        assert_eq!(p.progress, 0.5);

        let p2 = mapper.preview_to_logical(600.0);
        assert_eq!(p2.segment_index, 1);
        assert_eq!(p2.progress, 0.5);
    }

    #[test]
    fn test_roundtrip_stability() {
        let anchors = vec![(100.0, 250.0), (300.0, 450.0)];
        let mapper = ScrollMapper::build(1000.0, 1000.0, &anchors);

        for y in (0..1000).step_by(10) {
            let y = y as f32;
            let logical_p = mapper.editor_to_logical(y);
            let back_y = mapper.logical_to_editor(logical_p);
            assert!((y - back_y).abs() < 1e-4, "Editor drift at {}", y);

            let preview_y = mapper.logical_to_preview(logical_p);
            let logical_back = mapper.preview_to_logical(preview_y);
            assert_eq!(
                logical_p.segment_index, logical_back.segment_index,
                "Segment mismatch at {}",
                y
            );
            assert!(
                (logical_p.progress - logical_back.progress).abs() < 1e-6,
                "Progress drift at {}",
                y
            );
        }
    }

    #[test]
    fn mapper_tail_segment_reaches_eof() {
        let anchors = vec![(800.0, 600.0)];
        let mapper = ScrollMapper::build(1000.0, 1000.0, &anchors);

        assert_eq!(mapper.points.len(), 3);
        assert_eq!(
            mapper.points[2],
            MapPoint {
                editor_y: 1000.0,
                preview_y: 1000.0
            }
        );

        let p = mapper.editor_to_logical(1000.0);
        assert_eq!(p.segment_index, 1);
        assert_eq!(p.progress, 1.0);
        assert_eq!(mapper.logical_to_preview(p), 1000.0);
    }

    #[test]
    fn mapper_no_headings_full_range() {
        let mapper = ScrollMapper::build(1000.0, 2000.0, &[]);
        assert_eq!(mapper.points.len(), 2);
        assert_eq!(
            mapper.points[0],
            MapPoint {
                editor_y: 0.0,
                preview_y: 0.0
            }
        );
        assert_eq!(
            mapper.points[1],
            MapPoint {
                editor_y: 1000.0,
                preview_y: 2000.0
            }
        );

        let p = mapper.editor_to_logical(500.0);
        assert_eq!(p.progress, 0.5);
        assert_eq!(mapper.logical_to_preview(p), 1000.0);
    }

    #[test]
    fn mapper_skips_non_monotonic_segments() {
        let anchors = vec![(100.0, 200.0), (105.0, 195.0), (110.0, 210.0)];
        let mapper = ScrollMapper::build(1000.0, 1000.0, &anchors);

        assert_eq!(
            mapper.points[0],
            MapPoint {
                editor_y: 0.0,
                preview_y: 0.0
            }
        );
        assert_eq!(
            mapper.points[1],
            MapPoint {
                editor_y: 100.0,
                preview_y: 200.0
            }
        );
        /* WHY: (105, 195) is non-monotonic relative to (100, 200) for preview_y, so it should be skipped. */
        assert_eq!(
            mapper.points[2],
            MapPoint {
                editor_y: 110.0,
                preview_y: 210.0
            }
        );
    }

    #[test]
    fn snap_to_heading_within_threshold() {
        let anchors = vec![(100.0, 200.0)];
        let mapper = ScrollMapper::build(1000.0, 1000.0, &anchors);

        assert_eq!(mapper.snap_to_heading_editor(105.0), 100.0);
        assert_eq!(mapper.snap_to_heading_preview(195.0), 200.0);
    }

    #[test]
    fn snap_to_heading_beyond_threshold() {
        let anchors = vec![(100.0, 200.0)];
        let mapper = ScrollMapper::build(1000.0, 1000.0, &anchors);

        assert_eq!(mapper.snap_to_heading_editor(150.0), 150.0);
        assert_eq!(mapper.snap_to_heading_preview(250.0), 250.0);
    }

    #[test]
    fn snap_no_headings_passthrough() {
        let mapper = ScrollMapper::build(1000.0, 1000.0, &[]);
        assert_eq!(mapper.snap_to_heading_editor(100.0), 100.0);
    }

    #[test]
    fn echo_suppression_within_epsilon() {
        let echo = SyncEcho { offset_y: 100.0 };
        assert!(echo.is_echo(101.5));
        assert!(echo.is_echo(98.5));
    }

    #[test]
    fn echo_suppression_new_user_scroll() {
        let echo = SyncEcho { offset_y: 100.0 };
        assert!(!echo.is_echo(105.0));
    }
}
