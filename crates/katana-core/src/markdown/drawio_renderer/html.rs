use std::path::Path;

pub(super) struct DrawioHtmlOps;

impl DrawioHtmlOps {
    pub(super) fn write_temp_html(
        xml: &str,
        drawio_js: &Path,
    ) -> Result<tempfile::NamedTempFile, anyhow::Error> {
        let html = Self::build(xml, drawio_js);
        let temp_html = tempfile::Builder::new()
            .prefix("katana_drawio_")
            .suffix(".html")
            .tempfile()?;
        std::fs::write(temp_html.path(), html)?;
        Ok(temp_html)
    }

    fn build(xml: &str, drawio_js: &Path) -> String {
        let graph_config = serde_json::json!({ "xml": xml }).to_string();
        let graph_config_attr = Self::html_attribute_escape(&graph_config);
        let drawio_url = format!("file://{}", drawio_js.to_string_lossy());

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    html,
    body {{
      margin: 0;
      padding: 0;
      background: transparent;
    }}
    #graph-container {{
      display: inline-block;
      max-width: 100%;
      background: transparent;
      border: 1px solid transparent;
    }}
  </style>
</head>
<body>
  <div id="graph-container" class="mxgraph" data-mxgraph='{graph_config_attr}'></div>
  <script src="{drawio_url}"></script>
</body>
</html>"#,
            drawio_url = drawio_url,
            graph_config_attr = graph_config_attr
        )
    }

    fn html_attribute_escape(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('\'', "&#39;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}
