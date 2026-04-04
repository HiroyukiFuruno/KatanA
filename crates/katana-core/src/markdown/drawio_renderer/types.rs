pub struct DrawioEdgeOps;
pub struct DrawioSvgOps;
pub struct DrawioVertexOps;
pub struct DrawioUtilsOps;

pub struct DrawioRendererOps;

pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}
