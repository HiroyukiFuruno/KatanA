#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapPoint {
    pub editor_y: f32,
    pub preview_y: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LogicalPosition {
    pub segment_index: usize,
    pub progress: f32,
}

#[derive(Debug, Default, Clone)]
pub struct ScrollMapper {
    pub points: Vec<MapPoint>,
    pub editor_content_max: f32,
    pub preview_content_max: f32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SyncEcho {
    pub offset_y: f32,
}

impl SyncEcho {
    pub fn is_echo(&self, current: f32) -> bool {
        (self.offset_y - current).abs() <= 2.0 // ECHO_PIXEL_EPSILON
    }
}
