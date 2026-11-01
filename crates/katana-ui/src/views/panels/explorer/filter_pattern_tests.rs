use super::FilterPattern;

#[test]
fn unix_wildcard_keeps_backslash_inside_a_path_segment() {
    let regex = regex::Regex::new(&FilterPattern::normal_mode_pattern("a*.png", '/')).unwrap();

    assert!(regex.is_match(r"a\b.png"));
    assert!(!regex.is_match(r"b\a.png"));
}

#[test]
fn windows_wildcards_accept_both_separator_spellings() {
    let basename =
        regex::Regex::new(&FilterPattern::normal_mode_pattern("icon*.png", '\\')).unwrap();
    assert!(basename.is_match(r"assets\icon.png"));

    let relative =
        regex::Regex::new(&FilterPattern::normal_mode_pattern("assets/*.png", '\\')).unwrap();
    assert!(relative.is_match(r"assets\icon.png"));
    assert!(!relative.is_match(r"assets\nested\icon.png"));

    let backslash_query =
        regex::Regex::new(&FilterPattern::normal_mode_pattern(r"assets\*.png", '\\')).unwrap();
    assert!(backslash_query.is_match(r"assets\icon.png"));
    assert!(!backslash_query.is_match(r"assets\nested\icon.png"));
}
