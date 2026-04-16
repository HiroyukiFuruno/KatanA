use crate::markdown::types::DiagramKind;
use crate::preview::types::DiagramSectionOps;

impl DiagramSectionOps {
    pub fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
        /* WHY: Standard markdown triple-backtick fences. */
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

        /* WHY: Drawio <mxGraphModel tags. */
        if s.starts_with("<mxGraphModel") {
            if let Some(end_pos) = s.find("</mxGraphModel>") {
                let end_tag_len = "</mxGraphModel>".len();
                let source = &s[..end_pos + end_tag_len];
                let after = &s[end_pos + end_tag_len..];
                let after = after.strip_prefix('\n').unwrap_or(after);
                return Some((DiagramKind::DrawIo, source.to_string(), after));
            }
        }

        /* WHY: PlantUML @startuml markers. */
        if s.starts_with("@startuml") {
            if let Some(end_pos) = s.find("@enduml") {
                let end_tag_len = "@enduml".len();
                let source = &s[..end_pos + end_tag_len];
                let after = &s[end_pos + end_tag_len..];
                let after = after.strip_prefix('\n').unwrap_or(after);
                return Some((DiagramKind::PlantUml, source.to_string(), after));
            }
        }

        None
    }
}
