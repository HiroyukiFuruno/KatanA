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

    /// Returns how many bytes of `remaining` to consume when the fence at `pos` is
    /// not a recognized diagram kind (e.g. ` ```markdown `, ` ````rust `, etc.).
    ///
    /// Implements CommonMark closing-fence detection:
    /// - The opening fence length is the **actual** backtick run at `remaining[pos..]`
    ///   (may be > 3, e.g. 4 for ```` ````markdown ````).
    /// - A closing fence is a line whose leading backtick run is ≥ opening length
    ///   AND has no language identifier (only optional trailing spaces).
    /// - Inner fences with a language id (e.g. ` ```mermaid `) are therefore
    ///   never mistaken for closing fences, so nested diagram blocks remain invisible
    ///   to the outer `split_sections` loop and are not extracted as Diagram sections.
    ///
    /// For non-backtick markers (`<mxGraphModel`, `@startuml`) a simple marker-length
    /// skip is safe because `try_parse_diagram_fence` already validates the tag.
    pub fn non_diagram_fence_consume_len(pos: usize, marker: &str, remaining: &str) -> usize {
        if marker != "```" {
            return pos + marker.len();
        }
        /* Count the actual backtick run at the fence position — may be > 3. */
        let fence_len = remaining[pos..].bytes().take_while(|b| *b == b'`').count();

        /* Walk every line after the opening fence looking for a valid closing fence. */
        let mut scan = pos + fence_len; /* start scanning after the opening backtick run */
        while let Some(nl_rel) = remaining[scan..].find('\n') {
            let line_start = scan + nl_rel + 1;
            let line = &remaining[line_start..];

            /* Count leading backticks on this line. */
            let bt = line.bytes().take_while(|b| *b == b'`').count();
            if bt >= fence_len {
                /* Per CommonMark: after the backtick run only spaces then newline/EOF
                 * are permitted. A language identifier disqualifies a closing fence. */
                let after_bt = line[bt..].trim_start_matches(' ');
                if after_bt.is_empty() || after_bt.starts_with('\n') {
                    let close_end = line_start + bt;
                    let trailing = remaining
                        .get(close_end..)
                        .is_some_and(|s| s.starts_with('\n'));
                    return close_end + usize::from(trailing);
                }
            }
            scan = line_start;
        }
        /* No closing fence found — consume just the opening marker length. */
        pos + marker.len()
    }
}
