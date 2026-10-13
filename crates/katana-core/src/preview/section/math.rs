use crate::preview::types::MathPreviewOps;
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

static MATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    /* WHY: Match sequences starting and ending with `$`, but not `$$`.
    Captures the content inside. Non-greedy `+?` ensures we only match valid pairs
    and do not consume the entire document. */
    Regex::new(r"(?s)\$([^$]+?)\$").unwrap()
});

impl MathPreviewOps {
    pub fn process_relaxed_math(source: &str) -> Cow<'_, str> {
        if source.contains('$') {
            /* WHY: Protect existing `$$` with a null byte, convert `$ ... $` pairs,
            then restore the protected `$$`. Strings like "$ 500 $ 10." will
            be replaced if they happen to form a pair, but regex handles it cleaner. */
            let safe = source.replace("$$", "\x00");
            let replaced = MATH_RE.replace_all(&safe, "$$$$${1}$$$$");
            let final_str = replaced.replace('\x00', "$$");
            Cow::Owned(final_str)
        } else {
            Cow::Borrowed(source)
        }
    }
}
