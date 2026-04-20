use super::types::HtmlExporter;
use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{DiagramRenderer, MarkdownError, MarkdownRenderOps};

impl HtmlExporter {
    /* WHY: Exports Markdown as a standalone HTML document with embedded CSS.
    When `base_dir` is provided, relative image paths in the rendered HTML are
    resolved to absolute `file://` URLs so that images display correctly even
    when the HTML is opened from a different directory (e.g. a temp file). */
    pub fn export<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, MarkdownError> {
        let output = MarkdownRenderOps::render(source, renderer)?;
        let bg_color = Self::get_bg_color(preset);
        let props = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji'";
        let monos = "SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace";

        let css = Self::generate_css(preset, bg_color, props, monos);
        let body = match base_dir {
            Some(dir) => Self::resolve_relative_paths(&output.html, dir),
            None => output.html,
        };

        Ok(Self::assemble_html_document(&css, &body))
    }

    fn assemble_html_document(css: &str, body: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Exported Document</title>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css">
<style>
{css}
</style>
</head>
<body>
{body}
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"></script>
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/contrib/auto-render.min.js"
  onload="renderMathInElement(document.body, {{
    delimiters: [
      {{left: '$$', right: '$$', display: true}},
      {{left: '$', right: '$', display: false}}
    ],
    throwOnError: false
  }});
  document.querySelectorAll('[data-math-style=display]').forEach(function(el) {{
    var src = el.textContent;
    el.innerHTML = '';
    katex.render(src, el, {{displayMode: true, throwOnError: false}});
  }});
  document.querySelectorAll('[data-math-style=inline]').forEach(function(el) {{
    var src = el.textContent;
    el.innerHTML = '';
    katex.render(src, el, {{displayMode: false, throwOnError: false}});
  }});
"></script>
</body>
</html>"#
        )
    }

    fn resolve_relative_paths(html: &str, base_dir: &std::path::Path) -> String {
        /* WHY: Match src="..." that don't start with http://, https://, data:, or file:// */
        let re = regex::Regex::new(r#"src="([^"]+)""#).unwrap();
        re.replace_all(html, |caps: &regex::Captures| {
            let src = &caps[1];
            if src.starts_with("http://")
                || src.starts_with("https://")
                || src.starts_with("data:")
                || src.starts_with("file://")
            {
                caps[0].to_string()
            } else {
                let abs = base_dir.join(src);
                format!("src=\"file://{}\"", abs.display())
            }
        })
        .to_string()
    }

    fn get_bg_color(preset: &DiagramColorPreset) -> &str {
        if preset.background == "transparent" {
            if preset.text == "#E0E0E0" {
                "#1e1e1e"
            } else {
                "#ffffff"
            }
        } else {
            preset.background
        }
    }

    fn generate_css(
        preset: &DiagramColorPreset,
        bg_color: &str,
        props: &str,
        monos: &str,
    ) -> String {
        let base = Self::generate_base_css(preset, bg_color, props, monos);
        let elems = Self::generate_elements_css(preset, bg_color);
        format!("{base}{elems}")
    }

    fn generate_base_css(
        preset: &DiagramColorPreset,
        bg_color: &str,
        props: &str,
        monos: &str,
    ) -> String {
        format!(
            r#"
body {{ font-family: {props}; background-color: {bg_color}; color: {text}; line-height: 1.6; max-width: 900px; margin: 0 auto; padding: 2rem; }}
h1, h2, h3, h4, h5, h6 {{ margin-top: 1.5em; margin-bottom: 0.5em; font-weight: 600; }}
h1 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
h2 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
a {{ color: #0366d6; text-decoration: none; }}
pre {{ background-color: {fill}; border: 1px solid {stroke}; border-radius: 6px; padding: 16px; overflow: auto; line-height: 1.5; }}
code {{ font-family: {monos}; background-color: {fill}; border-radius: 3px; padding: 0.2em 0.4em; font-size: 85%; }}
pre code {{ background-color: transparent; padding: 0; }}
"#,
            props = props,
            bg_color = bg_color,
            text = preset.text,
            stroke = preset.stroke,
            fill = preset.fill,
            monos = monos
        )
    }

    fn generate_elements_css(preset: &DiagramColorPreset, bg_color: &str) -> String {
        let base = format!(
            r#"
blockquote {{ border-left: 0.25em solid {stroke}; color: {text}; opacity: 0.8; padding: 0 1em; margin: 0; }}
table {{ border-spacing: 0; border-collapse: collapse; margin-top: 0; margin-bottom: 16px; }}
table th, table td {{ padding: 6px 13px; border: 1px solid {stroke}; }}
img {{ max-width: 100%; box-sizing: content-box; background-color: {bg_color}; }}
.katana-diagram img {{ background-color: transparent; }}
hr {{ height: 0.25em; padding: 0; margin: 24px 0; background-color: {stroke}; border: 0; }}
"#,
            bg_color = bg_color,
            text = preset.text,
            stroke = preset.stroke
        );
        let alerts = Self::generate_alerts_css();
        let extras = Self::generate_extras_css(preset);
        format!("{base}{alerts}{extras}")
    }

    fn generate_alerts_css() -> String {
        /* WHY: GFM-compatible alert styling to match the preview pane's
        rendering of [!NOTE], [!TIP], [!IMPORTANT], [!WARNING], [!CAUTION]. */
        r#"
.markdown-alert { padding: 0.5rem 1rem; margin-bottom: 16px; border-left: 0.25em solid; border-radius: 4px; }
.markdown-alert-title { font-weight: 600; margin-bottom: 0.25rem; }
.markdown-alert-note { border-left-color: #539bf5; }
.markdown-alert-note .markdown-alert-title { color: #539bf5; }
.markdown-alert-tip { border-left-color: #57ab5a; }
.markdown-alert-tip .markdown-alert-title { color: #57ab5a; }
.markdown-alert-important { border-left-color: #986ee2; }
.markdown-alert-important .markdown-alert-title { color: #986ee2; }
.markdown-alert-warning { border-left-color: #c69026; }
.markdown-alert-warning .markdown-alert-title { color: #c69026; }
.markdown-alert-caution { border-left-color: #e5534b; }
.markdown-alert-caution .markdown-alert-title { color: #e5534b; }
"#.to_string()
    }

    fn generate_extras_css(preset: &DiagramColorPreset) -> String {
        /* WHY: Task list, footnote, math, and description list styles. */
        format!(
            r#"
ul.contains-task-list {{ list-style: none; padding-left: 1.5em; }}
input[type="checkbox"] {{ margin-right: 0.5em; }}
.footnotes {{ border-top: 1px solid {stroke}; margin-top: 2em; padding-top: 1em; font-size: 0.9em; }}
.footnote-ref {{ font-size: 0.75em; vertical-align: super; }}
math, .math-display, .math-inline {{ font-family: 'KaTeX_Main', 'Times New Roman', serif; }}
dt {{ font-weight: 600; margin-top: 0.5em; }}
dd {{ margin-left: 1.5em; margin-bottom: 0.5em; }}
"#,
            stroke = preset.stroke
        )
    }
}
