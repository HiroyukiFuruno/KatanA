use crate::markdown::color_preset::DiagramColorPreset;
use std::path::Path;

const MERMAID_CAPTURE_SCALE: f32 = 1.25;

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
      background: {background};
    }}
    #diagram {{
      display: inline-block;
      background: {background};
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
    function scaleRenderedSvg(svg) {{
      if (!svg) {{
        return;
      }}
      const box = svg.getBBox();
      const padding = 8;
      const viewBox = {{
        x: Math.floor(box.x - padding),
        y: Math.floor(box.y - padding),
        width: Math.ceil(box.width + padding * 2),
        height: Math.ceil(box.height + padding * 2)
      }};
      const width = Math.ceil(viewBox.width * {capture_scale});
      const height = Math.ceil(viewBox.height * {capture_scale});
      svg.setAttribute('viewBox', `${{viewBox.x}} ${{viewBox.y}} ${{viewBox.width}} ${{viewBox.height}}`);
      svg.setAttribute('width', String(width));
      svg.setAttribute('height', String(height));
      svg.style.width = `${{width}}px`;
      svg.style.height = `${{height}}px`;
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
