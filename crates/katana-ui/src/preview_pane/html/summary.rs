const MAX_HTML_SUMMARY_CHARS: usize = 80;
const MAX_HTML_SUMMARY_SCAN_CHARS: usize = 4096;
const HTML_SUMMARY_ELLIPSIS: &str = "...";

pub(super) struct HtmlSummary {
    text: String,
    used: usize,
    pending_space: bool,
    omitted: bool,
}

impl HtmlSummary {
    pub(super) fn summarize_html(html: &str) -> String {
        summarize_html_chars(html.chars())
    }

    fn new() -> Self {
        Self {
            text: String::with_capacity(MAX_HTML_SUMMARY_CHARS + HTML_SUMMARY_ELLIPSIS.len()),
            used: 0,
            pending_space: false,
            omitted: false,
        }
    }

    fn push(&mut self, ch: char) {
        if ch.is_whitespace() {
            self.pending_space = self.used > 0;
            return;
        }
        if self.pending_space {
            if self.used == MAX_HTML_SUMMARY_CHARS {
                self.omitted = true;
                return;
            }
            self.text.push('_');
            self.used += 1;
            self.pending_space = false;
        }
        if self.used == MAX_HTML_SUMMARY_CHARS {
            self.omitted = true;
            return;
        }
        self.text.push(ch);
        self.used += 1;
    }

    fn finish(mut self) -> String {
        if self.omitted {
            self.text.push_str(HTML_SUMMARY_ELLIPSIS);
        }
        self.text
    }
}

fn summarize_html_chars<I>(mut chars: I) -> String
where
    I: Iterator<Item = char>,
{
    let mut summary = HtmlSummary::new();
    /* WHY: 診断用要約は巨大なHTML/data URIのtailを走査せず、上限到達時は保守的に省略表示する。 */
    for _ in 0..MAX_HTML_SUMMARY_SCAN_CHARS {
        let Some(ch) = chars.next() else {
            return summary.finish();
        };
        summary.push(ch);
        if summary.omitted {
            return summary.finish();
        }
    }
    summary.omitted = true;
    summary.finish()
}

#[cfg(test)]
mod tests {
    use super::{HtmlSummary, MAX_HTML_SUMMARY_SCAN_CHARS, summarize_html_chars};

    const EXACT_PREFIX_CHARS: usize = 79;
    const SUMMARY_WITH_ELLIPSIS_CHARS: usize = 83;
    const TAIL_GUARD_LIMIT: usize = 64;

    #[test]
    fn summarize_html_compacts_whitespace_and_preserves_unicode() {
        assert_eq!(
            HtmlSummary::summarize_html(
                "\n <p> \u{3053}\u{3093}\u{306B}\u{3061}\u{306F}\t\u{4E16}\u{754C} </p>\u{000b}"
            ),
            "<p>_\u{3053}\u{3093}\u{306B}\u{3061}\u{306F}_\u{4E16}\u{754C}_</p>"
        );
    }

    #[test]
    fn summarize_html_preserves_short_url() {
        let url = "https://example.test/docs/index.html";
        assert_eq!(HtmlSummary::summarize_html(url), url);
    }

    #[test]
    fn summarize_html_keeps_exact_compact_boundary() {
        let exact = format!("{} z", "x".repeat(EXACT_PREFIX_CHARS));
        let omitted = format!("{} zz", "x".repeat(EXACT_PREFIX_CHARS));
        assert_eq!(
            HtmlSummary::summarize_html(&exact).chars().count(),
            SUMMARY_WITH_ELLIPSIS_CHARS
        );
        assert_eq!(
            HtmlSummary::summarize_html(&omitted).chars().count(),
            SUMMARY_WITH_ELLIPSIS_CHARS
        );
    }

    #[test]
    fn summarize_html_preserves_unicode_at_visible_limit() {
        let exact = "\u{754c}".repeat(super::MAX_HTML_SUMMARY_CHARS);
        assert_eq!(HtmlSummary::summarize_html(&exact), exact);
        assert_eq!(HtmlSummary::summarize_html(&format!("{exact}\n\t")), exact);
        assert_eq!(
            HtmlSummary::summarize_html(&format!("{exact}\u{5b57}")),
            format!("{exact}...")
        );
        let prefix = "\u{754c}".repeat(EXACT_PREFIX_CHARS);
        assert_eq!(
            HtmlSummary::summarize_html(&format!("{prefix}\n\u{5b57}")),
            format!("{prefix}_...")
        );
    }

    #[test]
    fn summarize_html_does_not_consume_unbounded_tail() {
        struct TailGuard {
            consumed: usize,
        }

        impl Iterator for TailGuard {
            type Item = char;

            fn next(&mut self) -> Option<Self::Item> {
                assert!(
                    self.consumed < TAIL_GUARD_LIMIT,
                    "summary consumed too much tail"
                );
                self.consumed += 1;
                Some('x')
            }
        }

        let mut tail = TailGuard { consumed: 0 };
        let summary = summarize_html_chars("data:image/svg+xml,".chars().chain(&mut tail));
        assert_eq!(summary.chars().count(), SUMMARY_WITH_ELLIPSIS_CHARS);
        assert!(summary.starts_with("data:image/svg+xml,"));
        assert!(summary.ends_with("..."));
        assert!(tail.consumed < TAIL_GUARD_LIMIT);
    }

    #[test]
    fn summarize_html_bounds_leading_and_trailing_whitespace() {
        struct WhitespaceGuard {
            consumed: usize,
        }

        impl Iterator for WhitespaceGuard {
            type Item = char;

            fn next(&mut self) -> Option<Self::Item> {
                assert!(self.consumed < MAX_HTML_SUMMARY_SCAN_CHARS);
                self.consumed += 1;
                Some('\n')
            }
        }

        let mut leading = WhitespaceGuard { consumed: 0 };
        assert_eq!(summarize_html_chars(&mut leading), "...");
        let mut trailing = WhitespaceGuard { consumed: 0 };
        assert_eq!(
            summarize_html_chars("x".chars().chain(&mut trailing)),
            "x..."
        );
    }
}
