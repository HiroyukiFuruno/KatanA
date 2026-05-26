pub(super) struct KdvMarkdownNormalizer;

const MIN_FENCE_MARKER_LEN: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq)]
enum FenceMarker {
    Backtick,
    Tilde,
}

#[derive(Clone, Copy)]
struct FenceLine<'a> {
    indent: &'a str,
    marker: FenceMarker,
    marker_len: usize,
    suffix: &'a str,
}

impl KdvMarkdownNormalizer {
    pub(super) fn normalize(source: &str) -> String {
        let mut output = Vec::new();
        let lines = source.lines().collect::<Vec<_>>();
        let mut index = 0;

        while index < lines.len() {
            let Some(fence) = Self::parse_fence_line(lines[index]) else {
                output.push(lines[index].to_string());
                index += 1;
                continue;
            };
            let Some(block_end) = Self::block_end(&lines, index, fence) else {
                output.extend(lines[index..].iter().map(|line| (*line).to_string()));
                break;
            };
            Self::push_fence_block(&mut output, &lines, index, block_end, fence);
            index = block_end + 1;
        }

        let mut normalized = output.join("\n");
        if source.ends_with('\n') {
            normalized.push('\n');
        }
        normalized
    }

    fn push_fence_block(
        output: &mut Vec<String>,
        lines: &[&str],
        start: usize,
        end: usize,
        fence: FenceLine<'_>,
    ) {
        if fence.marker == FenceMarker::Backtick || end == start {
            output.extend(lines[start..=end].iter().map(|line| (*line).to_string()));
            return;
        }

        let backtick_len = Self::backtick_len_for_block(&lines[start..=end]);
        output.push(Self::convert_tilde_line(lines[start], backtick_len));
        output.extend(lines[start + 1..end].iter().map(|line| (*line).to_string()));
        output.push(Self::convert_tilde_line(lines[end], backtick_len));
    }

    fn block_end(lines: &[&str], start: usize, fence: FenceLine<'_>) -> Option<usize> {
        let mut index = start + 1;
        while index < lines.len() {
            if Self::is_closing_line(lines[index], fence) {
                return Some(index);
            }
            index += 1;
        }
        None
    }

    fn parse_fence_line(line: &str) -> Option<FenceLine<'_>> {
        let indent_len = line
            .bytes()
            .take_while(|byte| matches!(*byte, b' ' | b'\t'))
            .count();
        let body = &line[indent_len..];
        let marker = Self::line_marker(body)?;
        let marker_len = body
            .bytes()
            .take_while(|byte| Self::marker_matches(*byte, marker))
            .count();
        (marker_len >= MIN_FENCE_MARKER_LEN).then_some(FenceLine {
            indent: &line[..indent_len],
            marker,
            marker_len,
            suffix: &body[marker_len..],
        })
    }

    fn is_closing_line(line: &str, opening: FenceLine<'_>) -> bool {
        let Some(candidate) = Self::parse_fence_line(line) else {
            return false;
        };
        candidate.marker == opening.marker
            && candidate.marker_len >= opening.marker_len
            && candidate
                .suffix
                .chars()
                .all(|character| character == ' ' || character == '\t')
    }

    fn line_marker(body: &str) -> Option<FenceMarker> {
        match body.as_bytes().first()? {
            b'`' => Some(FenceMarker::Backtick),
            b'~' => Some(FenceMarker::Tilde),
            _ => None,
        }
    }

    fn marker_matches(byte: u8, marker: FenceMarker) -> bool {
        matches!(
            (byte, marker),
            (b'`', FenceMarker::Backtick) | (b'~', FenceMarker::Tilde)
        )
    }

    fn backtick_len_for_block(lines: &[&str]) -> usize {
        lines
            .iter()
            .map(|line| Self::max_backtick_run(line))
            .max()
            .unwrap_or(0)
            .saturating_add(1)
            .max(MIN_FENCE_MARKER_LEN)
    }

    fn max_backtick_run(line: &str) -> usize {
        line.split(|character| character != '`')
            .map(str::len)
            .max()
            .unwrap_or(0)
    }

    fn convert_tilde_line(line: &str, backtick_len: usize) -> String {
        let fence = Self::parse_fence_line(line).expect("tilde fence line must be valid");
        format!(
            "{}{}{}",
            fence.indent,
            "`".repeat(backtick_len),
            fence.suffix
        )
    }
}

#[cfg(test)]
mod tests {
    use super::KdvMarkdownNormalizer;

    #[test]
    fn normalize_converts_tilde_fence_to_backtick_fence() {
        let source = "before\n~~~mermaid\ngraph TD; A-->B\n~~~\nafter\n";
        let normalized = KdvMarkdownNormalizer::normalize(source);
        assert_eq!(
            normalized,
            "before\n```mermaid\ngraph TD; A-->B\n```\nafter\n"
        );
    }

    #[test]
    fn normalize_keeps_nested_tilde_fence_inside_backtick_fence() {
        let source = "```markdown\n~~~mermaid\ngraph TD; A-->B\n~~~\n```\n";
        let normalized = KdvMarkdownNormalizer::normalize(source);
        assert_eq!(normalized, source);
    }

    #[test]
    fn normalize_keeps_tilde_content_inside_outer_tilde_fence() {
        let source = "~~~~markdown\n~~~mermaid\ngraph TD; A-->B\n~~~\n~~~~\n";
        let normalized = KdvMarkdownNormalizer::normalize(source);
        assert_eq!(
            normalized,
            "```markdown\n~~~mermaid\ngraph TD; A-->B\n~~~\n```\n"
        );
    }

    #[test]
    fn normalize_expands_backtick_fence_when_content_needs_it() {
        let source = "~~~markdown\n```mermaid\ngraph TD; A-->B\n```\n~~~\n";
        let normalized = KdvMarkdownNormalizer::normalize(source);
        assert_eq!(
            normalized,
            "````markdown\n```mermaid\ngraph TD; A-->B\n```\n````\n"
        );
    }

    #[test]
    fn normalize_keeps_unclosed_fence_unchanged() {
        let source = "~~~markdown\n~~~mermaid\ngraph TD; A-->B\n";
        let normalized = KdvMarkdownNormalizer::normalize(source);
        assert_eq!(normalized, source);
    }
}
