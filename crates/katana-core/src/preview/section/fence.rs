use crate::markdown::types::DiagramKind;
use crate::preview::types::DiagramSectionOps;

impl DiagramSectionOps {
    pub fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
        /* WHY: Standard markdown triple-backtick fences. */
        let try_fenced = || -> Option<(DiagramKind, String, &str)> {
            let rest = s.strip_prefix("```")?;
            let (info, rest) = rest.split_once('\n')?;
            let kind = DiagramKind::from_info(info.trim())?;
            let (source, after) = rest.split_once("\n```")?;
            let after = after.strip_prefix('\n').unwrap_or(after);
            Some((kind, source.to_string(), after))
        };
        if let Some(res) = try_fenced() {
            return Some(res);
        }

        /* WHY: Drawio <mxGraphModel tags. */
        let try_drawio = || -> Option<(DiagramKind, String, &str)> {
            if !s.starts_with("<mxGraphModel") {
                return None;
            }
            let end_pos = s.find("</mxGraphModel>")?;
            let end_tag_len = "</mxGraphModel>".len();
            let source = &s[..end_pos + end_tag_len];
            let after = &s[end_pos + end_tag_len..];
            let after = after.strip_prefix('\n').unwrap_or(after);
            Some((DiagramKind::DrawIo, source.to_string(), after))
        };
        if let Some(res) = try_drawio() {
            return Some(res);
        }

        /* WHY: PlantUML @startuml markers. */
        let try_plantuml = || -> Option<(DiagramKind, String, &str)> {
            if !s.starts_with("@startuml") {
                return None;
            }
            let end_pos = s.find("@enduml")?;
            let end_tag_len = "@enduml".len();
            let source = &s[..end_pos + end_tag_len];
            let after = &s[end_pos + end_tag_len..];
            let after = after.strip_prefix('\n').unwrap_or(after);
            Some((DiagramKind::PlantUml, source.to_string(), after))
        };
        if let Some(res) = try_plantuml() {
            return Some(res);
        }

        None
    }
}
