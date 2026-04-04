use super::types::HtmlRegexOps;
use regex::Regex;

impl HtmlRegexOps {
    pub fn br() -> &'static Regex {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?is)<br\s*/?>"#).unwrap());
        &RE
    }

    pub fn img() -> &'static Regex {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(?is)<img\s+([^>]+)>"#).unwrap());
        &RE
    }

    pub fn a() -> &'static Regex {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"(?is)<a\s+[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#).unwrap()
        });
        &RE
    }

    pub fn p() -> &'static Regex {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(?is)<p\s+([^>]*)>(.*?)</p>"#).unwrap());
        &RE
    }

    pub fn heading() -> &'static Regex {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(?is)<h([1-6])([^>]*)>(.*?)</h[1-6]>"#).unwrap());
        &RE
    }

    pub fn em() -> &'static Regex {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(?is)<em>(.*?)</em>"#).unwrap());
        &RE
    }

    pub fn strong() -> &'static Regex {
        use std::sync::LazyLock;
        static RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r#"(?is)<strong>(.*?)</strong>"#).unwrap());
        &RE
    }

    pub fn extract_attr(attrs: &str, attr_name: &str) -> Option<String> {
        let re = Regex::new(&format!(r#"(?is){}\s*=\s*"([^"]+)""#, attr_name)).ok()?;
        re.captures(attrs)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }
}
