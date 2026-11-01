use crate::state::search::SearchParams;

pub(super) struct FilterPattern {
    pub(super) regex: regex::Regex,
    pub(super) is_negated: bool,
}

impl FilterPattern {
    pub(super) fn compile(params: &SearchParams) -> Result<Self, regex::Error> {
        let (query, is_negated) = match params.query.strip_prefix('!') {
            Some(query) => (query, true),
            None => (params.query.as_str(), false),
        };
        let pattern = if params.use_regex {
            query.to_string()
        } else {
            Self::normal_mode_pattern(query, std::path::MAIN_SEPARATOR)
        };
        let pattern = if params.match_word {
            format!(r"\b{pattern}\b")
        } else {
            pattern
        };
        let regex = regex::RegexBuilder::new(&pattern)
            .case_insensitive(!params.match_case)
            .build()?;
        Ok(Self { regex, is_negated })
    }

    fn normal_mode_pattern(query: &str, separator: char) -> String {
        let escaped = regex::escape(query);
        if !query.contains('*') {
            return escaped;
        }

        let wildcard = Self::wildcard_pattern(query, separator);
        let has_path_separator = query.contains('/') || (separator == '\\' && query.contains('\\'));
        let separator_pattern = if separator == '\\' { r"[/\\]" } else { r"/" };
        if has_path_separator {
            format!(r"^{wildcard}$")
        } else {
            format!(r"(?:^|{separator_pattern}){wildcard}$")
        }
    }

    fn wildcard_pattern(query: &str, separator: char) -> String {
        let separator_pattern = if separator == '\\' { r"[/\\]" } else { r"/" };
        let wildcard_pattern = if separator == '\\' {
            r"[^/\\]*"
        } else {
            r"[^/]*"
        };
        let mut pattern = String::new();

        for character in query.chars() {
            match character {
                '*' => pattern.push_str(wildcard_pattern),
                '/' => pattern.push_str(separator_pattern),
                '\\' if separator == '\\' => pattern.push_str(separator_pattern),
                _ => pattern.push_str(&regex::escape(&character.to_string())),
            }
        }

        pattern
    }
}

#[cfg(test)]
#[path = "filter_pattern_tests.rs"]
mod tests;
