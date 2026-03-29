use xmltree::Element;

/// Minimum width of the Draw.io canvas (fallback when there are no elements).
pub const CANVAS_MIN_WIDTH: f64 = 400.0;

/// Minimum height of the Draw.io canvas (fallback when there are no elements).
pub const CANVAS_MIN_HEIGHT: f64 = 300.0;

/// Margin added from the edges of each element when estimating canvas size (px).
pub const CANVAS_EDGE_MARGIN: f64 = 20.0;

/// Struct holding the position and size of a rectangle.
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

/// Gets an XML attribute as `f64`. Returns 0.0 if it doesn't exist.
pub fn attr_f64(el: &Element, name: &str) -> f64 {
    el.attributes
        .get(name)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.0)
}

/// Extracts `key=value` from an mxGraph style string.
pub fn extract_style_value<'a>(style: &'a str, key: &str) -> Option<&'a str> {
    style.split(';').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k.trim() == key).then_some(v.trim())
    })
}

/// Minimal XML escape for SVG text nodes.
pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
