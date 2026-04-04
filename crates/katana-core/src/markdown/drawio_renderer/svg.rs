use xmltree::Element;

use super::types::{DrawioEdgeOps, DrawioSvgOps, DrawioVertexOps};
use crate::markdown::color_preset::DiagramColorPreset;

/// Default geometry values when mxGeometry attributes are missing.
const DEFAULT_CELL_WIDTH: f64 = 100.0;
const DEFAULT_CELL_HEIGHT: f64 = 60.0;

impl DrawioSvgOps {
    pub fn render_drawio_to_svg(source: &str) -> anyhow::Result<String> {
        let xml = Element::parse(source.as_bytes())?;
        // WHY: Element::parse returns the outermost tag as root.
        // Possible structures:
        //   1) <mxGraphModel><root>...</root></mxGraphModel>
        //   2) <mxfile><diagram><mxGraphModel><root>...</root></mxGraphModel></diagram></mxfile>
        let model = if xml.name == "mxGraphModel" {
            &xml
        } else if xml.name == "mxfile" {
            // WHY: mxfile wraps diagram which wraps mxGraphModel
            xml.get_child("diagram")
                .and_then(|d| d.get_child("mxGraphModel"))
                .or_else(|| xml.get_child("mxGraphModel"))
                .ok_or_else(|| anyhow::anyhow!("Invalid draw.io XML: missing mxGraphModel"))?
        } else {
            xml.get_child("mxGraphModel")
                .ok_or_else(|| anyhow::anyhow!("Invalid draw.io XML: missing mxGraphModel"))?
        };
        let root = match model.get_child("root") {
            Some(r) => r,
            None => {
                // WHY: Empty mxGraphModel without <root> is valid — return empty SVG
                return Ok(r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#.to_string());
            }
        };

        let mut vertices = Vec::new();
        let mut edges = Vec::new();
        let mut geo_map = Vec::new();

        for child in &root.children {
            let el = match child {
                xmltree::XMLNode::Element(e) => e,
                _ => continue,
            };

            if el
                .attributes
                .get("vertex")
                .map(|v| v == "1")
                .unwrap_or(false)
            {
                vertices.push(el);
                if let (Some(id), Some(geo)) = (el.attributes.get("id"), el.get_child("mxGeometry"))
                {
                    let x = geo
                        .attributes
                        .get("x")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let y = geo
                        .attributes
                        .get("y")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let w = geo
                        .attributes
                        .get("width")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(DEFAULT_CELL_WIDTH);
                    let h = geo
                        .attributes
                        .get("height")
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(DEFAULT_CELL_HEIGHT);
                    geo_map.push((id.clone(), (x, y, w, h)));
                }
            } else if el.attributes.get("edge").map(|v| v == "1").unwrap_or(false) {
                edges.push(el);
            }
        }

        let preset = DiagramColorPreset::default();
        let mut shapes = String::new();
        let mut labels = String::new();

        for vertex in vertices {
            DrawioVertexOps::render_vertex(vertex, &mut shapes, &mut labels, &preset);
        }

        for edge in edges {
            DrawioEdgeOps::render_edge(edge, &mut shapes, &geo_map, &preset);
        }

        let mut svg = String::from(r#"<svg xmlns="http://www.w3.org/2000/svg">"#);
        svg.push_str(&shapes);
        svg.push_str(&labels);
        svg.push_str("</svg>");
        Ok(svg)
    }
}
