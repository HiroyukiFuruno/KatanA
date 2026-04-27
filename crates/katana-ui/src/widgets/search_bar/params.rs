use crate::state::search::SearchParams;

pub(super) enum SearchParamsRef<'a> {
    Full(&'a mut SearchParams),
    Simple(&'a mut String),
}

impl<'a> SearchParamsRef<'a> {
    pub(super) fn query_mut(&mut self) -> &mut String {
        match self {
            Self::Full(params) => &mut params.query,
            Self::Simple(query) => query,
        }
    }

    pub(super) fn query(&self) -> &str {
        match self {
            Self::Full(params) => &params.query,
            Self::Simple(query) => query,
        }
    }

    pub(super) fn toggles(&mut self) -> Option<(&mut bool, &mut bool, &mut bool)> {
        match self {
            Self::Full(params) => Some((
                &mut params.match_case,
                &mut params.match_word,
                &mut params.use_regex,
            )),
            Self::Simple(_) => None,
        }
    }
}
