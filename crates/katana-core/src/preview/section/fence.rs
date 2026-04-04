use crate::markdown::types::DiagramKind;
use crate::preview::types::DiagramSectionOps;

impl DiagramSectionOps {
    pub fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
        let (info, rest) = s.strip_prefix("```")?.split_once('\n')?;
        let kind = DiagramKind::from_info(info.trim())?;
        let (source, after) = rest.split_once("\n```")?;
        let after = after.strip_prefix('\n').unwrap_or(after);
        Some((kind, source.to_string(), after))
    }
}
