use super::scroll_sync::{LogicalPosition, MapPoint, ScrollMapper};

impl ScrollMapper {
    pub(super) fn to_logical(
        &self,
        src: f32,
        get_src: impl Fn(&MapPoint) -> f32,
    ) -> LogicalPosition {
        let pts = &self.points;
        if pts.is_empty() {
            return LogicalPosition {
                segment_index: 0,
                progress: 0.0,
            };
        }
        if src <= get_src(&pts[0]) {
            return LogicalPosition {
                segment_index: 0,
                progress: 0.0,
            };
        }
        let last_idx = pts.len() - 1;
        if src >= get_src(&pts[last_idx]) {
            return LogicalPosition {
                segment_index: last_idx,
                progress: 1.0,
            };
        }
        for i in 0..last_idx {
            let s0 = get_src(&pts[i]);
            let s1 = get_src(&pts[i + 1]);
            if src >= s0 && src <= s1 {
                let progress = if s1 > s0 { (src - s0) / (s1 - s0) } else { 0.0 };
                return LogicalPosition {
                    segment_index: i,
                    progress,
                };
            }
        }
        LogicalPosition {
            segment_index: last_idx,
            progress: 1.0,
        }
    }

    pub(super) fn eval_logical_to_offset(
        &self,
        pos: LogicalPosition,
        get_dst: impl Fn(&MapPoint) -> f32,
    ) -> f32 {
        let pts = &self.points;
        if pts.is_empty() {
            return 0.0;
        }
        if pos.segment_index >= pts.len() - 1 {
            return get_dst(&pts[pts.len() - 1]);
        }
        let d0 = get_dst(&pts[pos.segment_index]);
        let d1 = get_dst(&pts[pos.segment_index + 1]);
        d0 + (d1 - d0) * pos.progress.clamp(0.0, 1.0)
    }
}
