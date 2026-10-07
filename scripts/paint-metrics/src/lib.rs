//! Utilities for collecting reusable text geometry from `egui` paint output.
//!
//! Coordinates in this module are logical screen points. Convert them to
//! screenshot pixels with `FullOutput::pixels_per_point` at the call site.

use egui::FullOutput;
use serde_json::{Value, json};

#[path = "capture_text_shape.rs"]
mod capture_text_shape;

/// Capture every text shape in a frame's paint output.
pub fn capture(output: &FullOutput) -> Value {
    let mut texts = Vec::new();
    for clipped in &output.shapes {
        capture_text_shape::visit(&clipped.shape, clipped.clip_rect, &mut texts);
    }
    json!({
        "coordinate_space": "logical_screen_points",
        "glyph_band_kind": "unclipped_glyph_mesh_quad_bounds_not_raster_ink",
        "pixels_per_point": capture_text_shape::finite(output.pixels_per_point),
        "text_shapes": texts,
    })
}

#[path = "capture_tests.rs"]
#[cfg(test)]
mod tests;
