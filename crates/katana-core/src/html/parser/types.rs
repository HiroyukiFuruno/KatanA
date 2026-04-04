use std::path::Path;

pub struct HtmlParser<'a> {
    pub base_dir: &'a Path,
}

pub struct HtmlInlineParserOps;
pub struct HtmlRegexOps;
