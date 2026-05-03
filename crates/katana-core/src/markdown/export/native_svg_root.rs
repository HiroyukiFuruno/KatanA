use crate::markdown::MarkdownError;

pub(crate) struct NativeSvgRoot;

impl NativeSvgRoot {
    pub(crate) fn with_numeric_size(
        svg: &str,
        width: u32,
        height: u32,
    ) -> Result<String, MarkdownError> {
        let Some(open_end) = svg.find('>') else {
            return Ok(svg.to_string());
        };
        let open_tag = &svg[..open_end];
        let rest = &svg[open_end..];
        let sized_open_tag = Self::set_attribute(open_tag, "width", &width.to_string())?;
        let sized_open_tag = Self::set_attribute(&sized_open_tag, "height", &height.to_string())?;
        Ok(format!("{sized_open_tag}{rest}"))
    }

    fn set_attribute(open_tag: &str, name: &str, value: &str) -> Result<String, MarkdownError> {
        let regex = regex::Regex::new(&format!(r#"(?is)\s{name}="[^"]*""#))
            .map_err(|error| MarkdownError::ExportFailed(error.to_string()))?;
        if regex.is_match(open_tag) {
            Ok(regex
                .replace(open_tag, format!(r#" {name}="{value}""#))
                .to_string())
        } else {
            Ok(format!(r#"{open_tag} {name}="{value}""#))
        }
    }
}
