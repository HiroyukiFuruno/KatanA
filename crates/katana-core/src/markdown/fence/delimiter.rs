#[derive(Clone, Copy)]
pub enum MarkdownDiagramMarker {
    Fence,
    Drawio,
    Plantuml,
}

#[derive(Clone, Copy)]
pub struct MarkdownFenceClosing {
    pub content_end: usize,
    pub close_end: usize,
}

#[derive(Clone, Copy)]
pub struct MarkdownFenceDelimiter {
    marker_byte: u8,
    len: usize,
}

impl MarkdownFenceDelimiter {
    const MIN_LEN: usize = 3;

    pub fn parse_at(source: &str) -> Option<Self> {
        let marker_byte = match source.as_bytes().first()? {
            b'`' => b'`',
            b'~' => b'~',
            _ => return None,
        };
        let len = source
            .bytes()
            .take_while(|byte| *byte == marker_byte)
            .count();
        (len >= Self::MIN_LEN).then_some(Self { marker_byte, len })
    }

    pub fn byte_len(self) -> usize {
        self.len
    }

    pub fn find_closing(self, body: &str) -> Option<MarkdownFenceClosing> {
        let mut line_start = 0;
        loop {
            let line = &body[line_start..];
            if let Some(line_end) = self.closing_line_end(line) {
                return Some(MarkdownFenceClosing {
                    content_end: line_start.saturating_sub(1),
                    close_end: line_start + line_end,
                });
            }

            let newline = line.find('\n')?;
            line_start += newline + 1;
        }
    }

    fn closing_line_end(self, line: &str) -> Option<usize> {
        let marker_len = line
            .bytes()
            .take_while(|byte| *byte == self.marker_byte)
            .count();
        if marker_len < self.len {
            return None;
        }

        let after_marker = &line[marker_len..];
        let trailing_end = after_marker.find('\n').unwrap_or(after_marker.len());
        let trailing = &after_marker[..trailing_end];
        trailing
            .chars()
            .all(|character| character == ' ' || character == '\t' || character == '\r')
            .then_some(marker_len + trailing_end)
    }
}
