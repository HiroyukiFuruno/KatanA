use crate::markdown::types::DiagramKind;
use crate::preview::types::DiagramSectionOps;

impl DiagramSectionOps {
    pub fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
        if let Some(rest) = s.strip_prefix("```") {
            if let Some((info, rest)) = rest.split_once('\n') {
                if let Some(kind) = DiagramKind::from_info(info.trim()) {
                    if let Some((source, after)) = rest.split_once("\n```") {
                        let after = after.strip_prefix('\n').unwrap_or(after);
                        return Some((kind, source.to_string(), after));
                    }
                }
            }
        }

        None
    }
}
