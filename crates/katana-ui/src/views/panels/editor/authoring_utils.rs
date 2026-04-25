use super::types::AuthoringTransform;

pub struct AuthoringUtils;

impl AuthoringUtils {
    /// Wrap the current selection (or insert markers at cursor) with `prefix` … `suffix`.
    pub fn wrap_inline(
        buffer: &str,
        lo: usize,
        hi: usize,
        selected: &str,
        prefix: &str,
        suffix: &str,
    ) -> AuthoringTransform {
        let before = &buffer[..lo];
        let after = &buffer[hi..];
        let inner = if selected.is_empty() {
            "text".to_string()
        } else {
            selected.to_string()
        };
        let inserted = format!("{prefix}{inner}{suffix}");
        let new_cursor_start = before.len() + prefix.len();
        let new_cursor_end = new_cursor_start + inner.len();
        AuthoringTransform {
            buffer: format!("{before}{inserted}{after}"),
            cursor_start: new_cursor_start,
            cursor_end: new_cursor_end,
        }
    }

    /// Prefix the current line (determined by the cursor position) with `prefix`.
    pub fn prefix_line(
        buffer: &str,
        lo: usize,
        _hi: usize,
        _selected: &str,
        prefix: &str,
    ) -> AuthoringTransform {
        /* WHY: Find the start of the line that contains `lo`. */
        let line_start = buffer[..lo].rfind('\n').map(|n| n + 1).unwrap_or(0);
        let before = &buffer[..line_start];
        let after = &buffer[line_start..];
        let new_offset = line_start + prefix.len() + (lo - line_start);
        AuthoringTransform {
            buffer: format!("{before}{prefix}{after}"),
            cursor_start: new_offset,
            cursor_end: new_offset,
        }
    }

    /// Prefix every line in the selection with `prefix`.
    pub fn prefix_each_line(
        buffer: &str,
        lo: usize,
        hi: usize,
        selected: &str,
        prefix: &str,
    ) -> AuthoringTransform {
        /* WHY: Align lo to the start of its containing line. */
        let line_start = buffer[..lo].rfind('\n').map(|n| n + 1).unwrap_or(0);
        let region = if selected.is_empty() {
            /* WHY: No selection — prefix only the current line. */
            let line_end = buffer[lo..]
                .find('\n')
                .map(|n| lo + n)
                .unwrap_or(buffer.len());
            &buffer[line_start..line_end]
        } else {
            &buffer[line_start..hi.min(buffer.len())]
        };

        /* WHY: When region is empty (cursor on blank line or past all content),
         * `.lines()` returns nothing and joining gives "" — the prefix would never
         * be inserted yet new_start still advances by prefix.len(), going out of
         * bounds.  Insert the prefix directly instead. */
        if region.is_empty() {
            let before = &buffer[..line_start];
            let after = &buffer[line_start..];
            let new_cursor = before.len() + prefix.len();
            return AuthoringTransform {
                buffer: format!("{before}{prefix}{after}"),
                cursor_start: new_cursor,
                cursor_end: new_cursor,
            };
        }

        let prefixed: String = region
            .lines()
            .map(|l| format!("{prefix}{l}"))
            .collect::<Vec<_>>()
            .join("\n");

        let before = &buffer[..line_start];
        let after_start = (line_start + region.len()).min(buffer.len());
        let after = &buffer[after_start..];

        let new_start = before.len() + prefix.len();
        let new_end = before.len() + prefixed.len();
        AuthoringTransform {
            buffer: format!("{before}{prefixed}{after}"),
            cursor_start: new_start,
            cursor_end: new_end,
        }
    }

    /// Prefix each line in selection with an incrementing number ("1. ", "2. ", …).
    pub fn prefix_each_line_numbered(
        buffer: &str,
        lo: usize,
        hi: usize,
        selected: &str,
    ) -> AuthoringTransform {
        let line_start = buffer[..lo].rfind('\n').map(|n| n + 1).unwrap_or(0);
        let region = if selected.is_empty() {
            let line_end = buffer[lo..]
                .find('\n')
                .map(|n| lo + n)
                .unwrap_or(buffer.len());
            &buffer[line_start..line_end]
        } else {
            &buffer[line_start..hi.min(buffer.len())]
        };

        /* WHY: Same empty-region guard as prefix_each_line — see that function. */
        if region.is_empty() {
            let before = &buffer[..line_start];
            let after = &buffer[line_start..];
            let new_cursor = before.len() + "1. ".len();
            return AuthoringTransform {
                buffer: format!("{before}1. {after}"),
                cursor_start: new_cursor,
                cursor_end: new_cursor,
            };
        }

        let prefixed: String = region
            .lines()
            .enumerate()
            .map(|(i, l)| format!("{}. {l}", i + 1))
            .collect::<Vec<_>>()
            .join("\n");

        let before = &buffer[..line_start];
        let after_start = (line_start + region.len()).min(buffer.len());
        let after = &buffer[after_start..];

        let new_start = before.len() + "1. ".len();
        let new_end = before.len() + prefixed.len();
        AuthoringTransform {
            buffer: format!("{before}{prefixed}{after}"),
            cursor_start: new_start,
            cursor_end: new_end,
        }
    }

    /// Wrap the selection (or insert a placeholder) between `open` and `close` block markers.
    pub fn wrap_block(
        buffer: &str,
        lo: usize,
        hi: usize,
        selected: &str,
        open: &str,
        close: &str,
    ) -> AuthoringTransform {
        let before = &buffer[..lo];
        let after = &buffer[hi..];
        let inner = if selected.is_empty() {
            "code here".to_string()
        } else {
            selected.to_string()
        };
        let inserted = format!("{open}{inner}{close}");
        /* WHY: Position cursor just after the opening fence so user types code. */
        let new_cursor_start = before.len() + open.len();
        let new_cursor_end = new_cursor_start + inner.len();
        AuthoringTransform {
            buffer: format!("{before}{inserted}{after}"),
            cursor_start: new_cursor_start,
            cursor_end: new_cursor_end,
        }
    }

    /// Insert a literal snippet at the cursor position (no selection used).
    pub fn insert_snippet(buffer: &str, lo: usize, snippet: &str) -> AuthoringTransform {
        let before = &buffer[..lo];
        let after = &buffer[lo..];
        let new_cursor = before.len() + snippet.len();
        AuthoringTransform {
            buffer: format!("{before}{snippet}{after}"),
            cursor_start: new_cursor,
            cursor_end: new_cursor,
        }
    }
}
