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
            Self::normal_mode_pattern(query)
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

    fn normal_mode_pattern(query: &str) -> String {
        let escaped = regex::escape(query);
        if !query.contains('*') {
            return escaped;
        }

        let wildcard = escaped.replace(r"\*", "[^/]*");
        if query.contains('/') {
            format!(r"^{wildcard}$")
        } else {
            format!(r"(?:^|/){wildcard}$")
        }
    }
}
