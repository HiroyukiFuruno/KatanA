use crate::preview::types::MathPreviewOps;
use std::borrow::Cow;

impl MathPreviewOps {
    pub fn process_relaxed_math(source: &str) -> Cow<'_, str> {
        if source.contains('$') {
            Cow::Owned(source.replace('$', "$$"))
        } else {
            Cow::Borrowed(source)
        }
    }
}
