use egui::{Pos2, Rect, epaint::Shape};
use serde_json::{Value, json};

pub(crate) fn finite(value: f32) -> Option<f32> {
    value.is_finite().then_some(value)
}

fn pos(pos: Pos2) -> Value {
    json!({ "x": finite(pos.x), "y": finite(pos.y) })
}

fn rect(rect: Rect) -> Value {
    json!({
        "min": pos(rect.min),
        "max": pos(rect.max),
        "width": finite(rect.width()),
        "height": finite(rect.height()),
    })
}

fn mesh_band_for_range(
    row: &egui::epaint::text::PlacedRow,
    origin: Pos2,
    range: std::ops::Range<usize>,
) -> Value {
    let vertices = &row.visuals.mesh.vertices;
    let mut bounds = Rect::NOTHING;
    let mut has_vertex = false;
    for vertex in vertices.get(range).into_iter().flatten() {
        if vertex.pos.x.is_finite() && vertex.pos.y.is_finite() {
            let p = vertex.pos + origin.to_vec2();
            bounds = if has_vertex {
                bounds.union(Rect::from_min_max(p, p))
            } else {
                has_vertex = true;
                Rect::from_min_max(p, p)
            };
        }
    }
    if has_vertex {
        rect(bounds)
    } else {
        Value::Null
    }
}

fn text_shape(shape: &egui::epaint::TextShape, clip_rect: Rect) -> Value {
    let galley = &shape.galley;
    if shape.angle != 0.0 {
        return json!({
            "text": galley.job.text,
            "clip_rect": rect(clip_rect),
            "angle": finite(shape.angle),
            "coordinate_status": "unsupported_rotation",
        });
    }
    let rows = galley
        .rows
        .iter()
        .map(|row| {
            let row_origin = shape.pos + row.pos.to_vec2();
            let glyphs = row
                .glyphs
                .iter()
                .enumerate()
                .map(|(index, glyph)| {
                    let vertex_start = glyph.first_vertex as usize;
                    let vertex_end = row
                        .glyphs
                        .get(index + 1)
                        .map_or(row.visuals.glyph_vertex_range.end, |next| {
                            next.first_vertex as usize
                        });
                    let vertex_range = vertex_start.max(row.visuals.glyph_vertex_range.start)
                        ..vertex_end.min(row.visuals.glyph_vertex_range.end);
                    json!({
                        "char": glyph.chr.to_string(),
                        "baseline": pos(shape.pos + row.pos.to_vec2() + glyph.pos.to_vec2()),
                        "baseline_offset_from_row_top": finite(glyph.pos.y),
                        "advance_width": finite(glyph.advance_width),
                        "line_height": finite(glyph.line_height),
                        "font_height": finite(glyph.font_height),
                        "font_ascent": finite(glyph.font_ascent),
                        "font_face_height": finite(glyph.font_face_height),
                        "glyph_band": mesh_band_for_range(
                            row,
                            row_origin,
                            vertex_range,
                        ),
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "text": row.text(),
                "pos": pos(row_origin),
                "rect": rect(row.rect().translate(shape.pos.to_vec2())),
                "mesh_bounds": mesh_band_for_range(
                    row,
                    row_origin,
                    row.visuals.glyph_vertex_range.clone(),
                ),
                "ends_with_newline": row.ends_with_newline,
                "glyphs": glyphs,
            })
        })
        .collect::<Vec<_>>();
    let sections = galley
        .job
        .sections
        .iter()
        .map(|section| {
            json!({
                "byte_start": section.byte_range.start.0,
                "byte_end": section.byte_range.end.0,
                "leading_space": finite(section.leading_space),
                "font_size": finite(section.format.font_id.size),
                "font_family": format!("{:?}", section.format.font_id.family),
                "line_height": section.format.line_height.and_then(finite),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "text": galley.job.text,
        "origin": pos(shape.pos),
        "clip_rect": rect(clip_rect),
        "galley_rect": rect(galley.rect.translate(shape.pos.to_vec2())),
        "mesh_bounds": rect(galley.mesh_bounds.translate(shape.pos.to_vec2())),
        "rows": rows,
        "sections": sections,
    })
}

pub(crate) fn visit(shape: &Shape, clip_rect: Rect, texts: &mut Vec<Value>) {
    match shape {
        Shape::Vec(shapes) => shapes
            .iter()
            .for_each(|shape| visit(shape, clip_rect, texts)),
        Shape::Text(shape) => texts.push(text_shape(shape, clip_rect)),
        _ => {}
    }
}
