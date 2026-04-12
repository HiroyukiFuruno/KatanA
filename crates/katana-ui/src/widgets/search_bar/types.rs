pub enum SearchParamsRef<'a> {
    Full(&'a mut crate::state::search::SearchParams),
    Simple(&'a mut String),
}

impl<'a> SearchParamsRef<'a> {
    pub fn query(&self) -> &str {
        match self {
            Self::Full(p) => &p.query,
            Self::Simple(q) => q,
        }
    }

    pub fn query_mut(&mut self) -> &mut String {
        match self {
            Self::Full(p) => &mut p.query,
            Self::Simple(q) => q,
        }
    }

    pub fn toggles(&mut self) -> Option<(&mut bool, &mut bool, &mut bool)> {
        match self {
            Self::Full(p) => Some((&mut p.match_case, &mut p.match_word, &mut p.use_regex)),
            Self::Simple(_) => None,
        }
    }
}
