const MAX_VISIBLE_CHARS: usize = 80;
const MAX_SCAN_CHARS: usize = 4096;
const ELLIPSIS: &str = "...";

struct HtmlSummary {
    text: String,
    used: usize,
    pending_space: bool,
    overflow: bool,
    budget_exceeded: bool,
}

impl HtmlSummary {
    fn new() -> Self {
        Self {
            text: String::with_capacity(MAX_VISIBLE_CHARS + ELLIPSIS.len()),
            used: 0,
            pending_space: false,
            overflow: false,
            budget_exceeded: false,
        }
    }

    fn push(&mut self, ch: char) {
        if ch.is_whitespace() {
            self.pending_space = self.used > 0;
            return;
        }
        if self.pending_space {
            if self.used == MAX_VISIBLE_CHARS {
                self.overflow = true;
                return;
            }
            self.text.push('_');
            self.used += 1;
            self.pending_space = false;
        }
        if self.used == MAX_VISIBLE_CHARS {
            self.overflow = true;
            return;
        }
        self.text.push(ch);
        self.used += 1;
    }

    fn finish(mut self) -> String {
        if self.budget_exceeded {
            self.text = self
                .text
                .chars()
                .take(MAX_VISIBLE_CHARS - ELLIPSIS.chars().count())
                .collect();
            self.text.push_str(ELLIPSIS);
        }
        self.text
    }
}

pub(crate) fn summarize_html<I>(mut chars: I) -> String
where
    I: Iterator<Item = char>,
{
    let mut summary = HtmlSummary::new();
    for _ in 0..MAX_SCAN_CHARS {
        let Some(ch) = chars.next() else {
            return summary.finish();
        };
        summary.push(ch);
        if summary.overflow {
            return summary.finish();
        }
    }
    /* WHY: デバッグ要約は巨大なHTML/data URIのtailを走査せず、予算超過を省略表示する。 */
    summary.budget_exceeded = true;
    summary.finish()
}

#[cfg(test)]
mod tests {
    use super::{MAX_SCAN_CHARS, MAX_VISIBLE_CHARS, summarize_html};

    const TAIL_LIMIT: usize = 128;

    #[test]
    fn summary_compacts_unicode_whitespace_and_preserves_short_url() {
        assert_eq!(
            summarize_html(
                "\n <p>\u{3053}\u{3093}\u{306B}\u{3061}\u{306F}\t\u{4E16}\u{754C} </p>".chars()
            ),
            "<p>\u{3053}\u{3093}\u{306B}\u{3061}\u{306F}_\u{4E16}\u{754C}_</p>"
        );
        let url = "https://example.test/docs/index.html";
        assert_eq!(summarize_html(url.chars()), url);
    }

    #[test]
    fn summary_preserves_exact_visible_boundary() {
        let exact = "\u{754C}".repeat(MAX_VISIBLE_CHARS);
        assert_eq!(summarize_html(exact.chars()), exact);
        assert_eq!(summarize_html(format!("{exact}\u{5B57}").chars()), exact);
    }

    #[test]
    fn summary_is_bounded_for_huge_tail() {
        struct TailGuard {
            consumed: usize,
        }

        impl Iterator for TailGuard {
            type Item = char;

            fn next(&mut self) -> Option<Self::Item> {
                assert!(
                    self.consumed < TAIL_LIMIT,
                    "summary consumed unbounded tail"
                );
                self.consumed += 1;
                Some('x')
            }
        }

        let mut tail = TailGuard { consumed: 0 };
        let summary = summarize_html("data:image/svg+xml,".chars().chain(&mut tail));
        assert_eq!(summary.chars().count(), MAX_VISIBLE_CHARS);
        assert!(tail.consumed < TAIL_LIMIT);
    }

    #[test]
    fn summary_bounds_whitespace_scan_budget() {
        let mut consumed = 0;
        let summary = summarize_html(std::iter::from_fn(|| {
            assert!(consumed < MAX_SCAN_CHARS);
            consumed += 1;
            Some('\n')
        }));
        assert_eq!(summary, super::ELLIPSIS);
        assert_eq!(consumed, MAX_SCAN_CHARS);
    }
}
