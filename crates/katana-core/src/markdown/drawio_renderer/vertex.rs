use super::types::DrawioVertexOps;
use xmltree::Element;

use crate::markdown::color_preset::DiagramColorPreset;

use super::types::DrawioUtilsOps;

impl DrawioVertexOps {
    pub fn render_vertex(
        cell: &Element,
        shapes: &mut String,
        labels: &mut String,
        preset: &DiagramColorPreset,
    ) {
        let Some(geo) = cell.get_child("mxGeometry") else {
            return;
        };
        let style = cell
            .attributes
            .get("style")
            .map(String::as_str)
            .unwrap_or("");
        let cx = DrawioUtilsOps::attr_f64(geo, "x")
            + DrawioUtilsOps::attr_f64(geo, "width").max(1.0) / 2.0;
        let cy = DrawioUtilsOps::attr_f64(geo, "y")
            + DrawioUtilsOps::attr_f64(geo, "height").max(1.0) / 2.0;
        Self::render_shape(geo, style, shapes, preset);
        Self::append_label(cell, cx, cy, labels, preset);
    }

    fn render_shape(geo: &Element, style: &str, shapes: &mut String, preset: &DiagramColorPreset) {
        let x = DrawioUtilsOps::attr_f64(geo, "x");
        let y = DrawioUtilsOps::attr_f64(geo, "y");
        let w = DrawioUtilsOps::attr_f64(geo, "width").max(1.0);
        let h = DrawioUtilsOps::attr_f64(geo, "height").max(1.0);
        let fill = DrawioUtilsOps::extract_style_value(style, "fillColor").unwrap_or(preset.fill);
        let stroke =
            DrawioUtilsOps::extract_style_value(style, "strokeColor").unwrap_or(preset.stroke);
        if style.contains("ellipse") {
            shapes.push_str(&format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#,
                x + w / 2.0,
                y + h / 2.0,
                w / 2.0,
                h / 2.0
            ));
        } else {
            let rx = if style.contains("rounded=1") {
                "6"
            } else {
                "0"
            };
            shapes.push_str(&format!(
                r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{fill}" stroke="{stroke}" stroke-width="1.5"/>"#
            ));
        }
    }

    fn append_label(
        cell: &Element,
        cx: f64,
        cy: f64,
        labels: &mut String,
        preset: &DiagramColorPreset,
    ) {
        let label = match cell.attributes.get("value") {
            Some(v) if !v.is_empty() => v.as_str(),
            _ => return,
        };
        let style = cell
            .attributes
            .get("style")
            .map(String::as_str)
            .unwrap_or("");
        let text_color = DrawioUtilsOps::extract_style_value(style, "fontColor")
            .unwrap_or(preset.drawio_label_color);
        labels.push_str(&format!(
            r#"<text x="{cx}" y="{cy}" dy="0.35em" text-anchor="middle" font-family="sans-serif" font-size="12" fill="{text_color}">{}</text>"#,
            DrawioUtilsOps::xml_escape(label)
        ));
    }
}
