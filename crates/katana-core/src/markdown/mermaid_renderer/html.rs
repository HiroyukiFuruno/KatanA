use crate::markdown::color_preset::DiagramColorPreset;
use std::path::Path;

const MERMAID_CAPTURE_SCALE: f32 = 1.25;
const MERMAID_CAPTURE_MAX_WIDTH: u32 = 1200;
const MERMAID_RENDER_WIDTH: u32 = 800;

pub(super) struct MermaidHtmlOps;

impl MermaidHtmlOps {
    pub(super) fn build(
        encoded_source: &str,
        mermaid_js: &Path,
        preset: &DiagramColorPreset,
    ) -> String {
        let mermaid_url = format!("file://{}", mermaid_js.to_string_lossy());
        let background = css_background(preset.background);
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <script src="{mermaid_url}"></script>
  <style>
    html, body {{
      margin: 0;
      padding: 0;
      width: {render_width}px;
      background: {background};
    }}
    #diagram {{
      display: flex;
      justify-content: center;
      align-items: flex-start;
      width: {render_width}px;
      background: {background};
      overflow: hidden;
    }}
    #diagram svg {{
      display: block;
      max-width: none !important;
      background: {background};
    }}
  </style>
</head>
<body>
  <div id="diagram"></div>
  <script>
    const source = {encoded_source};
    const container = document.getElementById('diagram');
    async function renderDiagram() {{
      if (typeof mermaid === 'undefined') {{
        throw new Error('Mermaid library not loaded');
      }}
      mermaid.initialize({{
        startOnLoad: false,
        securityLevel: 'loose',
        theme: '{theme}',
        flowchart: {{
          useMaxWidth: false
        }},
        sequence: {{
          useMaxWidth: false
        }},
        themeVariables: {{
          background: '{background}',
          mainBkg: '{fill}',
          primaryColor: '{fill}',
          primaryTextColor: '{text}',
          primaryBorderColor: '{stroke}',
          secondaryColor: '{fill}',
          secondaryTextColor: '{text}',
          secondaryBorderColor: '{stroke}',
          tertiaryColor: '{fill}',
          tertiaryTextColor: '{text}',
          tertiaryBorderColor: '{stroke}',
          nodeTextColor: '{text}',
          lineColor: '{arrow}',
          textColor: '{text}',
          edgeLabelBackground: '{fill}',
          actorBkg: '{fill}',
          actorTextColor: '{text}',
          actorBorder: '{stroke}',
          signalColor: '{arrow}',
          signalTextColor: '{text}',
          labelTextColor: '{text}',
          noteBkgColor: '{fill}',
          noteTextColor: '{text}',
          noteBorderColor: '{stroke}',
          clusterBkg: '{background}',
          clusterBorder: '{stroke}',
          titleColor: '{text}'
        }}
      }});
      const result = await mermaid.render('katana-mermaid-svg', source);
      container.innerHTML = result.svg;
      scaleRenderedSvg(container.querySelector('svg'));
    }}
    function hideOutOfRangeTodayMarker(svg) {{
      const markers = Array.from(svg.querySelectorAll('#today,.today,[class*="today"]'));
      if (markers.length === 0) {{
        return;
      }}
      const originalDisplay = markers.map((marker) => marker.style.display);
      markers.forEach((marker) => marker.style.display = 'none');
      const contentBox = svg.getBBox();
      markers.forEach((marker, index) => marker.style.display = originalDisplay[index]);
      markers.forEach((marker) => {{
        const markerBox = marker.getBBox();
        const horizontalEnd = contentBox.x + contentBox.width;
        const markerStart = markerBox.x;
        const markerEnd = markerBox.x + markerBox.width;
        const isOutside = markerEnd < contentBox.x || markerStart > horizontalEnd;
        if (isOutside) {{
          marker.style.display = 'none';
        }}
      }});
    }}
    function scaleRenderedSvg(svg) {{
      if (!svg) {{
        return;
      }}
      hideOutOfRangeTodayMarker(svg);
      const box = svg.getBBox();
      const padding = 8;
      const viewBox = {{
        x: Math.floor(box.x - padding),
        y: Math.floor(box.y - padding),
        width: Math.ceil(box.width + padding * 2),
        height: Math.ceil(box.height + padding * 2)
      }};
      const requestedWidth = viewBox.width * {capture_scale};
      const cappedScale = Math.min({capture_scale}, {capture_max_width} / requestedWidth * {capture_scale});
      const width = Math.ceil(viewBox.width * cappedScale);
      const height = Math.ceil(viewBox.height * cappedScale);
      svg.setAttribute('viewBox', `${{viewBox.x}} ${{viewBox.y}} ${{viewBox.width}} ${{viewBox.height}}`);
      svg.setAttribute('width', String(width));
      svg.setAttribute('height', String(height));
      svg.style.width = `${{width}}px`;
      svg.style.height = `${{height}}px`;
      container.style.width = `${{width}}px`;
      container.style.height = `${{height}}px`;
    }}
    window.katanaRenderError = '';
    renderDiagram()
      .then(() => document.body.setAttribute('data-katana-rendered', 'true'))
      .catch((error) => {{
        window.katanaRenderError = String(error && error.message ? error.message : error);
        document.body.setAttribute('data-katana-rendered', 'error');
      }});
  </script>
</body>
</html>"#,
            mermaid_url = mermaid_url,
            encoded_source = encoded_source,
            theme = preset.mermaid_theme,
            background = background,
            fill = preset.fill,
            text = preset.text,
            stroke = preset.stroke,
            arrow = preset.arrow,
            capture_scale = MERMAID_CAPTURE_SCALE,
            capture_max_width = MERMAID_CAPTURE_MAX_WIDTH,
            render_width = MERMAID_RENDER_WIDTH,
        )
    }
}

fn css_background(background: &str) -> &str {
    if background == "transparent" {
        "rgba(0, 0, 0, 0)"
    } else {
        background
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_html_excludes_out_of_range_today_marker_before_sizing() {
        let html = MermaidHtmlOps::build(
            "\"gantt\\ndateFormat YYYY-MM-DD\"",
            Path::new("mermaid.min.js"),
            &DiagramColorPreset::default(),
        );

        let hide_marker = html.find("hideOutOfRangeTodayMarker(svg);").unwrap();
        let measure_svg = html.find("const box = svg.getBBox();").unwrap();
        assert!(hide_marker < measure_svg);
        assert!(html.contains("#today,.today,[class*=\"today\"]"));
        assert!(html.contains("width: 800px;"));
    }
}
