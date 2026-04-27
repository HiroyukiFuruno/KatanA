use crate::markdown::fence::MarkdownFenceDelimiter;
use crate::markdown::types::DiagramKind;
use crate::preview::types::DiagramSectionOps;

impl DiagramSectionOps {
    pub fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
        if let Some(res) = Self::try_parse_fenced_diagram(s) {
            return Some(res);
        }

        if let Some(res) =
            Self::try_parse_raw_diagram(s, "<mxGraphModel", "</mxGraphModel>", DiagramKind::DrawIo)
        {
            return Some(res);
        }

        if let Some(res) =
            Self::try_parse_raw_diagram(s, "@startuml", "@enduml", DiagramKind::PlantUml)
        {
            return Some(res);
        }

        None
    }

    fn try_parse_fenced_diagram(s: &str) -> Option<(DiagramKind, String, &str)> {
        let delimiter = MarkdownFenceDelimiter::parse_at(s)?;
        let rest = &s[delimiter.byte_len()..];
        let (info, rest) = rest.split_once('\n')?;
        let kind = DiagramKind::from_info(info.trim())?;
        let closing = delimiter.find_closing(rest)?;
        let source = rest[..closing.content_end].to_string();
        let after = &rest[closing.close_end..];
        let after = after.strip_prefix('\n').unwrap_or(after);
        Some((kind, source, after))
    }

    fn try_parse_raw_diagram<'a>(
        s: &'a str,
        start_tag: &str,
        end_tag: &str,
        kind: DiagramKind,
    ) -> Option<(DiagramKind, String, &'a str)> {
        if !s.starts_with(start_tag) {
            return None;
        }
        let end_pos = s.find(end_tag)?;
        let end_tag_len = end_tag.len();
        let source = &s[..end_pos + end_tag_len];
        let after = &s[end_pos + end_tag_len..];
        let after = after.strip_prefix('\n').unwrap_or(after);
        Some((kind, source.to_string(), after))
    }

    /// Returns how many bytes of `remaining` to consume when the fence at `pos` is
    /// not a recognized diagram kind (e.g. ` ```markdown `, ` ````rust `, etc.).
    ///
    /// Implements CommonMark closing-fence detection:
    /// - The opening fence length is the **actual** marker run at `remaining[pos..]`
    ///   (may be > 3, e.g. 4 for ```` ````markdown ```` or `~~~~markdown`).
    /// - A closing fence is a line whose leading marker run is ≥ opening length
    ///   AND has no language identifier (only optional trailing spaces).
    /// - Inner fences with a language id (e.g. ` ```mermaid `) are therefore
    ///   never mistaken for closing fences, so nested diagram blocks remain invisible
    ///   to the outer `split_sections` loop and are not extracted as Diagram sections.
    ///
    /// For non-backtick markers (`<mxGraphModel`, `@startuml`) a simple marker-length
    /// skip is safe because `try_parse_diagram_fence` already validates the tag.
    pub fn non_diagram_fence_consume_len(pos: usize, marker: &str, remaining: &str) -> usize {
        if marker != "```" && marker != "~~~" {
            return pos + marker.len();
        }
        let Some(delimiter) = MarkdownFenceDelimiter::parse_at(&remaining[pos..]) else {
            return pos + marker.len();
        };
        let after_open = &remaining[pos + delimiter.byte_len()..];
        let Some(info_end) = after_open.find('\n') else {
            return pos + delimiter.byte_len();
        };
        let after_info = &after_open[info_end + 1..];
        let Some(closing) = delimiter.find_closing(after_info) else {
            return pos + delimiter.byte_len();
        };
        let close_end = pos + delimiter.byte_len() + info_end + 1 + closing.close_end;
        close_end + usize::from(remaining[close_end..].starts_with('\n'))
    }
}
